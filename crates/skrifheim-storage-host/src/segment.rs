use std::{
    collections::hash_map::RandomState,
    fmt,
    fs::{self, File, OpenOptions},
    hash::BuildHasher,
    io::{self, Read, Write},
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

#[cfg(unix)]
use std::{
    fs::Permissions,
    os::unix::fs::{MetadataExt, OpenOptionsExt, PermissionsExt},
};

use skrifheim_core::{Result as SkrifheimResult, SkrifheimError};
use skrifheim_crypto::EncryptionDomain;
use skrifheim_storage::{
    BodyChecksum, SEGMENT_FOOTER_BYTES, SEGMENT_HEADER_BYTES, SegmentFooter, SegmentHeader,
    wal_body_crc64,
};

use crate::common::{add_no_follow, fsync_parent_dir, require_explicit_parent};

pub const MAX_IN_MEMORY_SEGMENT_BYTES: u64 = 16 * 1024 * 1024;
static STAGED_SEGMENT_COUNTER: AtomicU64 = AtomicU64::new(1);
const STAGED_SEGMENT_CREATE_ATTEMPTS: usize = 8;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SegmentWriteOptions {
    sync_on_write: bool,
}

impl SegmentWriteOptions {
    /// Controls the optional pre-publication file sync after writing bytes.
    ///
    /// Immutable segment publication always performs the durability syncs
    /// required to make a completed target visible safely. Passing `false`
    /// skips only the extra sync before the mandatory publication sync.
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
    ResourceLimitExceeded,
    ContentDigestRejected(SkrifheimError),
    PublishedDurabilityUnknown { source: io::Error },
}

impl fmt::Display for SegmentFileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(f, "segment file I/O failed: {error}"),
            Self::InvalidSegment(error) => write!(f, "segment validation failed: {error}"),
            Self::BodyLengthMismatch => write!(f, "segment body length mismatch"),
            Self::FileLengthMismatch => write!(f, "segment file length mismatch"),
            Self::PartialSegment => write!(f, "segment file ended inside a segment"),
            Self::ResourceLimitExceeded => write!(f, "segment resource limit exceeded"),
            Self::ContentDigestRejected(error) => {
                write!(f, "segment content digest verification failed: {error}")
            }
            Self::PublishedDurabilityUnknown { .. } => {
                write!(f, "segment target is visible but durability is uncertain")
            }
        }
    }
}

impl std::error::Error for SegmentFileError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(error) | Self::PublishedDurabilityUnknown { source: error } => Some(error),
            Self::InvalidSegment(_) | Self::ContentDigestRejected(_) => None,
            Self::BodyLengthMismatch
            | Self::FileLengthMismatch
            | Self::PartialSegment
            | Self::ResourceLimitExceeded => None,
        }
    }
}

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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SegmentPublishOutcome {
    Published,
    PublishedWithCleanupPending,
}

pub struct SegmentFileWriter {
    file: File,
    staged_path: PathBuf,
    target_path: PathBuf,
    options: SegmentWriteOptions,
    published: bool,
}

impl SegmentFileWriter {
    pub fn create(path: impl AsRef<Path>, options: SegmentWriteOptions) -> SegmentResult<Self> {
        let path = path.as_ref();
        let _parent = require_explicit_parent(path)?;
        if path.try_exists()? {
            return Err(SegmentFileError::Io(io::Error::new(
                io::ErrorKind::AlreadyExists,
                "segment path already exists",
            )));
        }
        let mut open_options = OpenOptions::new();
        open_options.create_new(true).write(true);
        add_no_follow(&mut open_options);
        #[cfg(unix)]
        open_options.mode(0o600);
        let (file, staged_path) = create_staged_segment_file(path, &open_options)?;
        if !file.metadata()?.is_file() {
            return Err(SegmentFileError::Io(io::Error::other(
                "segment path must be a regular file",
            )));
        }
        #[cfg(unix)]
        {
            file.set_permissions(Permissions::from_mode(0o600))?;
        }
        Ok(Self {
            file,
            staged_path,
            target_path: path.to_path_buf(),
            options,
            published: false,
        })
    }

    pub fn write_segment(
        mut self,
        header: &SegmentHeader,
        encrypted_body: &[u8],
        verifier: &impl SegmentContentVerifier,
    ) -> SegmentResult<SegmentPublishOutcome> {
        header.validate()?;
        enforce_host_body_limit(header)?;
        validate_segment_body_len(header, encrypted_body)?;
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
        self.file.sync_all()?;
        fs::hard_link(&self.staged_path, &self.target_path)?;
        self.published = true;
        fsync_parent_dir(&self.target_path)
            .map_err(|source| SegmentFileError::PublishedDurabilityUnknown { source })?;
        let cleanup_pending = match fs::remove_file(&self.staged_path) {
            Ok(()) => fsync_parent_dir(&self.target_path).is_err(),
            Err(_) => true,
        };
        if cleanup_pending {
            return Ok(SegmentPublishOutcome::PublishedWithCleanupPending);
        }
        Ok(SegmentPublishOutcome::Published)
    }
}

