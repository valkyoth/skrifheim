#![forbid(unsafe_code)]

use std::{
    fmt,
    fs::{File, OpenOptions},
    io::{self, Read, Write},
    path::Path,
};

#[cfg(unix)]
use std::{
    fs::Permissions,
    os::unix::fs::{OpenOptionsExt, PermissionsExt},
};

use skrifheim_core::{Result as SkrifheimResult, SkrifheimError};
use skrifheim_crypto::EncryptionDomain;
use skrifheim_storage::{
    SEGMENT_FOOTER_BYTES, SEGMENT_HEADER_BYTES, SegmentFooter, SegmentHeader,
    WAL_FRAME_HEADER_BYTES, WalFrameHeader, wal_body_crc64,
};

#[cfg(any(target_os = "linux", target_os = "android"))]
const O_NOFOLLOW_FLAG: i32 = 0o400000;

#[cfg(any(target_os = "illumos", target_os = "solaris"))]
const O_NOFOLLOW_FLAG: i32 = 0x20000;

#[cfg(any(
    target_os = "macos",
    target_os = "ios",
    target_os = "freebsd",
    target_os = "openbsd",
    target_os = "netbsd",
    target_os = "dragonfly"
))]
const O_NOFOLLOW_FLAG: i32 = 0x0100;

#[cfg(all(
    unix,
    not(any(
        target_os = "linux",
        target_os = "android",
        target_os = "illumos",
        target_os = "solaris",
        target_os = "macos",
        target_os = "ios",
        target_os = "freebsd",
        target_os = "openbsd",
        target_os = "netbsd",
        target_os = "dragonfly"
    ))
))]
compile_error!(
    "O_NOFOLLOW is not defined for this Unix platform; add an explicit constant or disable WAL file I/O for this target"
);

const _: () = assert!(
    usize::BITS >= 32,
    "skrifheim-storage-host requires at least a 32-bit address space"
);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WalAppendOptions {
    sync_on_append: bool,
}

impl WalAppendOptions {
    #[must_use]
    pub const fn new(sync_on_append: bool) -> Self {
        Self { sync_on_append }
    }

    #[must_use]
    pub const fn sync_on_append(self) -> bool {
        self.sync_on_append
    }
}

impl Default for WalAppendOptions {
    fn default() -> Self {
        Self::new(true)
    }
}

#[derive(Debug)]
pub enum WalFileError {
    Io(io::Error),
    InvalidFrame(SkrifheimError),
    BodyLengthMismatch,
    PartialFrame,
}

impl fmt::Display for WalFileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(f, "WAL file I/O failed: {error}"),
            Self::InvalidFrame(error) => write!(f, "WAL frame validation failed: {error}"),
            Self::BodyLengthMismatch => write!(f, "WAL frame body length mismatch"),
            Self::PartialFrame => write!(f, "WAL file ended inside a frame"),
        }
    }
}

impl std::error::Error for WalFileError {}

impl From<io::Error> for WalFileError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<SkrifheimError> for WalFileError {
    fn from(error: SkrifheimError) -> Self {
        Self::InvalidFrame(error)
    }
}

pub struct WalFileWriter {
    file: File,
    options: WalAppendOptions,
}

impl WalFileWriter {
    pub fn open_append(path: impl AsRef<Path>, options: WalAppendOptions) -> Result<Self> {
        let path = path.as_ref();
        let created = !path.try_exists()?;
        let mut open_options = OpenOptions::new();
        open_options.create(true).append(true);
        #[cfg(unix)]
        {
            open_options.custom_flags(O_NOFOLLOW_FLAG);
            open_options.mode(0o600);
        }
        let file = open_options.open(path)?;
        if !file.metadata()?.is_file() {
            return Err(WalFileError::Io(io::Error::other(
                "WAL path must be a regular file",
            )));
        }
        #[cfg(unix)]
        {
            file.set_permissions(Permissions::from_mode(0o600))?;
            if created {
                fsync_parent_dir(path)?;
            }
        }
        Ok(Self { file, options })
    }

