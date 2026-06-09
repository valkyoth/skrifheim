#![no_std]
#![forbid(unsafe_code)]

extern crate alloc;

use alloc::{string::String, vec::Vec};
use skrifheim_core::{Result, SkrifheimError};

pub const ED25519_SIG_BYTES: usize = 64;
pub const ML_DSA_65_SIG_BYTES: usize = 3293;
pub const SLH_DSA_SHA2_S128S_SIG_BYTES: usize = 7856;

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

impl AlgorithmId {
    #[must_use]
    pub const fn is_signing_algorithm(&self) -> bool {
        matches!(
            self,
            Self::Ed25519
                | Self::MlDsa65
                | Self::SlhDsaSha2S128s
                | Self::HybridClassicalPq { .. }
                | Self::Named(_)
        )
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct CryptoEpoch(pub u64);

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SignatureEnvelope {
    algorithm: AlgorithmId,
    epoch: CryptoEpoch,
    key_id: String,
    signature: Vec<u8>,
}

impl SignatureEnvelope {
    pub fn new(
        algorithm: AlgorithmId,
        epoch: CryptoEpoch,
        key_id: impl Into<String>,
        signature: Vec<u8>,
    ) -> Result<Self> {
        let key_id = key_id.into();
        if key_id.is_empty() {
            return Err(SkrifheimError::InvalidSignatureEnvelope(
                "key id must not be empty",
            ));
        }
        validate_signature_length(&algorithm, signature.len())?;
        Ok(Self {
            algorithm,
            epoch,
            key_id,
            signature,
        })
    }

    #[must_use]
    pub const fn algorithm(&self) -> &AlgorithmId {
        &self.algorithm
    }

    #[must_use]
    pub const fn epoch(&self) -> CryptoEpoch {
        self.epoch
    }

    #[must_use]
    pub fn key_id(&self) -> &str {
        &self.key_id
    }

    #[must_use]
    pub fn signature(&self) -> &[u8] {
        &self.signature
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
        for signature in &self.signatures {
            validate_signature_length(&signature.algorithm, signature.signature.len())?;
            if signature.key_id.is_empty() {
                return Err(SkrifheimError::InvalidSignatureEnvelope(
                    "key id must not be empty",
                ));
            }
        }
        Ok(())
    }
}

fn validate_signature_length(algorithm: &AlgorithmId, actual: usize) -> Result<()> {
    if !algorithm.is_signing_algorithm() {
        return Err(SkrifheimError::InvalidSignatureEnvelope(
            "algorithm is not valid for signatures",
        ));
    }
    if actual == 0 {
        return Err(SkrifheimError::EmptySignatureSet);
    }
    let expected = match algorithm {
        AlgorithmId::Ed25519 => Some(ED25519_SIG_BYTES),
        AlgorithmId::MlDsa65 => Some(ML_DSA_65_SIG_BYTES),
        AlgorithmId::SlhDsaSha2S128s => Some(SLH_DSA_SHA2_S128S_SIG_BYTES),
        AlgorithmId::HybridClassicalPq { .. } | AlgorithmId::Named(_) => None,
        AlgorithmId::Blake3 => unreachable!("Blake3 is rejected before length checks"),
    };
    if let Some(expected) = expected
        && expected != actual
    {
        return Err(SkrifheimError::InvalidSignatureLength);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;

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

    #[test]
    fn empty_signature_bytes_are_rejected() {
        assert_eq!(
            SignatureEnvelope::new(
                AlgorithmId::Named(String::from("test")),
                CryptoEpoch(1),
                "k",
                Vec::new()
            ),
            Err(SkrifheimError::EmptySignatureSet)
        );
    }

    #[test]
    fn ed25519_signature_length_is_enforced() {
        assert_eq!(
            SignatureEnvelope::new(AlgorithmId::Ed25519, CryptoEpoch(1), "k", vec![1; 63]),
            Err(SkrifheimError::InvalidSignatureLength)
        );
        assert!(
            SignatureEnvelope::new(AlgorithmId::Ed25519, CryptoEpoch(1), "k", vec![1; 64]).is_ok()
        );
    }

    #[test]
    fn blake3_is_rejected_in_signature_contexts() {
        assert_eq!(
            SignatureEnvelope::new(AlgorithmId::Blake3, CryptoEpoch(1), "k", vec![1; 32]),
            Err(SkrifheimError::InvalidSignatureEnvelope(
                "algorithm is not valid for signatures"
            ))
        );
    }
}
