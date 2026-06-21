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
use skrifheim_storage::{WAL_FRAME_HEADER_BYTES, WalFrameHeader};

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
        let mut open_options = OpenOptions::new();
        open_options.create(true).append(true);
        #[cfg(unix)]
        open_options.mode(0o600);
        let file = open_options.open(path)?;
        #[cfg(unix)]
        file.set_permissions(Permissions::from_mode(0o600))?;
        Ok(Self { file, options })
    }

    pub fn append_frame(&mut self, header: &WalFrameHeader, encrypted_body: &[u8]) -> Result<()> {
        validate_body_len(header, encrypted_body)?;
        header.validate()?;
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
        let file = OpenOptions::new().read(true).open(path)?;
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
        let mut encrypted_body = vec![0_u8; header.encrypted_body_len() as usize];
        self.file
            .read_exact(&mut encrypted_body)
            .map_err(map_partial_read)?;
        Ok(Some(WalFileFrame {
            header,
            encrypted_body,
        }))
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

fn validate_body_len(header: &WalFrameHeader, encrypted_body: &[u8]) -> SkrifheimResult<()> {
    if encrypted_body.len() as u64 != header.encrypted_body_len() {
        return Err(SkrifheimError::InvalidWalFrame(
            "WAL frame body length mismatch".into(),
        ));
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

#[cfg(test)]
mod tests;
