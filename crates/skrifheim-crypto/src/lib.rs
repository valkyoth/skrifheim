#![no_std]
#![forbid(unsafe_code)]

extern crate alloc;

use alloc::{string::String, vec::Vec};
use core::fmt;
use skrifheim_core::{Result, SkrifheimError};

mod digest;
mod domain;
mod key;
mod key_hierarchy;
mod projection;
mod secret;

#[cfg(test)]
mod tests;

pub use digest::{
    ContentDigest, DigestPolicy, DigestStrength, DigestValue, ManifestDigest,
    SHA3_256_DIGEST_BYTES, SHA3_384_DIGEST_BYTES, SHA3_512_DIGEST_BYTES, SHAKE256_256_DIGEST_BYTES,
    SHAKE256_512_DIGEST_BYTES, WorldIdentityDigest,
};
pub use domain::{EncryptionDomain, EncryptionDomainPurpose};
pub use key::{
    CompartmentKeyId, DeploymentKeyId, KeyErasureMetadata, KeyErasureReason, KeyId,
    KeyLifecycleEventSequence, KeyLifecycleState, KeyMetadata, KeyRotationPreflight, KeyScope,
    RegionKeyId, SegmentKeyId,
};
pub use projection::{ProjectionEncryptionPolicy, ProjectionSurface};
pub use secret::{SECRET_VALUE_MAX_BYTES, SecretBytes};

pub const ED25519_SIG_BYTES: usize = 64;
pub const ML_DSA_65_SIG_BYTES: usize = 3293;
pub const SLH_DSA_SHA2_S128S_SIG_BYTES: usize = 7856;
pub const MAX_VARIABLE_SIGNATURE_BYTES: usize = 16 * 1024;
pub const MAX_SIGNATURES_PER_SET: usize = 16;
pub const KEY_ID_MAX_BYTES: usize = 128;
#[cfg(test)]
pub const APPROVED_NAMED_SIGNATURE_ALGORITHMS: &[&str] = &["SKRIFHEIM-TEST-SIG"];
#[cfg(not(test))]
pub const APPROVED_NAMED_SIGNATURE_ALGORITHMS: &[&str] = &[];
pub const APPROVED_HYBRID_SIGNATURE_COMPONENTS: &[&str] = &["ED25519", "ML-DSA-65"];

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

    pub fn validate_signature_context(&self) -> Result<()> {
        match self {
            Self::Blake3 => Err(SkrifheimError::InvalidSignatureEnvelope(
                "algorithm is not valid for signatures",
            )),
            Self::HybridClassicalPq {
                classical,
                post_quantum,
            } => {
                validate_algorithm_name(classical)?;
                validate_algorithm_name(post_quantum)?;
                require_approved_algorithm_name(classical, APPROVED_HYBRID_SIGNATURE_COMPONENTS)?;
                require_approved_algorithm_name(post_quantum, APPROVED_HYBRID_SIGNATURE_COMPONENTS)
            }
            Self::Named(name) => {
                validate_algorithm_name(name)?;
                require_approved_algorithm_name(name, APPROVED_NAMED_SIGNATURE_ALGORITHMS)
            }
            Self::Ed25519 | Self::MlDsa65 | Self::SlhDsaSha2S128s => Ok(()),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct CryptoEpoch(u64);

impl CryptoEpoch {
    #[must_use]
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }
}

#[derive(Clone)]
pub struct SignatureEnvelope {
    algorithm: AlgorithmId,
    epoch: CryptoEpoch,
    key_id: String,
    signature: Vec<u8>,
}

impl fmt::Debug for SignatureEnvelope {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SignatureEnvelope")
            .field("algorithm", &"<redacted>")
            .field("epoch", &"<redacted>")
            .field("key_id", &"<redacted>")
            .field("signature_bytes", &"<redacted>")
            .finish()
    }
}

impl SignatureEnvelope {
    pub fn new(
        algorithm: AlgorithmId,
        epoch: CryptoEpoch,
        key_id: impl Into<String>,
        signature: Vec<u8>,
    ) -> Result<Self> {
        let key_id = key_id.into();
        if key_id.is_empty() || key_id.len() > KEY_ID_MAX_BYTES {
            return Err(SkrifheimError::InvalidSignatureEnvelope(
                "key id length out of range",
            ));
        }
        if !key_id.bytes().all(|byte| {
            byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b':' | b'@')
        }) {
            return Err(SkrifheimError::InvalidSignatureEnvelope(
                "key id contains invalid characters",
            ));
        }
        validate_signature_envelope_parts(&algorithm, &signature)?;
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

    /// Compares signature-envelope structure with ordinary equality.
    ///
    /// This is not constant-time and must not be used for cryptographic
    /// verification or authentication decisions.
    #[must_use]
    pub fn structurally_equal(&self, other: &Self) -> bool {
        self.algorithm == other.algorithm
            && self.epoch == other.epoch
            && self.key_id == other.key_id
            && self.signature == other.signature
    }
}

#[derive(Clone)]
pub struct SignatureSet {
    signatures: Vec<SignatureEnvelope>,
}

impl fmt::Debug for SignatureSet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SignatureSet")
            .field("count", &self.signatures.len())
            .finish()
    }
}

