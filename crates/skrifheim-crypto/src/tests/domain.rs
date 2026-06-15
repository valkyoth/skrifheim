use super::*;
use skrifheim_core::{Classification, WorldId};

fn tenant() -> Result<TenantId> {
    id(TenantId::from_u128(12))
}

fn region(value: u128) -> Result<RegionKeyId> {
    id(RegionKeyId::from_u128(value))
}

fn compartment(value: u128) -> Result<CompartmentKeyId> {
    id(CompartmentKeyId::from_u128(value))
}

fn segment(value: u128) -> Result<SegmentKeyId> {
    id(SegmentKeyId::from_u128(value))
}

fn world(value: u128) -> Result<WorldId> {
    id(WorldId::from_u128(value))
}

#[test]
fn tenant_region_and_classification_domains_are_distinct() -> Result<()> {
    let tenant_id = tenant()?;
    let tenant_domain = EncryptionDomain::tenant(tenant_id);
    let region_domain = EncryptionDomain::region(tenant_id, region(1)?);
    let secret_domain =
        EncryptionDomain::classification(tenant_id, Some(region(1)?), Classification::Secret);

    assert_eq!(tenant_domain.purpose(), EncryptionDomainPurpose::Tenant);
    assert_eq!(region_domain.region_id(), Some(region(1)?));
    assert_eq!(
        secret_domain.classification_level(),
        Some(Classification::Secret)
    );
    assert!(!tenant_domain.is_merge_compatible_with(region_domain));
    assert!(!region_domain.is_merge_compatible_with(secret_domain));
    Ok(())
}

#[test]
fn compartment_domains_reject_cross_compartment_merges() -> Result<()> {
    let left = EncryptionDomain::compartment(
        tenant()?,
        Some(region(1)?),
        Classification::TopSecret,
        compartment(21)?,
    );
    let right = EncryptionDomain::compartment(
        tenant()?,
        Some(region(1)?),
        Classification::TopSecret,
        compartment(22)?,
    );

    assert_eq!(left.compartment_id(), Some(compartment(21)?));
    assert_eq!(left.merge_with(right), None);
    Ok(())
}

#[test]
fn world_domains_are_branch_specific() -> Result<()> {
    let production = EncryptionDomain::world(
        tenant()?,
        Some(region(1)?),
        Some(Classification::Secret),
        Some(compartment(21)?),
        world(31)?,
    );
    let simulation = EncryptionDomain::world(
        tenant()?,
        Some(region(1)?),
        Some(Classification::Secret),
        Some(compartment(21)?),
        world(32)?,
    );

    assert_eq!(production.world_id(), Some(world(31)?));
    assert!(!production.is_merge_compatible_with(simulation));
    Ok(())
}

#[test]
fn segment_domains_include_segment_identity() -> Result<()> {
    let left = EncryptionDomain::segment(
        tenant()?,
        Some(region(1)?),
        Classification::Restricted,
        compartment(21)?,
        segment(41)?,
    );
    let same = EncryptionDomain::segment(
        tenant()?,
        Some(region(1)?),
        Classification::Restricted,
        compartment(21)?,
        segment(41)?,
    );
    let other_segment = EncryptionDomain::segment(
        tenant()?,
        Some(region(1)?),
        Classification::Restricted,
        compartment(21)?,
        segment(42)?,
    );

    assert_eq!(left.segment_id(), Some(segment(41)?));
    assert_eq!(left.merge_with(same), Some(left));
    assert_eq!(left.merge_with(other_segment), None);
    Ok(())
}

#[test]
fn special_purpose_domains_do_not_merge_with_data_domains() -> Result<()> {
    let tenant_id = tenant()?;
    let region_id = Some(region(1)?);
    let projection = EncryptionDomain::projection(
        tenant_id,
        region_id,
        Classification::Secret,
        Some(compartment(21)?),
        Some(world(31)?),
    );
    let ai = EncryptionDomain::ai_artifact(
        tenant_id,
        region_id,
        Classification::Secret,
        Some(compartment(21)?),
        Some(world(31)?),
    );
    let export = EncryptionDomain::export_capsule(tenant_id, region_id, Classification::Secret);
    let backup = EncryptionDomain::backup(tenant_id, region_id);
    let wasm = EncryptionDomain::wasm_plugin_secret(tenant_id, region_id);
    let audit = EncryptionDomain::audit_log(tenant_id, region_id);
    let wal = EncryptionDomain::wal(tenant_id, region_id, Some(world(31)?));

    assert!(!projection.is_merge_compatible_with(ai));
    assert!(!projection.is_merge_compatible_with(export));
    assert!(!backup.is_merge_compatible_with(wasm));
    assert!(!audit.is_merge_compatible_with(wal));
    assert_eq!(ai.purpose(), EncryptionDomainPurpose::AiArtifact);
    Ok(())
}
