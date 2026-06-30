use super::*;
use alloc::{string::String, vec};

fn id<T>(id: Option<T>) -> Result<T> {
    id.ok_or(SkrifheimError::InvalidIdentifier)
}

fn tenant(value: u128) -> Result<TenantId> {
    id(TenantId::from_u128(value))
}

#[test]
fn fork_keeps_parent_identity() -> Result<()> {
    let production = World::root(tenant(1)?, "production", WorldKind::Production)?;
    let draft = production.fork("draft", WorldKind::Simulation)?;
    assert_eq!(draft.tenant_id(), production.tenant_id());
    assert_eq!(draft.parent(), Some(production.id()));
    assert_eq!(draft.depth(), production.depth() + 1);
    Ok(())
}

#[test]
fn deterministic_root_identity_repeats_for_same_metadata() -> Result<()> {
    let first = World::root(tenant(1)?, "production", WorldKind::Production)?;
    let second = World::root(tenant(1)?, "production", WorldKind::Production)?;
    assert_eq!(first.id(), second.id());
    assert_eq!(first.metadata(), second.metadata());
    Ok(())
}

#[test]
fn deterministic_root_identity_is_tenant_scoped() -> Result<()> {
    let tenant_one = World::root(tenant(1)?, "production", WorldKind::Production)?;
    let tenant_two = World::root(tenant(2)?, "production", WorldKind::Production)?;
    assert_ne!(tenant_one.id(), tenant_two.id());
    Ok(())
}

#[test]
fn deterministic_child_identity_depends_on_parent_and_tenant() -> Result<()> {
    let production = World::root(tenant(1)?, "production", WorldKind::Production)?;
    let staging = World::root(tenant(1)?, "staging", WorldKind::Staging)?;
    let other_tenant = World::root(tenant(2)?, "production", WorldKind::Production)?;
    let production_child = production.fork("draft", WorldKind::Simulation)?;
    let repeated_child = production.fork("draft", WorldKind::Simulation)?;
    let staging_child = staging.fork("draft", WorldKind::Simulation)?;
    let other_tenant_child = other_tenant.fork("draft", WorldKind::Simulation)?;
    assert_eq!(production_child.id(), repeated_child.id());
    assert_ne!(production_child.id(), staging_child.id());
    assert_ne!(production_child.id(), other_tenant_child.id());
    Ok(())
}

#[test]
fn deterministic_identity_changes_with_kind() -> Result<()> {
    let simulation = World::root(tenant(1)?, "draft", WorldKind::Simulation)?;
    let audit = World::root(tenant(1)?, "draft", WorldKind::LegalAudit)?;
    assert_ne!(simulation.id(), audit.id());
    Ok(())
}

#[test]
fn debug_redacts_world_operational_metadata() -> Result<()> {
    let world = World::root(
        tenant(1)?,
        "operation-arctic-shield",
        WorldKind::MissionCapsule,
    )?;
    let debug = alloc::format!("{world:?}");

    assert!(debug.contains("metadata: \"<redacted>\""));
    assert!(debug.contains("added_facts: \"<redacted>\""));
    assert!(debug.contains("hidden_facts: \"<redacted>\""));
    assert!(!debug.contains("operation-arctic-shield"));
    assert!(!debug.contains("MissionCapsule"));
    assert!(!debug.contains("WorldId"));
    assert!(!debug.contains("TenantId"));
    Ok(())
}

#[test]
fn debug_redacts_world_metadata_operational_fields() -> Result<()> {
    let metadata = WorldMetadata::root(
        tenant(1)?,
        "operation-arctic-shield",
        WorldKind::MissionCapsule,
    )?;
    let debug = alloc::format!("{metadata:?}");

    assert!(debug.contains("name: \"<redacted>\""));
    assert!(debug.contains("kind: \"<redacted>\""));
    assert!(!debug.contains("operation-arctic-shield"));
    assert!(!debug.contains("MissionCapsule"));
    assert!(!debug.contains("WorldId"));
    assert!(!debug.contains("TenantId"));
    Ok(())
}

#[test]
fn deterministic_identity_separates_hash_fields() -> Result<()> {
    let tenant_id = tenant(1)?;
    let parent = id(WorldId::from_u128(2))?;
    let root_with_child_prefix = derive_world_id(
        tenant_id,
        None,
        0,
        "child-production",
        WorldKind::Production,
    )?;
    let child_with_matching_prefix = derive_world_id(
        tenant_id,
        Some(parent),
        1,
        "production",
        WorldKind::Production,
    )?;

    assert_ne!(root_with_child_prefix, child_with_matching_prefix);
    Ok(())
}

