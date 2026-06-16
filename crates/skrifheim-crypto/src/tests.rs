use super::*;
use skrifheim_core::{Result, TenantId};

mod domain;
mod key_lifecycle;
mod secret;
mod signature;

fn id<T>(id: Option<T>) -> Result<T> {
    id.ok_or(SkrifheimError::InvalidIdentifier)
}

#[test]
fn key_hierarchy_accepts_valid_edges() -> Result<()> {
    let root = KeyMetadata::new(
        id(KeyId::from_u128(1))?,
        None,
        KeyScope::RootTrust,
        CryptoEpoch::new(1),
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
        CryptoEpoch::new(1),
    );
    let region = KeyMetadata::new(
        id(KeyId::from_u128(3))?,
        Some(deployment.key_id()),
        KeyScope::Region {
            deployment_id,
            region_id,
        },
        CryptoEpoch::new(1),
    );
    let tenant = KeyMetadata::new(
        id(KeyId::from_u128(4))?,
        Some(region.key_id()),
        KeyScope::Tenant {
            deployment_id,
            region_id,
            tenant_id,
        },
        CryptoEpoch::new(1),
    );
    let compartment = KeyMetadata::new(
        id(KeyId::from_u128(5))?,
        Some(tenant.key_id()),
        KeyScope::Compartment {
            tenant_id,
            compartment_id,
        },
        CryptoEpoch::new(1),
    );
    let segment = KeyMetadata::new(
        id(KeyId::from_u128(6))?,
        Some(compartment.key_id()),
        KeyScope::Segment {
            tenant_id,
            compartment_id,
            segment_id,
        },
        CryptoEpoch::new(1),
    );
    let data = KeyMetadata::new(
        id(KeyId::from_u128(7))?,
        Some(compartment.key_id()),
        KeyScope::Data {
            tenant_id,
            compartment_id,
            segment_id,
        },
        CryptoEpoch::new(1),
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
        CryptoEpoch::new(1),
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
        CryptoEpoch::new(1),
    );
    let tenant = KeyMetadata::new(
        id(KeyId::from_u128(4))?,
        Some(root.key_id()),
        KeyScope::Tenant {
            deployment_id: id(DeploymentKeyId::from_u128(10))?,
            region_id: id(RegionKeyId::from_u128(11))?,
            tenant_id: id(TenantId::from_u128(12))?,
        },
        CryptoEpoch::new(1),
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
        CryptoEpoch::new(1),
        KeyLifecycleState::Compromised,
    );
    let deployment = KeyMetadata::new(
        id(KeyId::from_u128(2))?,
        Some(root.key_id()),
        KeyScope::Deployment {
            deployment_id: id(DeploymentKeyId::from_u128(10))?,
        },
        CryptoEpoch::new(1),
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
        CryptoEpoch::new(1),
    );
    let tenant = KeyMetadata::new(
        id(KeyId::from_u128(4))?,
        Some(region.key_id()),
        KeyScope::Tenant {
            deployment_id,
            region_id: other_region_id,
            tenant_id: id(TenantId::from_u128(12))?,
        },
        CryptoEpoch::new(1),
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
        CryptoEpoch::new(1),
    );
    let data = KeyMetadata::new(
        id(KeyId::from_u128(7))?,
        Some(tenant.key_id()),
        KeyScope::Data {
            tenant_id,
            compartment_id: id(CompartmentKeyId::from_u128(13))?,
            segment_id: id(SegmentKeyId::from_u128(14))?,
        },
        CryptoEpoch::new(1),
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
        CryptoEpoch::new(1),
    );
    let segment = KeyMetadata::new(
        id(KeyId::from_u128(6))?,
        Some(parent_compartment.key_id()),
        KeyScope::Segment {
            tenant_id,
            compartment_id: id(CompartmentKeyId::from_u128(99))?,
            segment_id: id(SegmentKeyId::from_u128(14))?,
        },
        CryptoEpoch::new(1),
    );

    assert_eq!(
        segment.validate_parent(Some(&parent_compartment)),
        Err(SkrifheimError::InvalidKeyHierarchy)
    );
    Ok(())
}
