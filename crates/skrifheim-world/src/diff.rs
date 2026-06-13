use alloc::{collections::BTreeSet, vec::Vec};
use skrifheim_core::{FactId, Result, SkrifheimError, WorldId};

use crate::World;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorldConflictKind {
    AddedAndHiddenSameFact,
    ReintroducesParentHiddenFact,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorldConflict {
    pub kind: WorldConflictKind,
    pub fact_id: FactId,
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
        if from.tenant_id() != to.tenant_id()
            || (from.id() != to.id() && to.parent() != Some(from.id()))
        {
            return Err(SkrifheimError::InvalidWorldDiff);
        }
        Ok(Self {
            from: from.id(),
            to: to.id(),
            added: set_difference(to.added_facts(), from.added_facts()),
            hidden: set_difference(to.hidden_facts(), from.hidden_facts()),
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PromotionPreflight {
    pub diff: WorldDiff,
    pub conflicts: Vec<WorldConflict>,
}

impl PromotionPreflight {
    /// Builds a parent-vs-child promotion preflight.
    ///
    /// This scaffold does not inspect transitive ancestry. Future storage-backed
    /// promotion must validate the full ancestor chain before executing a merge.
    pub fn between(parent: &World, candidate: &World) -> Result<Self> {
        Ok(Self {
            diff: WorldDiff::between(parent, candidate)?,
            conflicts: collect_conflicts(parent, candidate),
        })
    }

    #[must_use]
    pub fn can_promote(&self) -> bool {
        self.conflicts.is_empty()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RollbackPreflight {
    pub from: WorldId,
    pub to: WorldId,
    pub reverts_added: Vec<FactId>,
    pub restores_hidden: Vec<FactId>,
    pub conflicts: Vec<WorldConflict>,
}

impl RollbackPreflight {
    /// Builds a parent-vs-child rollback preflight.
    ///
    /// This scaffold does not inspect transitive ancestry. Future storage-backed
    /// rollback must validate the full ancestor chain before executing a
    /// rollback.
    pub fn from_child(parent: &World, child: &World) -> Result<Self> {
        let diff = WorldDiff::between(parent, child)?;
        Ok(Self {
            from: child.id(),
            to: parent.id(),
            reverts_added: diff.added,
            restores_hidden: diff.hidden,
            conflicts: collect_conflicts(parent, child),
        })
    }

    #[must_use]
    pub fn can_rollback(&self) -> bool {
        self.conflicts.is_empty()
    }
}

fn collect_conflicts(parent: &World, candidate: &World) -> Vec<WorldConflict> {
    let added = fact_set(candidate.added_facts());
    let hidden = fact_set(candidate.hidden_facts());
    let parent_hidden = fact_set(parent.hidden_facts());
    let mut conflicts = Vec::new();

    for fact_id in added.intersection(&hidden) {
        conflicts.push(WorldConflict {
            kind: WorldConflictKind::AddedAndHiddenSameFact,
            fact_id: *fact_id,
        });
    }

    for fact_id in added.intersection(&parent_hidden) {
        conflicts.push(WorldConflict {
            kind: WorldConflictKind::ReintroducesParentHiddenFact,
            fact_id: *fact_id,
        });
    }

    conflicts
}

fn set_difference(values: &[FactId], existing: &[FactId]) -> Vec<FactId> {
    let existing = fact_set(existing);
    fact_set(values)
        .into_iter()
        .filter(|fact_id| !existing.contains(fact_id))
        .collect()
}

fn fact_set(values: &[FactId]) -> BTreeSet<FactId> {
    values.iter().copied().collect()
}
