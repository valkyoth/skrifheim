use super::*;
use alloc::vec;
use skrifheim_core::{Result, TenantId};

mod key_lifecycle;

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
    assert!(matches!(
        SignatureSet::new(Vec::new()),
        Err(SkrifheimError::EmptySignatureSet)
    ));
}

#[test]
fn signature_set_count_is_bounded() -> Result<()> {
    let mut signatures = Vec::new();
    for index in 0..=MAX_SIGNATURES_PER_SET {
        signatures.push(ed25519_envelope(index)?);
    }
    assert!(matches!(
        SignatureSet::new(signatures),
        Err(SkrifheimError::TooManySignatures)
    ));
    Ok(())
}

#[test]
fn debug_redacts_signature_bytes_and_set_contents() -> Result<()> {
    let envelope = SignatureEnvelope::new(
        AlgorithmId::Ed25519,
        CryptoEpoch(1),
        "debug-key",
        vec![255; ED25519_SIG_BYTES],
    )?;
    let envelope_debug = alloc::format!("{envelope:?}");
    assert!(envelope_debug.contains("signature_bytes"));
    assert!(envelope_debug.contains("64"));
    assert!(!envelope_debug.contains("[255"));

    let set = SignatureSet::new(vec![envelope])?;
    let set_debug = alloc::format!("{set:?}");
    assert!(set_debug.contains("count"));
    assert!(!set_debug.contains("debug-key"));
    assert!(!set_debug.contains("[255"));
    Ok(())
}

#[test]
fn empty_signature_bytes_are_rejected() {
    assert!(matches!(
        SignatureEnvelope::new(AlgorithmId::Ed25519, CryptoEpoch(1), "k", Vec::new()),
        Err(SkrifheimError::EmptySignatureSet)
    ));
}

#[test]
fn signature_key_ids_are_bounded() {
    assert!(matches!(
        SignatureEnvelope::new(AlgorithmId::Ed25519, CryptoEpoch(1), "", vec![1; 64]),
        Err(SkrifheimError::InvalidSignatureEnvelope(
            "key id length out of range"
        ))
    ));
    assert!(matches!(
        SignatureEnvelope::new(
            AlgorithmId::Ed25519,
            CryptoEpoch(1),
            alloc::string::String::from_utf8(vec![b'k'; KEY_ID_MAX_BYTES + 1])
                .unwrap_or_else(|_| alloc::string::String::new()),
            vec![1; 64]
        ),
        Err(SkrifheimError::InvalidSignatureEnvelope(
            "key id length out of range"
        ))
    ));
    assert!(matches!(
        SignatureEnvelope::new(
            AlgorithmId::Ed25519,
            CryptoEpoch(1),
            "key with spaces",
            vec![1; 64]
        ),
        Err(SkrifheimError::InvalidSignatureEnvelope(
            "key id contains invalid characters"
        ))
    ));
}

#[test]
fn ed25519_signature_length_is_enforced() {
    assert!(matches!(
        SignatureEnvelope::new(AlgorithmId::Ed25519, CryptoEpoch(1), "k", vec![1; 63]),
        Err(SkrifheimError::InvalidSignatureLength)
    ));
    assert!(SignatureEnvelope::new(AlgorithmId::Ed25519, CryptoEpoch(1), "k", vec![1; 64]).is_ok());
}

#[test]
fn variable_signature_length_is_bounded() {
    assert!(matches!(
        SignatureEnvelope::new(
            AlgorithmId::Named(String::from("SKRIFHEIM-TEST-SIG")),
            CryptoEpoch(1),
            "k",
            vec![1; ED25519_SIG_BYTES - 1],
        ),
        Err(SkrifheimError::InvalidSignatureLength)
    ));
    assert!(matches!(
        SignatureEnvelope::new(
            AlgorithmId::Named(String::from("SKRIFHEIM-TEST-SIG")),
            CryptoEpoch(1),
            "k",
            vec![1; MAX_VARIABLE_SIGNATURE_BYTES + 1],
        ),
        Err(SkrifheimError::InvalidSignatureLength)
    ));
    assert!(matches!(
        SignatureEnvelope::new(
            AlgorithmId::Named(String::from("SKRIFHEIM-TEST-SIG")),
            CryptoEpoch(1),
            "k",
            vec![1; ED25519_SIG_BYTES + 1],
        ),
        Err(SkrifheimError::InvalidSignatureLength)
    ));
    assert!(
        SignatureEnvelope::new(
            AlgorithmId::Named(String::from("SKRIFHEIM-TEST-SIG")),
            CryptoEpoch(1),
            "k",
            vec![1; ED25519_SIG_BYTES],
        )
        .is_ok()
    );
}

