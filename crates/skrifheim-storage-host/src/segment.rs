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
    BodyChecksum, SEGMENT_FOOTER_BYTES, SEGMENT_HEADER_BYTES, SegmentFooter, SegmentHeader,
    wal_body_crc64,
};

use crate::common::{add_no_follow, fsync_parent_dir};

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
        add_no_follow(&mut open_options);
        #[cfg(unix)]
        open_options.mode(0o600);
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
        add_no_follow(&mut open_options);
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

pub(crate) type SegmentResult<T> = core::result::Result<T, SegmentFileError>;

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
        BodyChecksum::Present(expected) if wal_body_crc64(encrypted_body) == expected => Ok(()),
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

fn map_segment_partial_read(error: io::Error) -> SegmentFileError {
    if error.kind() == io::ErrorKind::UnexpectedEof {
        SegmentFileError::PartialSegment
    } else {
        SegmentFileError::Io(error)
    }
}
