use alloc::vec::Vec;
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
    storage_validated: bool,
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
            storage_validated: false,
        })
    }

    #[must_use]
    pub fn can_promote(&self) -> bool {
        self.conflicts.is_empty()
    }

    /// Returns whether this preflight's ancestry chain was validated against
    /// durable storage.
    ///
    /// In-memory scaffold preflights are useful for conflict discovery but must
    /// not authorize merge execution.
    #[must_use]
    pub const fn is_storage_validated(&self) -> bool {
        self.storage_validated
    }

    pub fn require_storage_validated(&self) -> Result<()> {
        if !self.storage_validated {
            return Err(SkrifheimError::InvalidWorldDiff);
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RollbackPreflight {
    pub from: WorldId,
    pub to: WorldId,
    pub reverts_added: Vec<FactId>,
    pub restores_hidden: Vec<FactId>,
    pub conflicts: Vec<WorldConflict>,
    storage_validated: bool,
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
            storage_validated: false,
        })
    }

    #[must_use]
    pub fn can_rollback(&self) -> bool {
        self.conflicts.is_empty()
    }

    /// Returns whether this preflight's ancestry chain was validated against
    /// durable storage.
    ///
    /// In-memory scaffold preflights are useful for conflict discovery but must
    /// not authorize rollback execution.
    #[must_use]
    pub const fn is_storage_validated(&self) -> bool {
        self.storage_validated
    }

    pub fn require_storage_validated(&self) -> Result<()> {
        if !self.storage_validated {
            return Err(SkrifheimError::InvalidWorldDiff);
        }
        Ok(())
    }
}

fn collect_conflicts(parent: &World, candidate: &World) -> Vec<WorldConflict> {
    let mut conflicts = Vec::new();

    collect_intersections(
        candidate.added_facts(),
        candidate.hidden_facts(),
        WorldConflictKind::AddedAndHiddenSameFact,
        &mut conflicts,
    );
    collect_intersections(
        candidate.added_facts(),
        parent.hidden_facts(),
        WorldConflictKind::ReintroducesParentHiddenFact,
        &mut conflicts,
    );

    conflicts
}

fn collect_intersections(
    left: &[FactId],
    right: &[FactId],
    kind: WorldConflictKind,
    conflicts: &mut Vec<WorldConflict>,
) {
    let mut left_index = 0;
    let mut right_index = 0;
    while left_index < left.len() && right_index < right.len() {
        let left_fact = left[left_index];
        let right_fact = right[right_index];
        if left_fact < right_fact {
            left_index += 1;
            continue;
        }
        if right_fact < left_fact {
            right_index += 1;
            continue;
        }
        conflicts.push(WorldConflict {
            kind,
            fact_id: left_fact,
        });
        left_index += 1;
        right_index += 1;
    }
}

fn set_difference(values: &[FactId], existing: &[FactId]) -> Vec<FactId> {
    let mut result = Vec::new();
    let mut existing_index = 0;

    for value in values {
        while existing_index < existing.len() && existing[existing_index] < *value {
            existing_index += 1;
        }
        if existing_index == existing.len() || existing[existing_index] != *value {
            result.push(*value);
        }
    }

    result
}