#[test]
fn hybrid_signature_length_requires_component_minimums() {
    let algorithm = AlgorithmId::HybridClassicalPq {
        classical: String::from("ED25519"),
        post_quantum: String::from("ML-DSA-65"),
    };
    let min_len = ED25519_SIG_BYTES + ML_DSA_65_SIG_BYTES;

    assert!(matches!(
        SignatureEnvelope::new(algorithm.clone(), CryptoEpoch(1), "k", vec![1; min_len - 1],),
        Err(SkrifheimError::InvalidSignatureLength)
    ));
    assert!(matches!(
        SignatureEnvelope::new(algorithm.clone(), CryptoEpoch(1), "k", vec![1; min_len + 1],),
        Err(SkrifheimError::InvalidSignatureLength)
    ));
    assert!(SignatureEnvelope::new(algorithm, CryptoEpoch(1), "k", vec![1; min_len],).is_ok());
}

#[test]
fn blake3_is_rejected_in_signature_contexts() {
    assert!(matches!(
        SignatureEnvelope::new(AlgorithmId::Blake3, CryptoEpoch(1), "k", vec![1; 32]),
        Err(SkrifheimError::InvalidSignatureEnvelope(
            "algorithm is not valid for signatures"
        ))
    ));
}

#[test]
fn empty_named_algorithm_is_rejected_in_signature_contexts() {
    assert!(matches!(
        SignatureEnvelope::new(
            AlgorithmId::Named(String::new()),
            CryptoEpoch(1),
            "k",
            vec![1]
        ),
        Err(SkrifheimError::InvalidSignatureEnvelope(
            "algorithm name is empty or too long"
        ))
    ));
}

#[test]
fn unapproved_named_algorithm_is_rejected_in_signature_contexts() {
    assert!(matches!(
        SignatureEnvelope::new(
            AlgorithmId::Named(String::from("DEBUG-SKIP")),
            CryptoEpoch(1),
            "k",
            vec![1]
        ),
        Err(SkrifheimError::InvalidSignatureEnvelope(
            "algorithm name is not approved for signatures"
        ))
    ));
}

#[test]
fn hybrid_algorithm_names_are_validated() {
    assert!(matches!(
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
    ));
}

