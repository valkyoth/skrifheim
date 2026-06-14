use super::*;

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