#[test]
fn duplicate_fact_adds_are_idempotent() -> Result<()> {
    let mut world = World::root(tenant(1)?, "production", WorldKind::Production)?;
    world.add_fact(id(FactId::from_u128(7))?)?;
    world.add_fact(id(FactId::from_u128(7))?)?;
    assert_eq!(world.added_facts(), &[id(FactId::from_u128(7))?]);
    Ok(())
}

#[test]
fn fact_tracking_is_sorted_and_bounded() -> Result<()> {
    let mut facts = Vec::new();
    insert_fact_id_with_limit(&mut facts, id(FactId::from_u128(30))?, 3)?;
    insert_fact_id_with_limit(&mut facts, id(FactId::from_u128(10))?, 3)?;
    insert_fact_id_with_limit(&mut facts, id(FactId::from_u128(20))?, 3)?;
    insert_fact_id_with_limit(&mut facts, id(FactId::from_u128(20))?, 3)?;

    assert_eq!(
        facts,
        vec![
            id(FactId::from_u128(10))?,
            id(FactId::from_u128(20))?,
            id(FactId::from_u128(30))?,
        ]
    );
    assert_eq!(
        insert_fact_id_with_limit(&mut facts, id(FactId::from_u128(40))?, 3),
        Err(SkrifheimError::TooManyFactLinks)
    );
    Ok(())
}

#[test]
fn branch_fact_sets_are_isolated_from_parent() -> Result<()> {
    let mut production = World::root(tenant(1)?, "production", WorldKind::Production)?;
    production.add_fact(id(FactId::from_u128(7))?)?;
    let mut draft = production.fork("draft", WorldKind::Simulation)?;
    draft.add_fact(id(FactId::from_u128(8))?)?;
    draft.hide_fact(id(FactId::from_u128(7))?)?;
    assert_eq!(production.added_facts(), &[id(FactId::from_u128(7))?]);
    assert_eq!(production.hidden_facts(), &[]);
    assert_eq!(draft.added_facts(), &[id(FactId::from_u128(8))?]);
    assert_eq!(draft.hidden_facts(), &[id(FactId::from_u128(7))?]);
    Ok(())
}

#[test]
fn diff_requires_direct_child_relationship() -> Result<()> {
    let mut production = World::root(tenant(1)?, "production", WorldKind::Production)?;
    production.add_fact(id(FactId::from_u128(7))?)?;
    let unrelated = World::root(tenant(1)?, "unrelated", WorldKind::Simulation)?;
    assert_eq!(
        WorldDiff::between(&production, &unrelated),
        Err(SkrifheimError::InvalidWorldDiff)
    );
    Ok(())
}

#[test]
fn diff_requires_same_tenant() -> Result<()> {
    let production = World::root(tenant(1)?, "production", WorldKind::Production)?;
    let other_tenant = World::root(tenant(2)?, "production", WorldKind::Production)?;
    assert_eq!(
        WorldDiff::between(&production, &other_tenant),
        Err(SkrifheimError::InvalidWorldDiff)
    );
    Ok(())
}

#[test]
fn diff_returns_delta_against_parent() -> Result<()> {
    let mut production = World::root(tenant(1)?, "production", WorldKind::Production)?;
    production.add_fact(id(FactId::from_u128(7))?)?;
    let mut child = production.fork("child", WorldKind::Simulation)?;
    child.add_fact(id(FactId::from_u128(7))?)?;
    child.add_fact(id(FactId::from_u128(8))?)?;
    child.hide_fact(id(FactId::from_u128(7))?)?;
    let diff = WorldDiff::between(&production, &child)?;
    assert_eq!(diff.added, vec![id(FactId::from_u128(8))?]);
    assert_eq!(diff.hidden, vec![id(FactId::from_u128(7))?]);
    Ok(())
}

#[test]
fn promotion_preflight_allows_clean_child() -> Result<()> {
    let mut production = World::root(tenant(1)?, "production", WorldKind::Production)?;
    production.add_fact(id(FactId::from_u128(7))?)?;
    let mut child = production.fork("review", WorldKind::Staging)?;
    child.add_fact(id(FactId::from_u128(8))?)?;
    child.hide_fact(id(FactId::from_u128(7))?)?;

    let preflight = production.promotion_preflight(&child)?;
    assert!(preflight.can_promote());
    assert!(!preflight.is_storage_validated());
    assert_eq!(
        preflight.require_storage_validated(),
        Err(SkrifheimError::InvalidWorldDiff)
    );
    assert_eq!(preflight.diff.added, vec![id(FactId::from_u128(8))?]);
    assert_eq!(preflight.diff.hidden, vec![id(FactId::from_u128(7))?]);
    Ok(())
}