#[test]
fn hybrid_algorithm_components_must_be_approved() {
    assert!(matches!(
        SignatureEnvelope::new(
            AlgorithmId::HybridClassicalPq {
                classical: String::from("ED25519"),
                post_quantum: String::from("DEBUG-SKIP"),
            },
            CryptoEpoch(1),
            "k",
            vec![1]
        ),
        Err(SkrifheimError::InvalidSignatureEnvelope(
            "algorithm name is not approved for signatures"
        ))
    ));
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
        KeyScope::Tenant {
            deployment_id,
            region_id,
            tenant_id,
        },
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
            compartment_id,
            segment_id,
        },
        CryptoEpoch(1),
    );
    let data = KeyMetadata::new(
        id(KeyId::from_u128(7))?,
        Some(compartment.key_id()),
        KeyScope::Data {
            tenant_id,
            compartment_id,
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
    assert_eq!(data.validate_parent(Some(&compartment)), Ok(()));
    Ok(())
}

#[test]
fn key_hierarchy_rejects_database_wide_shortcut() -> Result<()> {
    let tenant = KeyMetadata::new(
        id(KeyId::from_u128(4))?,
        None,
        KeyScope::Tenant {
            deployment_id: id(DeploymentKeyId::from_u128(10))?,
            region_id: id(RegionKeyId::from_u128(11))?,
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
            deployment_id: id(DeploymentKeyId::from_u128(10))?,
            region_id: id(RegionKeyId::from_u128(11))?,
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

#[test]
fn key_hierarchy_rejects_unsafe_parent_lifecycle() -> Result<()> {
    let root = KeyMetadata::with_lifecycle(
        id(KeyId::from_u128(1))?,
        None,
        KeyScope::RootTrust,
        CryptoEpoch(1),
        KeyLifecycleState::Compromised,
    );
    let deployment = KeyMetadata::new(
        id(KeyId::from_u128(2))?,
        Some(root.key_id()),
        KeyScope::Deployment {
            deployment_id: id(DeploymentKeyId::from_u128(10))?,
        },
        CryptoEpoch(1),
    );

    assert_eq!(
        deployment.validate_parent(Some(&root)),
        Err(SkrifheimError::InvalidKeyHierarchy)
    );
    Ok(())
}

#[test]
fn key_hierarchy_rejects_cross_region_tenant_metadata() -> Result<()> {
    let deployment_id = id(DeploymentKeyId::from_u128(10))?;
    let region_id = id(RegionKeyId::from_u128(11))?;
    let other_region_id = id(RegionKeyId::from_u128(99))?;
    let region = KeyMetadata::new(
        id(KeyId::from_u128(3))?,
        Some(id(KeyId::from_u128(2))?),
        KeyScope::Region {
            deployment_id,
            region_id,
        },
        CryptoEpoch(1),
    );
    let tenant = KeyMetadata::new(
        id(KeyId::from_u128(4))?,
        Some(region.key_id()),
        KeyScope::Tenant {
            deployment_id,
            region_id: other_region_id,
            tenant_id: id(TenantId::from_u128(12))?,
        },
        CryptoEpoch(1),
    );
    assert_eq!(
        tenant.validate_parent(Some(&region)),
        Err(SkrifheimError::InvalidKeyHierarchy)
    );
    Ok(())
}

#[test]
fn key_hierarchy_rejects_data_under_non_compartment_parent() -> Result<()> {
    let tenant_id = id(TenantId::from_u128(12))?;
    let tenant = KeyMetadata::new(
        id(KeyId::from_u128(4))?,
        Some(id(KeyId::from_u128(3))?),
        KeyScope::Tenant {
            deployment_id: id(DeploymentKeyId::from_u128(10))?,
            region_id: id(RegionKeyId::from_u128(11))?,
            tenant_id,
        },
        CryptoEpoch(1),
    );
    let data = KeyMetadata::new(
        id(KeyId::from_u128(7))?,
        Some(tenant.key_id()),
        KeyScope::Data {
            tenant_id,
            compartment_id: id(CompartmentKeyId::from_u128(13))?,
            segment_id: id(SegmentKeyId::from_u128(14))?,
        },
        CryptoEpoch(1),
    );
    assert_eq!(
        data.validate_parent(Some(&tenant)),
        Err(SkrifheimError::InvalidKeyHierarchy)
    );
    Ok(())
}

#[test]
fn key_hierarchy_rejects_cross_compartment_segment_metadata() -> Result<()> {
    let tenant_id = id(TenantId::from_u128(12))?;
    let parent_compartment = KeyMetadata::new(
        id(KeyId::from_u128(5))?,
        Some(id(KeyId::from_u128(4))?),
        KeyScope::Compartment {
            tenant_id,
            compartment_id: id(CompartmentKeyId::from_u128(13))?,
        },
        CryptoEpoch(1),
    );
    let segment = KeyMetadata::new(
        id(KeyId::from_u128(6))?,
        Some(parent_compartment.key_id()),
        KeyScope::Segment {
            tenant_id,
            compartment_id: id(CompartmentKeyId::from_u128(99))?,
            segment_id: id(SegmentKeyId::from_u128(14))?,
        },
        CryptoEpoch(1),
    );

    assert_eq!(
        segment.validate_parent(Some(&parent_compartment)),
        Err(SkrifheimError::InvalidKeyHierarchy)
    );
    Ok(())
}