    pub fn append_frame(&mut self, header: &WalFrameHeader, encrypted_body: &[u8]) -> Result<()> {
        validate_body_len(header, encrypted_body)?;
        header.validate()?;
        verify_body_crc(header, encrypted_body)?;
        self.file.write_all(&header.encode())?;
        self.file.write_all(encrypted_body)?;
        if self.options.sync_on_append() {
            self.file.sync_all()?;
        }
        Ok(())
    }
}

pub struct WalFileReader {
    file: File,
    expected_domain: EncryptionDomain,
}

impl WalFileReader {
    pub fn open(path: impl AsRef<Path>, expected_domain: EncryptionDomain) -> Result<Self> {
        let mut open_options = OpenOptions::new();
        open_options.read(true);
        #[cfg(unix)]
        open_options.custom_flags(O_NOFOLLOW_FLAG);
        let file = open_options.open(path)?;
        if !file.metadata()?.is_file() {
            return Err(WalFileError::Io(io::Error::other(
                "WAL path must be a regular file",
            )));
        }
        Ok(Self {
            file,
            expected_domain,
        })
    }

    pub fn next_frame(&mut self) -> Result<Option<WalFileFrame>> {
        let mut header_bytes = [0_u8; WAL_FRAME_HEADER_BYTES];
        match read_exact_or_clean_eof(&mut self.file, &mut header_bytes)? {
            ReadState::CleanEof => return Ok(None),
            ReadState::Complete => {}
        }
        let header = WalFrameHeader::parse_for_domain(&header_bytes, self.expected_domain)?;
        let body_len = usize::try_from(header.encrypted_body_len()).map_err(|_| {
            WalFileError::Io(io::Error::other("WAL body length exceeds address space"))
        })?;
        let mut encrypted_body = vec![0_u8; body_len];
        self.file
            .read_exact(&mut encrypted_body)
            .map_err(map_partial_read)?;
        verify_body_crc(&header, &encrypted_body)?;
        Ok(Some(WalFileFrame {
            header,
            encrypted_body,
        }))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SegmentWriteOptions {
    sync_on_write: bool,
}

impl SegmentWriteOptions {
    #[must_use]
    pub const fn new(sync_on_write: bool) -> Self {
        Self { sync_on_write }
    }

    #[must_use]
    pub const fn sync_on_write(self) -> bool {
        self.sync_on_write
    }
}

impl Default for SegmentWriteOptions {
    fn default() -> Self {
        Self::new(true)
    }
}

#[derive(Debug)]
pub enum SegmentFileError {
    Io(io::Error),
    InvalidSegment(SkrifheimError),
    BodyLengthMismatch,
    FileLengthMismatch,
    PartialSegment,
    ContentDigestRejected(SkrifheimError),
}

impl fmt::Display for SegmentFileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(f, "segment file I/O failed: {error}"),
            Self::InvalidSegment(error) => write!(f, "segment validation failed: {error}"),
            Self::BodyLengthMismatch => write!(f, "segment body length mismatch"),
            Self::FileLengthMismatch => write!(f, "segment file length mismatch"),
            Self::PartialSegment => write!(f, "segment file ended inside a segment"),
            Self::ContentDigestRejected(error) => {
                write!(f, "segment content digest verification failed: {error}")
            }
        }
    }
}

impl std::error::Error for SegmentFileError {}

impl From<io::Error> for SegmentFileError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<SkrifheimError> for SegmentFileError {
    fn from(error: SkrifheimError) -> Self {
        Self::InvalidSegment(error)
    }
}

/// Verifies that the encrypted segment body matches the content digest carried
/// by the segment header/footer.
///
/// The host crate owns file-system enforcement and cannot choose the database's
/// long-term digest implementation. Readers must therefore inject the admitted
/// digest verifier at the trust boundary instead of silently accepting a header
/// digest as proof of body integrity.
///
/// Implementations must recompute the admitted content digest over
/// `encrypted_body`. Until authenticated encryption, signatures, or keyed
/// manifests are wired in, this API provides structural-corruption detection
/// only; a local attacker with write access can recompute unkeyed CRC/digest
/// metadata.
pub trait SegmentContentVerifier {
    fn verify_content_digest(
        &self,
        header: &SegmentHeader,
        encrypted_body: &[u8],
    ) -> SkrifheimResult<()>;
}

pub struct SegmentFileWriter {
    file: File,
    options: SegmentWriteOptions,
}