impl Drop for SegmentFileWriter {
    fn drop(&mut self) {
        if !self.published {
            let _ = fs::remove_file(&self.staged_path);
        }
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
        if header.body_len() > MAX_IN_MEMORY_SEGMENT_BYTES {
            return Err(SegmentFileError::ResourceLimitExceeded);
        }
        let body_len = usize::try_from(header.body_len()).map_err(|_| {
            SegmentFileError::Io(io::Error::other(
                "segment body length exceeds address space",
            ))
        })?;
        let mut encrypted_body = Vec::new();
        encrypted_body
            .try_reserve_exact(body_len)
            .map_err(|_| SegmentFileError::ResourceLimitExceeded)?;
        encrypted_body.resize(body_len, 0);
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

/// Removes owned stale segment staging files from a database directory.
///
/// This is a startup or maintenance operation. It must not run concurrently
/// with active segment writers in the same directory.
pub fn cleanup_staged_segments(dir: impl AsRef<Path>) -> SegmentResult<usize> {
    let dir = dir.as_ref();
    let dir_metadata = fs::metadata(dir)?;
    if !dir_metadata.is_dir() {
        return Err(SegmentFileError::Io(io::Error::new(
            io::ErrorKind::InvalidInput,
            "segment staging cleanup path must be a directory",
        )));
    }
    let mut removed = 0_usize;
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let file_name = entry.file_name();
        let Some(file_name) = file_name.to_str() else {
            continue;
        };
        if !is_strict_staged_segment_file_name(file_name) {
            continue;
        }
        let path = entry.path();
        validate_staged_cleanup_candidate(dir_metadata.uid(), &path)?;
        fs::remove_file(&path)?;
        removed += 1;
    }
    if removed != 0 {
        File::open(dir)?.sync_all()?;
    }
    Ok(removed)
}

fn validate_segment_body_len(header: &SegmentHeader, encrypted_body: &[u8]) -> SegmentResult<()> {
    if encrypted_body.len() as u64 != header.body_len() {
        return Err(SegmentFileError::BodyLengthMismatch);
    }
    Ok(())
}

fn enforce_host_body_limit(header: &SegmentHeader) -> SegmentResult<()> {
    if header.body_len() > MAX_IN_MEMORY_SEGMENT_BYTES {
        return Err(SegmentFileError::ResourceLimitExceeded);
    }
    Ok(())
}

fn create_staged_segment_file(
    target: &Path,
    open_options: &OpenOptions,
) -> SegmentResult<(File, PathBuf)> {
    let mut last_error = None;
    for _ in 0..STAGED_SEGMENT_CREATE_ATTEMPTS {
        let staged_path = staged_segment_path(target)?;
        match open_options.open(&staged_path) {
            Ok(file) => return Ok((file, staged_path)),
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {
                last_error = Some(error);
            }
            Err(error) => return Err(SegmentFileError::Io(error)),
        }
    }
    Err(SegmentFileError::Io(last_error.unwrap_or_else(|| {
        io::Error::new(
            io::ErrorKind::AlreadyExists,
            "could not create unique staged segment file",
        )
    })))
}

fn staged_segment_path(path: &Path) -> SegmentResult<PathBuf> {
    let parent = require_explicit_parent(path)?;
    let file_name = path.file_name().ok_or_else(|| {
        SegmentFileError::Io(io::Error::new(
            io::ErrorKind::InvalidInput,
            "segment path must include a file name",
        ))
    })?;
    let unique = STAGED_SEGMENT_COUNTER.fetch_add(1, Ordering::Relaxed);
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| SegmentFileError::Io(io::Error::other(error)))?
        .as_nanos();
    let random_state = RandomState::new();
    let nonce = random_state.hash_one((std::process::id(), unique, now));
    let staged_name = format!(
        ".{}.skrifheim-stage-{nonce:016x}-{}",
        file_name.to_string_lossy(),
        unique
    );
    Ok(parent.join(staged_name))
}

fn is_strict_staged_segment_file_name(file_name: &str) -> bool {
    let Some((prefix, unique)) = file_name.rsplit_once('-') else {
        return false;
    };
    if unique.is_empty() || !unique.bytes().all(|byte| byte.is_ascii_digit()) {
        return false;
    }
    let Some((_target_name, nonce)) = prefix.rsplit_once(".skrifheim-stage-") else {
        return false;
    };
    !nonce.is_empty() && nonce.len() == 16 && nonce.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn validate_staged_cleanup_candidate(owner_uid: u32, path: &Path) -> SegmentResult<()> {
    let symlink_metadata = fs::symlink_metadata(path)?;
    if symlink_metadata.file_type().is_symlink() {
        return Err(SegmentFileError::Io(io::Error::new(
            io::ErrorKind::InvalidInput,
            "segment staging cleanup refuses symlink candidates",
        )));
    }
    if !symlink_metadata.is_file() {
        return Err(SegmentFileError::Io(io::Error::new(
            io::ErrorKind::InvalidInput,
            "segment staging cleanup refuses non-file candidates",
        )));
    }
    if symlink_metadata.uid() != owner_uid {
        return Err(SegmentFileError::Io(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "segment staging cleanup refuses files with unexpected owner",
        )));
    }
    if symlink_metadata.permissions().mode() & 0o777 != 0o600 {
        return Err(SegmentFileError::Io(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "segment staging cleanup refuses files with unexpected permissions",
        )));
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
