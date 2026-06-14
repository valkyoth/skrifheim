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
        SignatureEnvelope::new(AlgorithmId::Ed25519, CryptoEpoch(1), "k", Vec::new()),
        Err(SkrifheimError::EmptySignatureSet)
    );
}

#[test]
fn signature_key_ids_are_bounded() {
    assert_eq!(
        SignatureEnvelope::new(AlgorithmId::Ed25519, CryptoEpoch(1), "", vec![1; 64]),
        Err(SkrifheimError::InvalidSignatureEnvelope(
            "key id length out of range"
        ))
    );
    assert_eq!(
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
    );
    assert_eq!(
        SignatureEnvelope::new(
            AlgorithmId::Ed25519,
            CryptoEpoch(1),
            "key with spaces",
            vec![1; 64]
        ),
        Err(SkrifheimError::InvalidSignatureEnvelope(
            "key id contains invalid characters"
        ))
    );
}

#[test]
fn ed25519_signature_length_is_enforced() {
    assert_eq!(
        SignatureEnvelope::new(AlgorithmId::Ed25519, CryptoEpoch(1), "k", vec![1; 63]),
        Err(SkrifheimError::InvalidSignatureLength)
    );
    assert!(SignatureEnvelope::new(AlgorithmId::Ed25519, CryptoEpoch(1), "k", vec![1; 64]).is_ok());
}

#[test]
fn variable_signature_length_is_bounded() {
    assert_eq!(
        SignatureEnvelope::new(
            AlgorithmId::Named(String::from("SKRIFHEIM-TEST-SIG")),
            CryptoEpoch(1),
            "k",
            vec![1; MAX_VARIABLE_SIGNATURE_BYTES + 1],
        ),
        Err(SkrifheimError::InvalidSignatureLength)
    );
    assert!(
        SignatureEnvelope::new(
            AlgorithmId::Named(String::from("SKRIFHEIM-TEST-SIG")),
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
fn unapproved_named_algorithm_is_rejected_in_signature_contexts() {
    assert_eq!(
        SignatureEnvelope::new(
            AlgorithmId::Named(String::from("DEBUG-SKIP")),
            CryptoEpoch(1),
            "k",
            vec![1]
        ),
        Err(SkrifheimError::InvalidSignatureEnvelope(
            "algorithm name is not approved for signatures"
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
fn hybrid_algorithm_components_must_be_approved() {
    assert_eq!(
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
            segment_id,
        },
        CryptoEpoch(1),
    );
    let data = KeyMetadata::new(
        id(KeyId::from_u128(7))?,
        Some(compartment.key_id()),
        KeyScope::Data {
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
fn key_lifecycle_accepts_valid_activation_and_rotation() -> Result<()> {
    let scope = KeyScope::Compartment {
        tenant_id: id(TenantId::from_u128(12))?,
        compartment_id: id(CompartmentKeyId::from_u128(13))?,
    };
    let current = KeyMetadata::new(id(KeyId::from_u128(50))?, None, scope, CryptoEpoch(1))
        .transition(KeyLifecycleState::Active, CryptoEpoch(1))?;
    let rotating = current.transition(KeyLifecycleState::Rotating, CryptoEpoch(2))?;
    assert_eq!(rotating.lifecycle(), KeyLifecycleState::Rotating);

    let candidate = KeyMetadata::new(id(KeyId::from_u128(51))?, None, scope, CryptoEpoch(3));
    let preflight = current.rotation_preflight(&candidate)?;
    assert_eq!(preflight.current_key(), current.key_id());
    assert_eq!(preflight.candidate_key(), candidate.key_id());
    assert_eq!(preflight.from_epoch(), CryptoEpoch(1));
    assert_eq!(preflight.to_epoch(), CryptoEpoch(3));
    Ok(())
}

#[test]
fn key_lifecycle_rejects_invalid_transitions() -> Result<()> {
    let key = KeyMetadata::new(
        id(KeyId::from_u128(60))?,
        None,
        KeyScope::RootTrust,
        CryptoEpoch(2),
    );
    assert_eq!(
        key.transition(KeyLifecycleState::Retired, CryptoEpoch(2)),
        Err(SkrifheimError::InvalidKeyLifecycle)
    );
    assert_eq!(
        key.transition(KeyLifecycleState::Active, CryptoEpoch(1)),
        Err(SkrifheimError::InvalidKeyLifecycle)
    );
    Ok(())
}

#[test]
fn key_rotation_preflight_rejects_wrong_scope_or_epoch() -> Result<()> {
    let tenant_id = id(TenantId::from_u128(12))?;
    let current = KeyMetadata::with_lifecycle(
        id(KeyId::from_u128(70))?,
        None,
        KeyScope::Compartment {
            tenant_id,
            compartment_id: id(CompartmentKeyId::from_u128(13))?,
        },
        CryptoEpoch(5),
        KeyLifecycleState::Active,
    );
    let wrong_scope = KeyMetadata::new(
        id(KeyId::from_u128(71))?,
        None,
        KeyScope::Segment {
            tenant_id,
            segment_id: id(SegmentKeyId::from_u128(14))?,
        },
        CryptoEpoch(6),
    );
    let stale_epoch = KeyMetadata::new(
        id(KeyId::from_u128(72))?,
        None,
        current.scope(),
        CryptoEpoch(5),
    );

    assert_eq!(
        current.rotation_preflight(&wrong_scope),
        Err(SkrifheimError::InvalidKeyLifecycle)
    );
    assert_eq!(
        current.rotation_preflight(&stale_epoch),
        Err(SkrifheimError::InvalidKeyLifecycle)
    );
    Ok(())
}

#[test]
fn compromised_key_can_be_quarantined_destroyed_and_erased() -> Result<()> {
    let key = KeyMetadata::new(
        id(KeyId::from_u128(80))?,
        None,
        KeyScope::RootTrust,
        CryptoEpoch(1),
    )
    .transition(KeyLifecycleState::Active, CryptoEpoch(1))?
    .transition(KeyLifecycleState::Compromised, CryptoEpoch(2))?
    .transition(KeyLifecycleState::Quarantined, CryptoEpoch(2))?
    .transition(KeyLifecycleState::Destroyed, CryptoEpoch(3))?
    .crypto_erase(KeyErasureReason::Compromise)?;

    assert_eq!(key.lifecycle(), KeyLifecycleState::CryptoErased);
    let erasure = key.erasure().ok_or(SkrifheimError::InvalidKeyLifecycle)?;
    assert_eq!(erasure.key_id(), key.key_id());
    assert_eq!(erasure.epoch(), CryptoEpoch(3));
    assert_eq!(erasure.reason(), KeyErasureReason::Compromise);
    Ok(())
}

#[test]
fn active_key_cannot_be_crypto_erased_directly() -> Result<()> {
    let key = KeyMetadata::with_lifecycle(
        id(KeyId::from_u128(90))?,
        None,
        KeyScope::RootTrust,
        CryptoEpoch(1),
        KeyLifecycleState::Active,
    );
    assert_eq!(
        key.crypto_erase(KeyErasureReason::OperatorApproved),
        Err(SkrifheimError::InvalidKeyLifecycle)
    );
    Ok(())
}
