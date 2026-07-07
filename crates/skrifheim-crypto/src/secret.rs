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

    /// Copies secret bytes from a borrowed slice into clear-on-drop storage.
    ///
    /// The source slice is not cleared. Callers that own a `Vec<u8>` should
    /// prefer [`Self::from_vec`] to avoid leaving a duplicate plaintext
    /// allocation alive outside this wrapper.
    pub fn from_slice(bytes: &[u8]) -> Result<Self> {
        if bytes.is_empty() || bytes.len() > SECRET_VALUE_MAX_BYTES {
            return Err(SkrifheimError::InvalidSecretMaterial);
        }
        let secret = Self {
            bytes: SecretVec::from_slice(bytes),
        };
        secret.validate()
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.bytes.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.bytes.is_empty()
    }

    /// Temporarily exposes the secret to a closure.
    ///
    /// The closure must not copy, return, log, format, cache, or otherwise
    /// retain bytes derived from the provided slice. Any returned value `R`
    /// must be non-secret metadata, a verification result, or an error/status
    /// value. Use this only for operations that consume the slice in-place
    /// while it is borrowed.
    pub fn with_secret<R>(&self, inspect: impl FnOnce(&[u8]) -> R) -> R {
        self.bytes.with_secret(inspect)
    }

    /// Fallible form of [`Self::with_secret`] with the same no-retention rule.
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
