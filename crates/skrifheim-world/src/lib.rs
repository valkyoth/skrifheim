#![no_std]
#![forbid(unsafe_code)]

extern crate alloc;

use alloc::{string::String, vec::Vec};
use skrifheim_core::{FactId, Result, SkrifheimError, WorldId};

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
    pub id: WorldId,
    pub name: String,
    pub kind: WorldKind,
    pub parent: Option<WorldId>,
    pub added_facts: Vec<FactId>,
    pub hidden_facts: Vec<FactId>,
}

impl World {
    #[must_use]
    pub fn fork(&self, id: WorldId, name: impl Into<String>, kind: WorldKind) -> Self {
        Self {
            id,
            name: name.into(),
            kind,
            parent: Some(self.id),
            added_facts: Vec::new(),
            hidden_facts: Vec::new(),
        }
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
        let production = World {
            id: id(WorldId::from_u128(1))?,
            name: String::from("production"),
            kind: WorldKind::Production,
            parent: None,
            added_facts: Vec::new(),
            hidden_facts: Vec::new(),
        };
        let draft = production.fork(id(WorldId::from_u128(2))?, "draft", WorldKind::Simulation);
        assert_eq!(draft.parent, Some(id(WorldId::from_u128(1))?));
        Ok(())
    }

    #[test]
    fn duplicate_fact_adds_are_idempotent() -> Result<()> {
        let mut world = World {
            id: id(WorldId::from_u128(1))?,
            name: String::from("production"),
            kind: WorldKind::Production,
            parent: None,
            added_facts: Vec::new(),
            hidden_facts: Vec::new(),
        };
        world.add_fact(id(FactId::from_u128(7))?);
        world.add_fact(id(FactId::from_u128(7))?);
        assert_eq!(world.added_facts, vec![id(FactId::from_u128(7))?]);
        Ok(())
    }

    #[test]
    fn diff_requires_direct_child_relationship() -> Result<()> {
        let production = World {
            id: id(WorldId::from_u128(1))?,
            name: String::from("production"),
            kind: WorldKind::Production,
            parent: None,
            added_facts: vec![id(FactId::from_u128(7))?],
            hidden_facts: Vec::new(),
        };
        let unrelated = World {
            id: id(WorldId::from_u128(2))?,
            name: String::from("unrelated"),
            kind: WorldKind::Simulation,
            parent: None,
            added_facts: Vec::new(),
            hidden_facts: Vec::new(),
        };
        assert_eq!(
            WorldDiff::between(&production, &unrelated),
            Err(SkrifheimError::InvalidWorldDiff)
        );
        Ok(())
    }

    #[test]
    fn diff_returns_delta_against_parent() -> Result<()> {
        let mut production = World {
            id: id(WorldId::from_u128(1))?,
            name: String::from("production"),
            kind: WorldKind::Production,
            parent: None,
            added_facts: Vec::new(),
            hidden_facts: Vec::new(),
        };
        production.add_fact(id(FactId::from_u128(7))?);
        let mut child = production.fork(id(WorldId::from_u128(2))?, "child", WorldKind::Simulation);
        child.add_fact(id(FactId::from_u128(7))?);
        child.add_fact(id(FactId::from_u128(8))?);
        let diff = WorldDiff::between(&production, &child)?;
        assert_eq!(diff.added, vec![id(FactId::from_u128(8))?]);
        Ok(())
    }
}
