use super::*;

#[test]
fn descendant_keys_reject_wrong_deployment_or_region_at_each_level() -> Result<()> {
    let deployment_id = id(DeploymentKeyId::from_u128(10))?;
    let region_id = id(RegionKeyId::from_u128(11))?;
    let tenant_id = id(TenantId::from_u128(12))?;
    let compartment_id = id(CompartmentKeyId::from_u128(13))?;
    let segment_id = id(SegmentKeyId::from_u128(14))?;
    for (wrong_deployment, wrong_region) in [
        (id(DeploymentKeyId::from_u128(99))?, region_id),
        (deployment_id, id(RegionKeyId::from_u128(99))?),
    ] {
        let tenant = KeyMetadata::new(
            id(KeyId::from_u128(1))?,
            None,
            KeyScope::Tenant {
                deployment_id: wrong_deployment,
                region_id: wrong_region,
                tenant_id,
            },
            CryptoEpoch::new(1),
        );
        let compartment = KeyMetadata::new(
            id(KeyId::from_u128(2))?,
            Some(tenant.key_id()),
            KeyScope::Compartment {
                deployment_id,
                region_id,
                tenant_id,
                compartment_id,
            },
            CryptoEpoch::new(1),
        );
        assert_eq!(
            compartment.validate_parent(Some(&tenant)),
            Err(SkrifheimError::InvalidKeyHierarchy)
        );
        let wrong_parent = KeyMetadata::new(
            compartment.key_id(),
            Some(tenant.key_id()),
            KeyScope::Compartment {
                deployment_id: wrong_deployment,
                region_id: wrong_region,
                tenant_id,
                compartment_id,
            },
            CryptoEpoch::new(1),
        );
        for scope in [
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
            let child = KeyMetadata::new(
                id(KeyId::from_u128(3))?,
                Some(compartment.key_id()),
                scope,
                CryptoEpoch::new(1),
            );
            assert_eq!(child.validate_parent(Some(&compartment)), Ok(()));
            assert_eq!(
                child.validate_parent(Some(&wrong_parent)),
                Err(SkrifheimError::InvalidKeyHierarchy)
            );
        }
    }
    Ok(())
}
