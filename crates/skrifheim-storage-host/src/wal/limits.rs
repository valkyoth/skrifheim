use super::{Result, WalFileError};

/// Operational WAL-v1 scan bounds, not canonical format/corruption limits.
pub const WAL_FILE_MAX_BYTES: u64 = 128 * 1024 * 1024;
pub const WAL_FILE_MAX_FRAMES: u64 = 8192;

#[derive(Clone, Copy, Eq, PartialEq)]
pub struct WalFileLimits {
    max_bytes: u64,
    max_frames: u64,
}

impl core::fmt::Debug for WalFileLimits {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("WalFileLimits(<redacted>)")
    }
}

impl WalFileLimits {
    pub fn new(max_bytes: u64, max_frames: u64) -> Result<Self> {
        if max_bytes == 0
            || max_bytes > WAL_FILE_MAX_BYTES
            || max_frames == 0
            || max_frames > WAL_FILE_MAX_FRAMES
        {
            return Err(WalFileError::Io(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "invalid WAL scan limits",
            )));
        }
        Ok(Self {
            max_bytes,
            max_frames,
        })
    }

    pub(super) const fn maximum() -> Self {
        Self {
            max_bytes: WAL_FILE_MAX_BYTES,
            max_frames: WAL_FILE_MAX_FRAMES,
        }
    }

    pub(super) fn require(self, bytes: u64, frames: u64) -> Result<()> {
        if bytes > self.max_bytes || frames > self.max_frames {
            return Err(WalFileError::RotationRequired);
        }
        Ok(())
    }
}

impl Default for WalFileLimits {
    fn default() -> Self {
        Self::maximum()
    }
}