impl SignatureSet {
    pub fn new(signatures: Vec<SignatureEnvelope>) -> Result<Self> {
        if signatures.len() > MAX_SIGNATURES_PER_SET {
            return Err(SkrifheimError::TooManySignatures);
        }
        let set = Self { signatures };
        set.require_non_empty()?;
        Ok(set)
    }

    #[must_use]
    pub fn envelopes(&self) -> &[SignatureEnvelope] {
        &self.signatures
    }

    /// Compares signature-set structure with ordinary equality.
    ///
    /// This is not constant-time and must not be used for cryptographic
    /// verification or authentication decisions.
    #[must_use]
    pub fn structurally_equal(&self, other: &Self) -> bool {
        self.signatures.len() == other.signatures.len()
            && self
                .signatures
                .iter()
                .zip(other.signatures.iter())
                .all(|(left, right)| left.structurally_equal(right))
    }

    pub fn require_non_empty(&self) -> Result<()> {
        if self.signatures.is_empty() {
            return Err(SkrifheimError::EmptySignatureSet);
        }
        if self.signatures.len() > MAX_SIGNATURES_PER_SET {
            return Err(SkrifheimError::TooManySignatures);
        }
        for signature in &self.signatures {
            validate_signature_envelope_parts(&signature.algorithm, &signature.signature)?;
            if signature.key_id.is_empty() || signature.key_id.len() > KEY_ID_MAX_BYTES {
                return Err(SkrifheimError::InvalidSignatureEnvelope(
                    "key id length out of range",
                ));
            }
            if !signature.key_id.bytes().all(|byte| {
                byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b':' | b'@')
            }) {
                return Err(SkrifheimError::InvalidSignatureEnvelope(
                    "key id contains invalid characters",
                ));
            }
        }
        require_unique_signature_signers(&self.signatures)?;
        Ok(())
    }
}

fn validate_signature_envelope_parts(algorithm: &AlgorithmId, signature: &[u8]) -> Result<()> {
    algorithm.validate_signature_context()?;
    if signature.is_empty() {
        return Err(SkrifheimError::InvalidSignatureLength);
    }
    if signature.len() > MAX_VARIABLE_SIGNATURE_BYTES {
        return Err(SkrifheimError::InvalidSignatureLength);
    }
    let expected = match algorithm {
        AlgorithmId::Ed25519 => Some(ED25519_SIG_BYTES),
        AlgorithmId::MlDsa65 => Some(ML_DSA_65_SIG_BYTES),
        AlgorithmId::SlhDsaSha2S128s => Some(SLH_DSA_SHA2_S128S_SIG_BYTES),
        AlgorithmId::HybridClassicalPq {
            classical,
            post_quantum,
        } => {
            let expected_len = component_min_signature_len(classical)?
                .checked_add(component_min_signature_len(post_quantum)?)
                .ok_or(SkrifheimError::InvalidSignatureLength)?;
            if signature.len() != expected_len {
                return Err(SkrifheimError::InvalidSignatureLength);
            }
            None
        }
        AlgorithmId::Named(name) => {
            let expected_len = named_signature_len(name)?;
            if signature.len() != expected_len {
                return Err(SkrifheimError::InvalidSignatureLength);
            }
            None
        }
        AlgorithmId::Blake3 => {
            return Err(SkrifheimError::InvalidSignatureEnvelope(
                "algorithm is not valid for signatures",
            ));
        }
    };
    if let Some(expected) = expected
        && expected != signature.len()
    {
        return Err(SkrifheimError::InvalidSignatureLength);
    }
    Ok(())
}

fn require_unique_signature_signers(signatures: &[SignatureEnvelope]) -> Result<()> {
    let mut index = 0;
    while index < signatures.len() {
        let mut other = index + 1;
        while other < signatures.len() {
            if signatures[index].algorithm == signatures[other].algorithm
                && signatures[index].key_id == signatures[other].key_id
            {
                return Err(SkrifheimError::DuplicateSignatureSigner);
            }
            other += 1;
        }
        index += 1;
    }
    Ok(())
}

fn component_min_signature_len(name: &str) -> Result<usize> {
    match name {
        "ED25519" => Ok(ED25519_SIG_BYTES),
        "ML-DSA-65" => Ok(ML_DSA_65_SIG_BYTES),
        _ => Err(SkrifheimError::InvalidSignatureEnvelope(
            "algorithm name is not approved for signatures",
        )),
    }
}

fn named_signature_len(name: &str) -> Result<usize> {
    match name {
        #[cfg(test)]
        "SKRIFHEIM-TEST-SIG" => Ok(ED25519_SIG_BYTES),
        _ => Err(SkrifheimError::InvalidSignatureEnvelope(
            "algorithm name is not approved for signatures",
        )),
    }
}

fn require_approved_algorithm_name(name: &str, approved: &[&str]) -> Result<()> {
    if approved.contains(&name) {
        Ok(())
    } else {
        Err(SkrifheimError::InvalidSignatureEnvelope(
            "algorithm name is not approved for signatures",
        ))
    }
}

fn validate_algorithm_name(name: &str) -> Result<()> {
    if name.is_empty() || name.len() > 64 {
        return Err(SkrifheimError::InvalidSignatureEnvelope(
            "algorithm name is empty or too long",
        ));
    }
    if !name
        .bytes()
        .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
    {
        return Err(SkrifheimError::InvalidSignatureEnvelope(
            "algorithm name contains invalid characters",
        ));
    }
    Ok(())
}
