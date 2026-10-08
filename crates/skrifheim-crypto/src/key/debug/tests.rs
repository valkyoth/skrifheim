use super::*;
use alloc::format;

fn check(value: &impl fmt::Debug, name: &str) {
    let expected = format!("{name}(<redacted>)");
    assert_eq!(format!("{value:?}"), expected);
    assert_eq!(format!("{value:#?}"), expected);
}

#[test]
fn identifiers_and_all_key_control_states_are_redacted() -> Result<()> {
    for raw in [1, u128::MAX] {
        let raw = NonZeroU128::new(raw).ok_or(SkrifheimError::InvalidIdentifier)?;
        let key = KeyId::new(raw);
        let deployment_id = DeploymentKeyId::new(raw);
        let region_id = RegionKeyId::new(raw);
        let compartment_id = CompartmentKeyId::new(raw);
        let segment_id = SegmentKeyId::new(raw);
        let tenant_id = TenantId::from_u128(raw.get()).ok_or(SkrifheimError::InvalidIdentifier)?;
        check(&key, "KeyId");
        check(&deployment_id, "DeploymentKeyId");
        check(&region_id, "RegionKeyId");
        check(&compartment_id, "CompartmentKeyId");
        check(&segment_id, "SegmentKeyId");
        for scope in [
            KeyScope::RootTrust,
            KeyScope::Deployment { deployment_id },
            KeyScope::Region {
                deployment_id,
                region_id,
            },
            KeyScope::Tenant {
                deployment_id,
                region_id,
                tenant_id,
            },
            KeyScope::Compartment {
                deployment_id,
                region_id,
                tenant_id,
                compartment_id,
            },
            KeyScope::Segment {
                deployment_id,
                region_id,
                tenant_id,
                compartment_id,
                segment_id,
            },
            KeyScope::Data {
                deployment_id,
                region_id,
                tenant_id,
                compartment_id,
                segment_id,
            },
        ] {
            check(&scope, "KeyScope");
            for sequence in [1, u64::MAX] {
                let epoch = CryptoEpoch::new(sequence);
                let lifecycle_event_sequence = KeyLifecycleEventSequence::new(sequence);
                check(&lifecycle_event_sequence, "KeyLifecycleEventSequence");
                for reason in [
                    KeyErasureReason::Rotation,
                    KeyErasureReason::Expiration,
                    KeyErasureReason::Compromise,
                    KeyErasureReason::OperatorApproved,
                ] {
                    check(&reason, "KeyErasureReason");
                    let erasure = KeyErasureMetadata {
                        key_id: key,
                        scope,
                        epoch,
                        lifecycle_event_sequence,
                        reason,
                    };
                    check(&erasure, "KeyErasureMetadata");
                    for lifecycle in [
                        KeyLifecycleState::Created,
                        KeyLifecycleState::Active,
                        KeyLifecycleState::Rotating,
                        KeyLifecycleState::Retired,
                        KeyLifecycleState::Compromised,
                        KeyLifecycleState::Quarantined,
                        KeyLifecycleState::Destroyed,
                        KeyLifecycleState::CryptoErased,
                    ] {
                        check(&lifecycle, "KeyLifecycleState");
                        for parent in [None, Some(key)] {
                            for erasure in [None, Some(erasure)] {
                                let metadata = KeyMetadata {
                                    key_id: key,
                                    parent,
                                    scope,
                                    epoch,
                                    lifecycle_event_sequence,
                                    lifecycle,
                                    erasure,
                                };
                                check(&metadata, "KeyMetadata");
                            }
                        }
                    }
                }
                let rotation = KeyRotationPreflight {
                    current_key: key,
                    candidate_key: key,
                    scope,
                    from_epoch: epoch,
                    to_epoch: epoch,
                };
                check(&rotation, "KeyRotationPreflight");
            }
        }
    }
    Ok(())
}
