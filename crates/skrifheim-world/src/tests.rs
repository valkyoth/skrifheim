use super::*;
use alloc::{string::String, vec};

fn id<T>(id: Option<T>) -> Result<T> {
    id.ok_or(SkrifheimError::InvalidIdentifier)
}

#[test]
fn fork_keeps_parent_identity() -> Result<()> {
    let production = World::root("production", WorldKind::Production)?;
    let draft = production.fork("draft", WorldKind::Simulation)?;
    assert_eq!(draft.parent(), Some(production.id()));
    assert_eq!(draft.depth(), production.depth() + 1);
    Ok(())
}

#[test]
fn deterministic_root_identity_repeats_for_same_metadata() -> Result<()> {
    let first = World::root("production", WorldKind::Production)?;
    let second = World::root("production", WorldKind::Production)?;
    assert_eq!(first.id(), second.id());
    assert_eq!(first.metadata(), second.metadata());
    Ok(())
}

#[test]
fn deterministic_child_identity_depends_on_parent() -> Result<()> {
    let production = World::root("production", WorldKind::Production)?;
    let staging = World::root("staging", WorldKind::Staging)?;
    let production_child = production.fork("draft", WorldKind::Simulation)?;
    let repeated_child = production.fork("draft", WorldKind::Simulation)?;
    let staging_child = staging.fork("draft", WorldKind::Simulation)?;
    assert_eq!(production_child.id(), repeated_child.id());
    assert_ne!(production_child.id(), staging_child.id());
    Ok(())
}

#[test]
fn deterministic_identity_changes_with_kind() -> Result<()> {
    let simulation = World::root("draft", WorldKind::Simulation)?;
    let audit = World::root("draft", WorldKind::LegalAudit)?;
    assert_ne!(simulation.id(), audit.id());
    Ok(())
}

#[test]
fn duplicate_fact_adds_are_idempotent() -> Result<()> {
    let mut world = World::root("production", WorldKind::Production)?;
    world.add_fact(id(FactId::from_u128(7))?);
    world.add_fact(id(FactId::from_u128(7))?);
    assert_eq!(world.added_facts(), &[id(FactId::from_u128(7))?]);
    Ok(())
}

#[test]
fn branch_fact_sets_are_isolated_from_parent() -> Result<()> {
    let mut production = World::root("production", WorldKind::Production)?;
    production.add_fact(id(FactId::from_u128(7))?);
    let mut draft = production.fork("draft", WorldKind::Simulation)?;
    draft.add_fact(id(FactId::from_u128(8))?);
    draft.hide_fact(id(FactId::from_u128(7))?);
    assert_eq!(production.added_facts(), &[id(FactId::from_u128(7))?]);
    assert_eq!(production.hidden_facts(), &[]);
    assert_eq!(draft.added_facts(), &[id(FactId::from_u128(8))?]);
    assert_eq!(draft.hidden_facts(), &[id(FactId::from_u128(7))?]);
    Ok(())
}

#[test]
fn diff_requires_direct_child_relationship() -> Result<()> {
    let mut production = World::root("production", WorldKind::Production)?;
    production.add_fact(id(FactId::from_u128(7))?);
    let unrelated = World::root("unrelated", WorldKind::Simulation)?;
    assert_eq!(
        WorldDiff::between(&production, &unrelated),
        Err(SkrifheimError::InvalidWorldDiff)
    );
    Ok(())
}

#[test]
fn diff_returns_delta_against_parent() -> Result<()> {
    let mut production = World::root("production", WorldKind::Production)?;
    production.add_fact(id(FactId::from_u128(7))?);
    let mut child = production.fork("child", WorldKind::Simulation)?;
    child.add_fact(id(FactId::from_u128(7))?);
    child.add_fact(id(FactId::from_u128(8))?);
    child.hide_fact(id(FactId::from_u128(7))?);
    let diff = WorldDiff::between(&production, &child)?;
    assert_eq!(diff.added, vec![id(FactId::from_u128(8))?]);
    assert_eq!(diff.hidden, vec![id(FactId::from_u128(7))?]);
    Ok(())
}

#[test]
fn root_rejects_invalid_world_names() {
    assert_eq!(
        World::root("", WorldKind::Production),
        Err(SkrifheimError::InvalidWorldName)
    );
    assert_eq!(
        World::root("production\nbad", WorldKind::Production),
        Err(SkrifheimError::InvalidWorldName)
    );
}

#[test]
fn root_rejects_overlong_world_names() -> Result<()> {
    let name = String::from_utf8(vec![b'a'; WORLD_NAME_MAX_BYTES + 1])
        .map_err(|_| SkrifheimError::InvalidWorldName)?;
    assert_eq!(
        World::root(name, WorldKind::Production),
        Err(SkrifheimError::InvalidWorldName)
    );
    Ok(())
}
