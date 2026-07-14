use super::*;

#[test]
fn key_lifecycle_accepts_valid_activation_and_rotation() -> Result<()> {
    let scope = KeyScope::Compartment {
        tenant_id: id(TenantId::from_u128(12))?,
        compartment_id: id(CompartmentKeyId::from_u128(13))?,
    };
    let current = KeyMetadata::new(id(KeyId::from_u128(50))?, None, scope, CryptoEpoch::new(1))
        .transition(KeyLifecycleState::Active, CryptoEpoch::new(2))?;
    let rotating = current.transition(KeyLifecycleState::Rotating, CryptoEpoch::new(3))?;
    assert_eq!(rotating.lifecycle(), KeyLifecycleState::Rotating);
    assert_eq!(rotating.lifecycle_event_sequence().get(), 3);

    let candidate = KeyMetadata::new(id(KeyId::from_u128(51))?, None, scope, CryptoEpoch::new(4));
    let preflight = current.rotation_preflight(&candidate)?;
    assert_eq!(preflight.current_key(), current.key_id());
    assert_eq!(preflight.candidate_key(), candidate.key_id());
    assert_eq!(preflight.from_epoch(), CryptoEpoch::new(2));
    assert_eq!(preflight.to_epoch(), CryptoEpoch::new(4));
    Ok(())
}

#[test]
fn key_lifecycle_rejects_invalid_transitions_and_epoch_shape() -> Result<()> {
    let key = KeyMetadata::new(
        id(KeyId::from_u128(60))?,
        None,
        KeyScope::RootTrust,
        CryptoEpoch::new(2),
    );
    assert_eq!(
        key.transition(KeyLifecycleState::Retired, CryptoEpoch::new(2)),
        Err(SkrifheimError::InvalidKeyLifecycle)
    );
    assert_eq!(
        key.transition(KeyLifecycleState::Active, CryptoEpoch::new(1)),
        Err(SkrifheimError::InvalidKeyLifecycle)
    );
    assert_eq!(
        key.transition(KeyLifecycleState::Active, CryptoEpoch::new(2)),
        Err(SkrifheimError::InvalidKeyLifecycle)
    );

    let active = key.transition(KeyLifecycleState::Active, CryptoEpoch::new(3))?;
    assert_eq!(
        active.transition(KeyLifecycleState::Compromised, CryptoEpoch::new(4)),
        Err(SkrifheimError::InvalidKeyLifecycle)
    );
    assert_eq!(
        active.transition(KeyLifecycleState::Rotating, CryptoEpoch::new(3)),
        Err(SkrifheimError::InvalidKeyLifecycle)
    );
    Ok(())
}

#[test]
fn rotating_key_cannot_return_to_active() -> Result<()> {
    let key = KeyMetadata::with_lifecycle_for_test(
        id(KeyId::from_u128(64))?,
        None,
        KeyScope::RootTrust,
        CryptoEpoch::new(3),
        KeyLifecycleState::Rotating,
    );

    assert_eq!(
        key.transition(KeyLifecycleState::Active, CryptoEpoch::new(4)),
        Err(SkrifheimError::InvalidKeyLifecycle)
    );
    Ok(())
}

#[test]
fn compromise_can_be_declared_from_non_active_material_states() -> Result<()> {
    let created = KeyMetadata::new(
        id(KeyId::from_u128(61))?,
        None,
        KeyScope::RootTrust,
        CryptoEpoch::new(1),
    );
    assert_eq!(
        created
            .transition(KeyLifecycleState::Compromised, CryptoEpoch::new(1))?
            .lifecycle(),
        KeyLifecycleState::Compromised
    );

    let retired = KeyMetadata::with_lifecycle_for_test(
        id(KeyId::from_u128(62))?,
        None,
        KeyScope::RootTrust,
        CryptoEpoch::new(4),
        KeyLifecycleState::Retired,
    );
    assert_eq!(
        retired
            .transition(KeyLifecycleState::Compromised, CryptoEpoch::new(4))?
            .lifecycle(),
        KeyLifecycleState::Compromised
    );
    assert_eq!(
        retired
            .transition(KeyLifecycleState::Quarantined, CryptoEpoch::new(4))?
            .lifecycle(),
        KeyLifecycleState::Quarantined
    );

    let quarantined = KeyMetadata::with_lifecycle_for_test(
        id(KeyId::from_u128(63))?,
        None,
        KeyScope::RootTrust,
        CryptoEpoch::new(6),
        KeyLifecycleState::Quarantined,
    );
    assert_eq!(
        quarantined
            .transition(KeyLifecycleState::Compromised, CryptoEpoch::new(6))?
            .lifecycle(),
        KeyLifecycleState::Compromised
    );
    Ok(())
}

#[test]
fn metadata_only_lifecycle_changes_preserve_crypto_epoch_and_advance_event_sequence() -> Result<()>
{
    let active = KeyMetadata::new(
        id(KeyId::from_u128(65))?,
        None,
        KeyScope::RootTrust,
        CryptoEpoch::new(10),
    )
    .transition(KeyLifecycleState::Active, CryptoEpoch::new(11))?;

    let compromised = active.transition(KeyLifecycleState::Compromised, CryptoEpoch::new(11))?;
    assert_eq!(compromised.epoch(), CryptoEpoch::new(11));
    assert_eq!(compromised.lifecycle_event_sequence().get(), 3);

    let quarantined =
        compromised.transition(KeyLifecycleState::Quarantined, CryptoEpoch::new(11))?;
    assert_eq!(quarantined.epoch(), CryptoEpoch::new(11));
    assert_eq!(quarantined.lifecycle_event_sequence().get(), 4);

    let destroyed = quarantined.transition(KeyLifecycleState::Destroyed, CryptoEpoch::new(11))?;
    assert_eq!(destroyed.epoch(), CryptoEpoch::new(11));
    assert_eq!(destroyed.lifecycle_event_sequence().get(), 5);

    let erased = destroyed.crypto_erase(KeyErasureReason::Compromise)?;
    assert_eq!(erased.epoch(), CryptoEpoch::new(11));
    assert_eq!(erased.lifecycle_event_sequence().get(), 6);
    let erasure = erased
        .erasure()
        .ok_or(SkrifheimError::InvalidKeyLifecycle)?;
    assert_eq!(erasure.epoch(), CryptoEpoch::new(11));
    assert_eq!(erasure.lifecycle_event_sequence().get(), 6);
    Ok(())
}

