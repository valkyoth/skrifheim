#![no_std]
#![forbid(unsafe_code)]

extern crate alloc;

use alloc::{string::String, vec::Vec};
use skrifheim_core::{Result, SkrifheimError};

mod key;

pub use key::{
    CompartmentKeyId, DeploymentKeyId, KeyId, KeyMetadata, KeyScope, RegionKeyId, SegmentKeyId,
};

pub const ED25519_SIG_BYTES: usize = 64;
pub const ML_DSA_65_SIG_BYTES: usize = 3293;
pub const SLH_DSA_SHA2_S128S_SIG_BYTES: usize = 7856;
pub const MAX_VARIABLE_SIGNATURE_BYTES: usize = 16 * 1024;
pub const MAX_SIGNATURES_PER_SET: usize = 16;

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
                validate_algorithm_name(post_quantum)
            }
            Self::Named(name) => validate_algorithm_name(name),
            Self::Ed25519 | Self::MlDsa65 | Self::SlhDsaSha2S128s => Ok(()),
        }
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
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SignatureSet {
    signatures: Vec<SignatureEnvelope>,
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

    pub fn require_non_empty(&self) -> Result<()> {
        if self.signatures.is_empty() {
            return Err(SkrifheimError::EmptySignatureSet);
        }
        if self.signatures.len() > MAX_SIGNATURES_PER_SET {
            return Err(SkrifheimError::TooManySignatures);
        }
        for signature in &self.signatures {
            validate_signature_envelope_parts(&signature.algorithm, &signature.signature)?;
            if signature.key_id.is_empty() {
                return Err(SkrifheimError::InvalidSignatureEnvelope(
                    "key id must not be empty",
                ));
            }
        }
        Ok(())
    }
}