impl SegmentFileWriter {
    pub fn create(path: impl AsRef<Path>, options: SegmentWriteOptions) -> SegmentResult<Self> {
        let path = path.as_ref();
        let mut open_options = OpenOptions::new();
        open_options.create_new(true).write(true);
        #[cfg(unix)]
        {
            open_options.custom_flags(O_NOFOLLOW_FLAG);
            open_options.mode(0o600);
        }
        let file = open_options.open(path)?;
        if !file.metadata()?.is_file() {
            return Err(SegmentFileError::Io(io::Error::other(
                "segment path must be a regular file",
            )));
        }
        #[cfg(unix)]
        {
            file.set_permissions(Permissions::from_mode(0o600))?;
            fsync_parent_dir(path)?;
        }
        Ok(Self { file, options })
    }

    pub fn write_segment(
        &mut self,
        header: &SegmentHeader,
        encrypted_body: &[u8],
        verifier: &impl SegmentContentVerifier,
    ) -> SegmentResult<()> {
        validate_segment_body_len(header, encrypted_body)?;
        header.validate()?;
        verify_segment_body_crc(header, encrypted_body)?;
        verifier
            .verify_content_digest(header, encrypted_body)
            .map_err(SegmentFileError::ContentDigestRejected)?;
        let footer = SegmentFooter::from_header(header)?;
        self.file.write_all(&header.encode())?;
        self.file.write_all(encrypted_body)?;
        self.file.write_all(&footer.encode())?;
        if self.options.sync_on_write() {
            self.file.sync_all()?;
        }
        Ok(())
    }
}

pub struct SegmentFileReader {
    file: File,
    expected_domain: EncryptionDomain,
}

impl SegmentFileReader {
    pub fn open(path: impl AsRef<Path>, expected_domain: EncryptionDomain) -> SegmentResult<Self> {
        let mut open_options = OpenOptions::new();
        open_options.read(true);
        #[cfg(unix)]
        open_options.custom_flags(O_NOFOLLOW_FLAG);
        let file = open_options.open(path)?;
        if !file.metadata()?.is_file() {
            return Err(SegmentFileError::Io(io::Error::other(
                "segment path must be a regular file",
            )));
        }
        Ok(Self {
            file,
            expected_domain,
        })
    }

    pub fn read_segment(
        &mut self,
        verifier: &impl SegmentContentVerifier,
    ) -> SegmentResult<SegmentFileSegment> {
        let mut header_bytes = [0_u8; SEGMENT_HEADER_BYTES];
        self.file
            .read_exact(&mut header_bytes)
            .map_err(map_segment_partial_read)?;
        let header = SegmentHeader::parse_for_domain(&header_bytes, self.expected_domain)?;
        verify_segment_file_len(&self.file, &header)?;
        let body_len = usize::try_from(header.body_len()).map_err(|_| {
            SegmentFileError::Io(io::Error::other(
                "segment body length exceeds address space",
            ))
        })?;
        let mut encrypted_body = vec![0_u8; body_len];
        self.file
            .read_exact(&mut encrypted_body)
            .map_err(map_segment_partial_read)?;
        verify_segment_body_crc(&header, &encrypted_body)?;

        let mut footer_bytes = [0_u8; SEGMENT_FOOTER_BYTES];
        self.file
            .read_exact(&mut footer_bytes)
            .map_err(map_segment_partial_read)?;
        let footer = SegmentFooter::parse(&footer_bytes)?;
        footer.validate_against_header(&header)?;
        verifier
            .verify_content_digest(&header, &encrypted_body)
            .map_err(SegmentFileError::ContentDigestRejected)?;

        Ok(SegmentFileSegment {
            header,
            footer,
            encrypted_body,
        })
    }
}

#[derive(Debug)]
pub struct SegmentFileSegment {
    header: SegmentHeader,
    footer: SegmentFooter,
    encrypted_body: Vec<u8>,
}

impl SegmentFileSegment {
    #[must_use]
    pub const fn header(&self) -> &SegmentHeader {
        &self.header
    }

    #[must_use]
    pub const fn footer(&self) -> &SegmentFooter {
        &self.footer
    }

    #[must_use]
    pub fn encrypted_body(&self) -> &[u8] {
        &self.encrypted_body
    }
}

