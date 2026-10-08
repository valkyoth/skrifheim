use super::{PromotionPreflight, RollbackPreflight, WorldConflict, WorldDiff};
use core::fmt;

impl fmt::Debug for WorldConflict {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("WorldConflict(<redacted>)")
    }
}

impl fmt::Debug for WorldDiff {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("WorldDiff(<redacted>)")
    }
}

impl fmt::Debug for PromotionPreflight {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("PromotionPreflight(<redacted>)")
    }
}

impl fmt::Debug for RollbackPreflight {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("RollbackPreflight(<redacted>)")
    }
}

#[cfg(test)]
mod tests;