#[test]
fn reconstructed_key_metadata_requires_valid_lifecycle_sequence_and_erasure() -> Result<()> {
    let key_id = id(KeyId::from_u128(66))?;
    let scope = KeyScope::RootTrust;
    assert_eq!(
        KeyMetadata::reconstruct(
            key_id,
            None,
            scope,
            CryptoEpoch::new(3),
            KeyLifecycleEventSequence::initial(),
            KeyLifecycleState::Compromised,
            None,
        ),
        Err(SkrifheimError::InvalidKeyLifecycle)
    );

    let destroyed = KeyMetadata::reconstruct(
        key_id,
        None,
        scope,
        CryptoEpoch::new(3),
        KeyLifecycleEventSequence::new(4),
        KeyLifecycleState::Destroyed,
        None,
    )?;
    assert_eq!(destroyed.lifecycle(), KeyLifecycleState::Destroyed);
    assert_eq!(destroyed.lifecycle_event_sequence().get(), 4);

    assert_eq!(
        KeyMetadata::reconstruct(
            key_id,
            None,
            scope,
            CryptoEpoch::new(3),
            KeyLifecycleEventSequence::new(5),
            KeyLifecycleState::CryptoErased,
            None,
        ),
        Err(SkrifheimError::InvalidKeyLifecycle)
    );
    Ok(())
}

#[test]
fn key_rotation_preflight_rejects_wrong_scope_or_epoch() -> Result<()> {
    let tenant_id = id(TenantId::from_u128(12))?;
    let current = KeyMetadata::with_lifecycle_for_test(
        id(KeyId::from_u128(70))?,
        None,
        KeyScope::Compartment {
            tenant_id,
            compartment_id: id(CompartmentKeyId::from_u128(13))?,
        },
        CryptoEpoch::new(5),
        KeyLifecycleState::Active,
    );
    let wrong_scope = KeyMetadata::new(
        id(KeyId::from_u128(71))?,
        None,
        KeyScope::Segment {
            tenant_id,
            compartment_id: id(CompartmentKeyId::from_u128(13))?,
            segment_id: id(SegmentKeyId::from_u128(14))?,
        },
        CryptoEpoch::new(6),
    );
    let stale_epoch = KeyMetadata::new(
        id(KeyId::from_u128(72))?,
        None,
        current.scope(),
        CryptoEpoch::new(5),
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
        CryptoEpoch::new(0),
    )
    .transition(KeyLifecycleState::Active, CryptoEpoch::new(1))?
    .transition(KeyLifecycleState::Compromised, CryptoEpoch::new(1))?
    .transition(KeyLifecycleState::Quarantined, CryptoEpoch::new(1))?
    .transition(KeyLifecycleState::Destroyed, CryptoEpoch::new(1))?
    .crypto_erase(KeyErasureReason::Compromise)?;

    assert_eq!(key.lifecycle(), KeyLifecycleState::CryptoErased);
    let erasure = key.erasure().ok_or(SkrifheimError::InvalidKeyLifecycle)?;
    assert_eq!(erasure.key_id(), key.key_id());
    assert_eq!(erasure.epoch(), CryptoEpoch::new(1));
    assert_eq!(erasure.lifecycle_event_sequence().get(), 6);
    assert_eq!(erasure.reason(), KeyErasureReason::Compromise);
    Ok(())
}

#[test]
fn active_key_cannot_be_crypto_erased_directly() -> Result<()> {
    let key = KeyMetadata::with_lifecycle_for_test(
        id(KeyId::from_u128(90))?,
        None,
        KeyScope::RootTrust,
        CryptoEpoch::new(1),
        KeyLifecycleState::Active,
    );
    assert_eq!(
        key.crypto_erase(KeyErasureReason::OperatorApproved),
        Err(SkrifheimError::InvalidKeyLifecycle)
    );
    Ok(())
}

#[test]
fn compromised_or_quarantined_key_must_be_destroyed_before_crypto_erasure() -> Result<()> {
    let compromised = KeyMetadata::with_lifecycle_for_test(
        id(KeyId::from_u128(91))?,
        None,
        KeyScope::RootTrust,
        CryptoEpoch::new(1),
        KeyLifecycleState::Compromised,
    );
    let quarantined = KeyMetadata::with_lifecycle_for_test(
        id(KeyId::from_u128(92))?,
        None,
        KeyScope::RootTrust,
        CryptoEpoch::new(1),
        KeyLifecycleState::Quarantined,
    );

    assert_eq!(
        compromised.crypto_erase(KeyErasureReason::Compromise),
        Err(SkrifheimError::InvalidKeyLifecycle)
    );
    assert_eq!(
        quarantined.crypto_erase(KeyErasureReason::Compromise),
        Err(SkrifheimError::InvalidKeyLifecycle)
    );
    Ok(())
}
