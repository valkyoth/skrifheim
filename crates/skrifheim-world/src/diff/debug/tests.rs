use super::*;
use crate::{WORLD_FACT_LIST_MAX_ITEMS, WorldConflictKind};
use alloc::{format, vec};
use skrifheim_core::{FactId, Result, SkrifheimError, WorldId};

#[test]
fn diff_and_preflight_debug_is_constant_size_for_empty_and_full_lists() -> Result<()> {
    let world = WorldId::from_u128(987654).ok_or(SkrifheimError::InvalidIdentifier)?;
    let fact = FactId::from_u128(123456).ok_or(SkrifheimError::InvalidIdentifier)?;
    for kind in [
        WorldConflictKind::AddedAndHiddenSameFact,
        WorldConflictKind::ReintroducesParentHiddenFact,
    ] {
        let conflict = WorldConflict {
            kind,
            fact_id: fact,
        };
        assert_eq!(format!("{conflict:?}"), "WorldConflict(<redacted>)");
        assert_eq!(format!("{conflict:#?}"), "WorldConflict(<redacted>)");
        for size in [0, 1, WORLD_FACT_LIST_MAX_ITEMS] {
            let diff = WorldDiff {
                from: world,
                to: world,
                added: vec![fact; size],
                hidden: vec![fact; size],
            };
            assert_eq!(format!("{diff:?}"), "WorldDiff(<redacted>)");
            assert_eq!(format!("{diff:#?}"), "WorldDiff(<redacted>)");
            let promotion = PromotionPreflight {
                diff,
                conflicts: vec![conflict; size],
                storage_validated: false,
            };
            assert_eq!(format!("{promotion:?}"), "PromotionPreflight(<redacted>)");
            assert_eq!(format!("{promotion:#?}"), "PromotionPreflight(<redacted>)");
            let rollback = RollbackPreflight {
                from: world,
                to: world,
                reverts_added: promotion.diff.added,
                restores_hidden: promotion.diff.hidden,
                conflicts: promotion.conflicts,
                storage_validated: false,
            };
            assert_eq!(format!("{rollback:?}"), "RollbackPreflight(<redacted>)");
            assert_eq!(format!("{rollback:#?}"), "RollbackPreflight(<redacted>)");
        }
    }
    Ok(())
}