#[test]
fn promotion_preflight_rejects_non_child_relationship() -> Result<()> {
    let production = World::root(tenant(1)?, "production", WorldKind::Production)?;
    let unrelated = World::root(tenant(1)?, "unrelated", WorldKind::Staging)?;

    assert_eq!(
        production.promotion_preflight(&unrelated),
        Err(SkrifheimError::InvalidWorldDiff)
    );
    Ok(())
}

#[test]
fn promotion_preflight_detects_fact_added_and_hidden() -> Result<()> {
    let production = World::root(tenant(1)?, "production", WorldKind::Production)?;
    let mut child = production.fork("review", WorldKind::Staging)?;
    child.add_fact(id(FactId::from_u128(8))?)?;
    child.hide_fact(id(FactId::from_u128(8))?)?;

    let preflight = production.promotion_preflight(&child)?;
    assert!(!preflight.can_promote());
    assert_eq!(
        preflight.conflicts,
        vec![WorldConflict {
            kind: WorldConflictKind::AddedAndHiddenSameFact,
            fact_id: id(FactId::from_u128(8))?,
        }]
    );
    Ok(())
}

#[test]
fn promotion_preflight_detects_reintroduced_hidden_fact() -> Result<()> {
    let mut production = World::root(tenant(1)?, "production", WorldKind::Production)?;
    production.hide_fact(id(FactId::from_u128(7))?)?;
    let mut child = production.fork("review", WorldKind::Staging)?;
    child.add_fact(id(FactId::from_u128(7))?)?;

    let preflight = production.promotion_preflight(&child)?;
    assert!(!preflight.can_promote());
    assert_eq!(
        preflight.conflicts,
        vec![WorldConflict {
            kind: WorldConflictKind::ReintroducesParentHiddenFact,
            fact_id: id(FactId::from_u128(7))?,
        }]
    );
    Ok(())
}

#[test]
fn rollback_preflight_reports_inverse_delta() -> Result<()> {
    let mut production = World::root(tenant(1)?, "production", WorldKind::Production)?;
    production.add_fact(id(FactId::from_u128(7))?)?;
    let mut child = production.fork("review", WorldKind::Staging)?;
    child.add_fact(id(FactId::from_u128(8))?)?;
    child.hide_fact(id(FactId::from_u128(7))?)?;

    let preflight = production.rollback_preflight(&child)?;
    assert!(preflight.can_rollback());
    assert!(!preflight.is_storage_validated());
    assert_eq!(
        preflight.require_storage_validated(),
        Err(SkrifheimError::InvalidWorldDiff)
    );
    assert_eq!(preflight.from, child.id());
    assert_eq!(preflight.to, production.id());
    assert_eq!(preflight.reverts_added, vec![id(FactId::from_u128(8))?]);
    assert_eq!(preflight.restores_hidden, vec![id(FactId::from_u128(7))?]);
    Ok(())
}

#[test]
fn root_rejects_invalid_world_names() -> Result<()> {
    assert_eq!(
        World::root(tenant(1)?, "", WorldKind::Production),
        Err(SkrifheimError::InvalidWorldName)
    );
    assert_eq!(
        World::root(tenant(1)?, "production\nbad", WorldKind::Production),
        Err(SkrifheimError::InvalidWorldName)
    );
    assert_eq!(
        World::root(tenant(1)?, "production bad", WorldKind::Production),
        Err(SkrifheimError::InvalidWorldName)
    );
    assert_eq!(
        World::root(tenant(1)?, "../production", WorldKind::Production),
        Err(SkrifheimError::InvalidWorldName)
    );
    Ok(())
}

#[test]
fn root_rejects_overlong_world_names() -> Result<()> {
    let name = String::from_utf8(vec![b'a'; WORLD_NAME_MAX_BYTES + 1])
        .map_err(|_| SkrifheimError::InvalidWorldName)?;
    assert_eq!(
        World::root(tenant(1)?, name, WorldKind::Production),
        Err(SkrifheimError::InvalidWorldName)
    );
    Ok(())
}
