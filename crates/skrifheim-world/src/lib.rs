#![no_std]
#![forbid(unsafe_code)]

extern crate alloc;

use alloc::{string::String, vec::Vec};
use skrifheim_core::{FactId, Result, SkrifheimError, WorldId};

pub const WORLD_NAME_MAX_BYTES: usize = 256;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorldKind {
    Production,
    Staging,
    UserLocal,
    AgentScratchpad,
    Simulation,
    LegalAudit,
    MissionCapsule,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct World {
    id: WorldId,
    name: String,
    kind: WorldKind,
    parent: Option<WorldId>,
    added_facts: Vec<FactId>,
    hidden_facts: Vec<FactId>,
}

impl World {
    pub fn root(id: WorldId, name: impl Into<String>, kind: WorldKind) -> Result<Self> {
        Ok(Self {
            id,
            name: validate_world_name(name.into())?,
            kind,
            parent: None,
            added_facts: Vec::new(),
            hidden_facts: Vec::new(),
        })
    }

    pub fn fork(&self, id: WorldId, name: impl Into<String>, kind: WorldKind) -> Result<Self> {
        if id == self.id {
            return Err(SkrifheimError::InvalidWorldDiff);
        }
        Ok(Self {
            id,
            name: validate_world_name(name.into())?,
            kind,
            parent: Some(self.id),
            added_facts: Vec::new(),
            hidden_facts: Vec::new(),
        })
    }

    #[must_use]
    pub const fn id(&self) -> WorldId {
        self.id
    }

    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    #[must_use]
    pub const fn kind(&self) -> &WorldKind {
        &self.kind
    }

    #[must_use]
    pub const fn parent(&self) -> Option<WorldId> {
        self.parent
    }

    #[must_use]
    pub fn added_facts(&self) -> &[FactId] {
        &self.added_facts
    }

    #[must_use]
    pub fn hidden_facts(&self) -> &[FactId] {
        &self.hidden_facts
    }

    pub fn add_fact(&mut self, fact_id: FactId) {
        if !self.added_facts.contains(&fact_id) {
            self.added_facts.push(fact_id);
        }
    }

    pub fn hide_fact(&mut self, fact_id: FactId) {
        if !self.hidden_facts.contains(&fact_id) {
            self.hidden_facts.push(fact_id);
        }
    }
}

fn validate_world_name(name: String) -> Result<String> {
    if name.is_empty() || name.len() > WORLD_NAME_MAX_BYTES {
        return Err(SkrifheimError::InvalidWorldName);
    }
    if !name.bytes().all(|byte| {
        byte.is_ascii_alphanumeric()
            || byte == b' '
            || matches!(byte, b'-' | b'_' | b'.' | b':' | b'/')
    }) {
        return Err(SkrifheimError::InvalidWorldName);
    }
    Ok(name)
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorldDiff {
    pub from: WorldId,
    pub to: WorldId,
    pub added: Vec<FactId>,
    pub hidden: Vec<FactId>,
}

impl WorldDiff {
    pub fn between(from: &World, to: &World) -> Result<Self> {
        if from.id != to.id && to.parent != Some(from.id) {
            return Err(SkrifheimError::InvalidWorldDiff);
        }
        Ok(Self {
            from: from.id,
            to: to.id,
            added: to
                .added_facts
                .iter()
                .filter(|fact_id| !from.added_facts.contains(fact_id))
                .copied()
                .collect(),
            hidden: to
                .hidden_facts
                .iter()
                .filter(|fact_id| !from.hidden_facts.contains(fact_id))
                .copied()
                .collect(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;

    fn id<T>(id: Option<T>) -> Result<T> {
        id.ok_or(SkrifheimError::InvalidIdentifier)
    }

    #[test]
    fn fork_keeps_parent_identity() -> Result<()> {
        let production = World::root(
            id(WorldId::from_u128(1))?,
            "production",
            WorldKind::Production,
        )?;
        let draft = production.fork(id(WorldId::from_u128(2))?, "draft", WorldKind::Simulation)?;
        assert_eq!(draft.parent(), Some(id(WorldId::from_u128(1))?));
        Ok(())
    }

    #[test]
    fn duplicate_fact_adds_are_idempotent() -> Result<()> {
        let mut world = World::root(
            id(WorldId::from_u128(1))?,
            "production",
            WorldKind::Production,
        )?;
        world.add_fact(id(FactId::from_u128(7))?);
        world.add_fact(id(FactId::from_u128(7))?);
        assert_eq!(world.added_facts(), &[id(FactId::from_u128(7))?]);
        Ok(())
    }

    #[test]
    fn diff_requires_direct_child_relationship() -> Result<()> {
        let mut production = World::root(
            id(WorldId::from_u128(1))?,
            "production",
            WorldKind::Production,
        )?;
        production.add_fact(id(FactId::from_u128(7))?);
        let unrelated = World::root(
            id(WorldId::from_u128(2))?,
            "unrelated",
            WorldKind::Simulation,
        )?;
        assert_eq!(
            WorldDiff::between(&production, &unrelated),
            Err(SkrifheimError::InvalidWorldDiff)
        );
        Ok(())
    }

    #[test]
    fn diff_returns_delta_against_parent() -> Result<()> {
        let mut production = World::root(
            id(WorldId::from_u128(1))?,
            "production",
            WorldKind::Production,
        )?;
        production.add_fact(id(FactId::from_u128(7))?);
        let mut child =
            production.fork(id(WorldId::from_u128(2))?, "child", WorldKind::Simulation)?;
        child.add_fact(id(FactId::from_u128(7))?);
        child.add_fact(id(FactId::from_u128(8))?);
        let diff = WorldDiff::between(&production, &child)?;
        assert_eq!(diff.added, vec![id(FactId::from_u128(8))?]);
        Ok(())
    }

    #[test]
    fn fork_rejects_self_parent() -> Result<()> {
        let production = World::root(
            id(WorldId::from_u128(1))?,
            "production",
            WorldKind::Production,
        )?;
        assert_eq!(
            production.fork(id(WorldId::from_u128(1))?, "bad", WorldKind::Simulation),
            Err(SkrifheimError::InvalidWorldDiff)
        );
        Ok(())
    }

    #[test]
    fn root_rejects_invalid_world_names() -> Result<()> {
        assert_eq!(
            World::root(id(WorldId::from_u128(1))?, "", WorldKind::Production),
            Err(SkrifheimError::InvalidWorldName)
        );
        assert_eq!(
            World::root(
                id(WorldId::from_u128(1))?,
                "production\nbad",
                WorldKind::Production
            ),
            Err(SkrifheimError::InvalidWorldName)
        );
        Ok(())
    }

    #[test]
    fn root_rejects_overlong_world_names() -> Result<()> {
        let name = String::from_utf8(vec![b'a'; WORLD_NAME_MAX_BYTES + 1])
            .map_err(|_| SkrifheimError::InvalidWorldName)?;
        assert_eq!(
            World::root(id(WorldId::from_u128(1))?, name, WorldKind::Production),
            Err(SkrifheimError::InvalidWorldName)
        );
        Ok(())
    }
}