fn validate_signature_envelope_parts(algorithm: &AlgorithmId, signature: &[u8]) -> Result<()> {
    algorithm.validate_signature_context()?;
    if signature.is_empty() {
        return Err(SkrifheimError::EmptySignatureSet);
    }
    let expected = match algorithm {
        AlgorithmId::Ed25519 => Some(ED25519_SIG_BYTES),
        AlgorithmId::MlDsa65 => Some(ML_DSA_65_SIG_BYTES),
        AlgorithmId::SlhDsaSha2S128s => Some(SLH_DSA_SHA2_S128S_SIG_BYTES),
        AlgorithmId::HybridClassicalPq { .. } | AlgorithmId::Named(_) => {
            if signature.len() > MAX_VARIABLE_SIGNATURE_BYTES {
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

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;
    use skrifheim_core::{Result, TenantId};

    fn id<T>(id: Option<T>) -> Result<T> {
        id.ok_or(SkrifheimError::InvalidIdentifier)
    }

    fn ed25519_envelope(index: usize) -> Result<SignatureEnvelope> {
        SignatureEnvelope::new(
            AlgorithmId::Ed25519,
            CryptoEpoch(1),
            alloc::format!("key-{index}"),
            vec![1; ED25519_SIG_BYTES],
        )
    }

    #[test]
    fn empty_signature_set_is_rejected() {
        assert_eq!(
            SignatureSet::new(Vec::new()),
            Err(SkrifheimError::EmptySignatureSet)
        );
    }

    #[test]
    fn signature_set_count_is_bounded() -> Result<()> {
        let mut signatures = Vec::new();
        for index in 0..=MAX_SIGNATURES_PER_SET {
            signatures.push(ed25519_envelope(index)?);
        }
        assert_eq!(
            SignatureSet::new(signatures),
            Err(SkrifheimError::TooManySignatures)
        );
        Ok(())
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
    fn variable_signature_length_is_bounded() {
        assert_eq!(
            SignatureEnvelope::new(
                AlgorithmId::Named(String::from("TEST-SIG")),
                CryptoEpoch(1),
                "k",
                vec![1; MAX_VARIABLE_SIGNATURE_BYTES + 1],
            ),
            Err(SkrifheimError::InvalidSignatureLength)
        );
        assert!(
            SignatureEnvelope::new(
                AlgorithmId::Named(String::from("TEST-SIG")),
                CryptoEpoch(1),
                "k",
                vec![1; MAX_VARIABLE_SIGNATURE_BYTES],
            )
            .is_ok()
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

    #[test]
    fn empty_named_algorithm_is_rejected_in_signature_contexts() {
        assert_eq!(
            SignatureEnvelope::new(
                AlgorithmId::Named(String::new()),
                CryptoEpoch(1),
                "k",
                vec![1]
            ),
            Err(SkrifheimError::InvalidSignatureEnvelope(
                "algorithm name is empty or too long"
            ))
        );
    }

    #[test]
    fn hybrid_algorithm_names_are_validated() {
        assert_eq!(
            SignatureEnvelope::new(
                AlgorithmId::HybridClassicalPq {
                    classical: String::from(""),
                    post_quantum: String::from("ML-DSA-65"),
                },
                CryptoEpoch(1),
                "k",
                vec![1]
            ),
            Err(SkrifheimError::InvalidSignatureEnvelope(
                "algorithm name is empty or too long"
            ))
        );
    }

    #[test]
    fn key_hierarchy_accepts_valid_edges() -> Result<()> {
        let root = KeyMetadata::new(
            id(KeyId::from_u128(1))?,
            None,
            KeyScope::RootTrust,
            CryptoEpoch(1),
        );
        let deployment_id = id(DeploymentKeyId::from_u128(10))?;
        let region_id = id(RegionKeyId::from_u128(11))?;
        let tenant_id = id(TenantId::from_u128(12))?;
        let compartment_id = id(CompartmentKeyId::from_u128(13))?;
        let segment_id = id(SegmentKeyId::from_u128(14))?;
        let deployment = KeyMetadata::new(
            id(KeyId::from_u128(2))?,
            Some(root.key_id()),
            KeyScope::Deployment { deployment_id },
            CryptoEpoch(1),
        );
        let region = KeyMetadata::new(
            id(KeyId::from_u128(3))?,
            Some(deployment.key_id()),
            KeyScope::Region {
                deployment_id,
                region_id,
            },
            CryptoEpoch(1),
        );
        let tenant = KeyMetadata::new(
            id(KeyId::from_u128(4))?,
            Some(region.key_id()),
            KeyScope::Tenant { tenant_id },
            CryptoEpoch(1),
        );
        let compartment = KeyMetadata::new(
            id(KeyId::from_u128(5))?,
            Some(tenant.key_id()),
            KeyScope::Compartment {
                tenant_id,
                compartment_id,
            },
            CryptoEpoch(1),
        );
        let segment = KeyMetadata::new(
            id(KeyId::from_u128(6))?,
            Some(compartment.key_id()),
            KeyScope::Segment {
                tenant_id,
                segment_id,
            },
            CryptoEpoch(1),
        );

        assert_eq!(root.validate_parent(None), Ok(()));
        assert_eq!(deployment.validate_parent(Some(&root)), Ok(()));
        assert_eq!(region.validate_parent(Some(&deployment)), Ok(()));
        assert_eq!(tenant.validate_parent(Some(&region)), Ok(()));
        assert_eq!(compartment.validate_parent(Some(&tenant)), Ok(()));
        assert_eq!(segment.validate_parent(Some(&compartment)), Ok(()));
        Ok(())
    }

    #[test]
    fn key_hierarchy_rejects_database_wide_shortcut() -> Result<()> {
        let tenant = KeyMetadata::new(
            id(KeyId::from_u128(4))?,
            None,
            KeyScope::Tenant {
                tenant_id: id(TenantId::from_u128(12))?,
            },
            CryptoEpoch(1),
        );
        assert_eq!(
            tenant.validate_parent(None),
            Err(SkrifheimError::InvalidKeyHierarchy)
        );
        Ok(())
    }

    #[test]
    fn key_hierarchy_rejects_invalid_parent_edges() -> Result<()> {
        let root = KeyMetadata::new(
            id(KeyId::from_u128(1))?,
            None,
            KeyScope::RootTrust,
            CryptoEpoch(1),
        );
        let tenant = KeyMetadata::new(
            id(KeyId::from_u128(4))?,
            Some(root.key_id()),
            KeyScope::Tenant {
                tenant_id: id(TenantId::from_u128(12))?,
            },
            CryptoEpoch(1),
        );
        assert_eq!(
            tenant.validate_parent(Some(&root)),
            Err(SkrifheimError::InvalidKeyHierarchy)
        );
        Ok(())
    }
}
