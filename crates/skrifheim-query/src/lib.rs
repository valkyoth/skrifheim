#![no_std]
#![forbid(unsafe_code)]

extern crate alloc;

use alloc::vec::Vec;
use skrifheim_core::{SecurityLabel, WorldId};
use skrifheim_policy::{PlannerDecision, SubjectContext, evaluate_read};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum QueryIntent {
    ReadFacts,
    ExplainCausality,
    SimulateConsequences,
    BuildContextPack,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QueryRequest {
    pub world: WorldId,
    pub intent: QueryIntent,
    pub requested_labels: Vec<SecurityLabel>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QueryPlan {
    pub world: WorldId,
    pub intent: QueryIntent,
    pub decisions: Vec<PlannerDecision>,
}

impl QueryRequest {
    #[must_use]
    pub fn plan(&self, subject: &SubjectContext) -> QueryPlan {
        QueryPlan {
            world: self.world,
            intent: self.intent.clone(),
            decisions: self
                .requested_labels
                .iter()
                .map(|label| evaluate_read(subject, label))
                .collect(),
        }
    }
}

impl QueryPlan {
    #[must_use]
    pub fn has_rejection(&self) -> bool {
        self.decisions
            .iter()
            .any(|decision| matches!(decision, PlannerDecision::Reject { .. }))
    }

    #[must_use]
    pub fn has_redaction(&self) -> bool {
        self.decisions
            .iter()
            .any(|decision| matches!(decision, PlannerDecision::Redact { .. }))
    }

    #[must_use]
    pub fn is_executable(&self) -> bool {
        !self.has_rejection() && !self.has_redaction()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::{string::String, vec};
    use skrifheim_core::{Classification, SkrifheimError};

    fn id<T>(id: Option<T>) -> skrifheim_core::Result<T> {
        id.ok_or(SkrifheimError::InvalidIdentifier)
    }

    #[test]
    fn plan_records_rejection() -> skrifheim_core::Result<()> {
        let request = QueryRequest {
            world: id(WorldId::from_u128(1))?,
            intent: QueryIntent::BuildContextPack,
            requested_labels: vec![SecurityLabel::new(
                Classification::TopSecret,
                Vec::new(),
                Vec::new(),
            )],
        };
        let subject = SubjectContext::new(Classification::Secret, Vec::new(), Vec::new());
        assert!(request.plan(&subject).has_rejection());
        Ok(())
    }

    #[test]
    fn redaction_blocks_executability() -> skrifheim_core::Result<()> {
        let request = QueryRequest {
            world: id(WorldId::from_u128(1))?,
            intent: QueryIntent::ReadFacts,
            requested_labels: vec![SecurityLabel::new(
                Classification::Secret,
                Vec::new(),
                vec![String::from("EU")],
            )],
        };
        let subject = SubjectContext::new(Classification::Secret, Vec::new(), Vec::new());
        let plan = request.plan(&subject);
        assert!(plan.has_redaction());
        assert!(!plan.is_executable());
        Ok(())
    }
}
