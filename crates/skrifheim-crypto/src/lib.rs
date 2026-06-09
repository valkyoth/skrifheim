#![no_std]
#![forbid(unsafe_code)]

extern crate alloc;

use alloc::{string::String, vec::Vec};
use skrifheim_core::{Result, SkrifheimError};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AlgorithmId {
    Blake3,
    Ed25519,
    MlDsa65,
    SlhDsaSha2S128s,
    HybridClassicalPq {
        classical: String,
        post_quantum: String,
    },
    Named(String),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct CryptoEpoch(pub u64);

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SignatureEnvelope {
    pub algorithm: AlgorithmId,
    pub epoch: CryptoEpoch,
    pub key_id: String,
    pub signature: Vec<u8>,
}

impl SignatureEnvelope {
    #[must_use]
    pub fn new(
        algorithm: AlgorithmId,
        epoch: CryptoEpoch,
        key_id: impl Into<String>,
        signature: Vec<u8>,
    ) -> Self {
        Self {
            algorithm,
            epoch,
            key_id: key_id.into(),
            signature,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SignatureSet {
    pub signatures: Vec<SignatureEnvelope>,
}

impl SignatureSet {
    pub fn require_non_empty(&self) -> Result<()> {
        if self.signatures.is_empty() {
            return Err(SkrifheimError::EmptySignatureSet);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_signature_set_is_rejected() {
        let signatures = SignatureSet {
            signatures: Vec::new(),
        };
        assert_eq!(
            signatures.require_non_empty(),
            Err(SkrifheimError::EmptySignatureSet)
        );
    }
}