#[derive(Debug)]
pub struct WalFileFrame {
    header: WalFrameHeader,
    encrypted_body: Vec<u8>,
}

impl WalFileFrame {
    #[must_use]
    pub const fn header(&self) -> &WalFrameHeader {
        &self.header
    }

    #[must_use]
    pub fn encrypted_body(&self) -> &[u8] {
        &self.encrypted_body
    }
}

type Result<T> = core::result::Result<T, WalFileError>;
type SegmentResult<T> = core::result::Result<T, SegmentFileError>;

fn validate_body_len(header: &WalFrameHeader, encrypted_body: &[u8]) -> SkrifheimResult<()> {
    if encrypted_body.len() as u64 != header.encrypted_body_len() {
        return Err(SkrifheimError::InvalidWalFrame(
            "WAL frame body length mismatch".into(),
        ));
    }
    Ok(())
}

fn verify_body_crc(header: &WalFrameHeader, encrypted_body: &[u8]) -> SkrifheimResult<()> {
    if wal_body_crc64(encrypted_body) != header.body_crc64() {
        return Err(SkrifheimError::InvalidWalFrame(
            "WAL frame body CRC mismatch".into(),
        ));
    }
    Ok(())
}

fn validate_segment_body_len(header: &SegmentHeader, encrypted_body: &[u8]) -> SegmentResult<()> {
    if encrypted_body.len() as u64 != header.body_len() {
        return Err(SegmentFileError::BodyLengthMismatch);
    }
    Ok(())
}

fn verify_segment_body_crc(header: &SegmentHeader, encrypted_body: &[u8]) -> SegmentResult<()> {
    // CRC64 catches accidental corruption and malformed files only. It is not a
    // keyed integrity mechanism and must be paired with AEAD/signatures or a
    // keyed manifest before stored segments are trusted against local tampering.
    match header.body_crc64() {
        skrifheim_storage::BodyChecksum::Present(expected)
            if wal_body_crc64(encrypted_body) == expected =>
        {
            Ok(())
        }
        _ => Err(SegmentFileError::InvalidSegment(
            SkrifheimError::InvalidStorageHeader("segment body CRC mismatch".into()),
        )),
    }
}

fn verify_segment_file_len(file: &File, header: &SegmentHeader) -> SegmentResult<()> {
    let expected_len = (SEGMENT_HEADER_BYTES as u64)
        .checked_add(header.body_len())
        .and_then(|value| value.checked_add(SEGMENT_FOOTER_BYTES as u64))
        .ok_or_else(|| {
            SegmentFileError::Io(io::Error::other("segment file length overflows u64"))
        })?;
    if file.metadata()?.len() != expected_len {
        return Err(SegmentFileError::FileLengthMismatch);
    }
    Ok(())
}

enum ReadState {
    CleanEof,
    Complete,
}

fn read_exact_or_clean_eof(file: &mut File, mut buf: &mut [u8]) -> Result<ReadState> {
    let mut read_any = false;
    while !buf.is_empty() {
        match file.read(buf) {
            Ok(0) if read_any => return Err(WalFileError::PartialFrame),
            Ok(0) => return Ok(ReadState::CleanEof),
            Ok(read) => {
                read_any = true;
                let remaining = buf;
                buf = &mut remaining[read..];
            }
            Err(error) if error.kind() == io::ErrorKind::Interrupted => {}
            Err(error) => return Err(WalFileError::Io(error)),
        }
    }
    Ok(ReadState::Complete)
}

fn map_partial_read(error: io::Error) -> WalFileError {
    if error.kind() == io::ErrorKind::UnexpectedEof {
        WalFileError::PartialFrame
    } else {
        WalFileError::Io(error)
    }
}

fn map_segment_partial_read(error: io::Error) -> SegmentFileError {
    if error.kind() == io::ErrorKind::UnexpectedEof {
        SegmentFileError::PartialSegment
    } else {
        SegmentFileError::Io(error)
    }
}

#[cfg(unix)]
fn fsync_parent_dir(path: &Path) -> io::Result<()> {
    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        File::open(parent)?.sync_all()?;
    }
    Ok(())
}

#[cfg(test)]
mod tests;
