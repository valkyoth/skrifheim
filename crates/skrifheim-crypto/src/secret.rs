use alloc::vec::Vec;
use core::fmt;

use sanitization::SecretVec;
use skrifheim_core::{Result, SkrifheimError};

pub const SECRET_VALUE_MAX_BYTES: usize = 64 * 1024;

pub struct SecretBytes {
    bytes: SecretVec,
}

impl SecretBytes {
    pub fn from_vec(bytes: Vec<u8>) -> Result<Self> {
        let secret = Self {
            bytes: SecretVec::from_vec(bytes),
        };
        secret.validate()
    }

    pub fn from_slice(bytes: &[u8]) -> Result<Self> {
        if bytes.is_empty() || bytes.len() > SECRET_VALUE_MAX_BYTES {
            return Err(SkrifheimError::InvalidSecretMaterial);
        }
        Ok(Self {
            bytes: SecretVec::from_slice(bytes),
        })
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.bytes.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.bytes.is_empty()
    }

    pub fn with_secret<R>(&self, inspect: impl FnOnce(&[u8]) -> R) -> R {
        self.bytes.with_secret(inspect)
    }

    pub fn try_with_secret<R, E>(
        &self,
        inspect: impl FnOnce(&[u8]) -> core::result::Result<R, E>,
    ) -> core::result::Result<R, E> {
        self.bytes.with_secret(inspect)
    }

    pub fn clear_secret(&mut self) {
        self.bytes.clear_secret();
    }

    pub fn into_cleared(self) {
        self.bytes.into_cleared();
    }

    fn validate(self) -> Result<Self> {
        let len = self.bytes.len();
        let capacity = self.bytes.capacity();
        if len == 0 || len > SECRET_VALUE_MAX_BYTES || capacity > SECRET_VALUE_MAX_BYTES {
            return Err(SkrifheimError::InvalidSecretMaterial);
        }
        Ok(self)
    }
}

impl fmt::Debug for SecretBytes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SecretBytes")
            .field("bytes", &"<redacted>")
            .field("contents", &"<redacted>")
            .finish()
    }
}
