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
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;
    use skrifheim_core::Classification;

    #[test]
    fn plan_records_rejection() {
        let request = QueryRequest {
            world: WorldId(1),
            intent: QueryIntent::BuildContextPack,
            requested_labels: vec![SecurityLabel {
                classification: Classification::TopSecret,
                compartments: Vec::new(),
                releasable_to: Vec::new(),
            }],
        };
        let subject = SubjectContext {
            clearance: Classification::Secret,
            compartments: Vec::new(),
            releasable_to: Vec::new(),
        };
        assert!(request.plan(&subject).has_rejection());
    }
}
