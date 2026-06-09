#![no_std]
#![forbid(unsafe_code)]

extern crate alloc;

use alloc::{string::String, vec::Vec};
use skrifheim_core::{FactId, WorldId};

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
    #[must_use]
    pub fn between(from: &World, to: &World) -> Self {
        Self {
            from: from.id,
            to: to.id,
            added: to.added_facts.clone(),
            hidden: to.hidden_facts.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;

    #[test]
    fn fork_keeps_parent_identity() {
        let production = World {
            id: WorldId(1),
            name: String::from("production"),
            kind: WorldKind::Production,
            parent: None,
            added_facts: Vec::new(),
            hidden_facts: Vec::new(),
        };
        let draft = production.fork(WorldId(2), "draft", WorldKind::Simulation);
        assert_eq!(draft.parent, Some(WorldId(1)));
    }

    #[test]
    fn duplicate_fact_adds_are_idempotent() {
        let mut world = World {
            id: WorldId(1),
            name: String::from("production"),
            kind: WorldKind::Production,
            parent: None,
            added_facts: Vec::new(),
            hidden_facts: Vec::new(),
        };
        world.add_fact(FactId(7));
        world.add_fact(FactId(7));
        assert_eq!(world.added_facts, vec![FactId(7)]);
    }
}
