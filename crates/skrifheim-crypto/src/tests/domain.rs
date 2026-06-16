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
    assert!(left.merge_with(right).is_none());
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
    assert!(
        left.merge_with(same)
            .is_some_and(|merged| merged.structurally_equal(&left))
    );
    assert!(left.merge_with(other_segment).is_none());
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

#[test]
fn projection_policy_requires_projection_domains() -> Result<()> {
    let tenant_id = tenant()?;
    let region_id = Some(region(1)?);
    let domain = EncryptionDomain::projection(
        tenant_id,
        region_id,
        Classification::Secret,
        Some(compartment(21)?),
        Some(world(31)?),
    );
    let segment_domain = EncryptionDomain::segment(
        tenant_id,
        region_id,
        Classification::Secret,
        compartment(21)?,
        segment(41)?,
    );

    let secondary = ProjectionEncryptionPolicy::secondary_index(domain)?;
    let graph = ProjectionEncryptionPolicy::graph_index(domain)?;
    let search = ProjectionEncryptionPolicy::search_index(domain)?;
    let vector = ProjectionEncryptionPolicy::vector_index(domain)?;
    let columnar = ProjectionEncryptionPolicy::columnar_projection(domain)?;

    assert!(secondary.requires_encryption_at_rest());
    assert!(graph.requires_encryption_at_rest());
    assert!(search.requires_encryption_at_rest());
    assert!(vector.requires_encryption_at_rest());
    assert!(columnar.requires_encryption_at_rest());
    assert!(matches!(
        ProjectionEncryptionPolicy::secondary_index(segment_domain),
        Err(SkrifheimError::InvalidProjectionPolicy)
    ));
    assert!(matches!(
        ProjectionEncryptionPolicy::secondary_index(
            EncryptionDomain::projection_without_classification_for_test(tenant_id)
        ),
        Err(SkrifheimError::InvalidProjectionPolicy)
    ));
    Ok(())
}

#[test]
fn projection_policy_rejects_cross_compartment_mixing() -> Result<()> {
    let tenant_id = tenant()?;
    let region_id = Some(region(1)?);
    let left = ProjectionEncryptionPolicy::search_index(EncryptionDomain::projection(
        tenant_id,
        region_id,
        Classification::Secret,
        Some(compartment(21)?),
        Some(world(31)?),
    ))?;
    let right = ProjectionEncryptionPolicy::search_index(EncryptionDomain::projection(
        tenant_id,
        region_id,
        Classification::Secret,
        Some(compartment(22)?),
        Some(world(31)?),
    ))?;

    assert!(!left.is_domain_compatible_with(right));
    assert!(left.merge_with(right).is_none());
    Ok(())
}

#[test]
fn projection_policy_compatibility_requires_same_surface() -> Result<()> {
    let domain = EncryptionDomain::projection(
        tenant()?,
        Some(region(1)?),
        Classification::Secret,
        Some(compartment(21)?),
        Some(world(31)?),
    );
    let secondary = ProjectionEncryptionPolicy::secondary_index(domain)?;
    let vector = ProjectionEncryptionPolicy::vector_index(domain)?;

    assert!(!secondary.is_domain_compatible_with(vector));
    assert!(secondary.merge_with(vector).is_none());
    Ok(())
}

#[test]
fn compaction_temporary_projection_files_are_encrypted() -> Result<()> {
    let policy =
        ProjectionEncryptionPolicy::compaction_temporary_file(EncryptionDomain::projection(
            tenant()?,
            Some(region(1)?),
            Classification::Restricted,
            Some(compartment(21)?),
            Some(world(31)?),
        ))?;

    assert!(matches!(
        policy.surface(),
        ProjectionSurface::CompactionTemporaryFile
    ));
    assert!(policy.requires_encryption_at_rest());
    assert!(!policy.allows_plaintext_temporary_files());
    Ok(())
}

#[test]
fn encryption_domain_debug_redacts_sensitive_metadata() -> Result<()> {
    let domain = EncryptionDomain::projection(
        tenant()?,
        Some(region(1)?),
        Classification::TopSecret,
        Some(compartment(21)?),
        Some(world(31)?),
    );
    let debug = alloc::format!("{domain:?}");

    assert!(!debug.contains("TopSecret"));
    assert!(!debug.contains("TenantId"));
    assert!(!debug.contains("CompartmentKeyId"));
    assert!(!debug.contains("WorldId"));
    assert!(debug.contains("<redacted>"));
    Ok(())
}

#[test]
fn projection_policy_debug_redacts_surface_and_domain() -> Result<()> {
    let policy = ProjectionEncryptionPolicy::vector_index(EncryptionDomain::projection(
        tenant()?,
        Some(region(1)?),
        Classification::TopSecret,
        Some(compartment(21)?),
        Some(world(31)?),
    ))?;
    let debug = alloc::format!("{policy:?}");

    assert!(!debug.contains("VectorIndex"));
    assert!(!debug.contains("TopSecret"));
    assert!(!debug.contains("CompartmentKeyId"));
    assert!(debug.contains("<redacted>"));
    Ok(())
}

#[test]
fn projection_surface_debug_is_redacted() {
    let debug = alloc::format!("{:?}", ProjectionSurface::VectorIndex);

    assert!(!debug.contains("VectorIndex"));
    assert!(debug.contains("<redacted>"));
}
