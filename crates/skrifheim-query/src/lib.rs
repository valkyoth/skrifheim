#![no_std]
#![forbid(unsafe_code)]

extern crate alloc;

use alloc::vec::Vec;
use skrifheim_core::{Result, SecurityLabel, SkrifheimError, WorldId};
use skrifheim_policy::{
    AuthorityContext, PlannerDecision, PolicyProof, evaluate_read, evaluate_read_set,
};

pub const QUERY_REQUEST_LABEL_MAX_ITEMS: usize = 1024;

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
    world: WorldId,
    intent: QueryIntent,
    proof: PolicyProof,
    decisions: Vec<PlannerDecision>,
}

impl QueryRequest {
    pub fn plan(&self, authority: &AuthorityContext) -> Result<QueryPlan> {
        if self.requested_labels.len() > QUERY_REQUEST_LABEL_MAX_ITEMS {
            return Err(SkrifheimError::InvalidQueryRequest);
        }
        let aggregate_decision = evaluate_read_set(authority, &self.requested_labels);
        Ok(QueryPlan {
            world: self.world,
            intent: self.intent.clone(),
            proof: aggregate_decision.proof().clone(),
            decisions: self
                .requested_labels
                .iter()
                .map(|label| evaluate_read(authority, label))
                .collect(),
        })
    }
}

impl QueryPlan {
    #[must_use]
    pub const fn world(&self) -> WorldId {
        self.world
    }

    #[must_use]
    pub const fn intent(&self) -> &QueryIntent {
        &self.intent
    }

    #[must_use]
    pub const fn proof(&self) -> &PolicyProof {
        &self.proof
    }

    #[must_use]
    pub const fn output_classification(&self) -> skrifheim_core::Classification {
        self.proof.output_classification()
    }

    #[must_use]
    pub fn decisions(&self) -> &[PlannerDecision] {
        &self.decisions
    }

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
    use skrifheim_core::{Classification, DeviceId, SkrifheimError, WorkloadId};
    use skrifheim_policy::{DeviceContext, SubjectContext, WorkloadContext};

    fn id<T>(id: Option<T>) -> skrifheim_core::Result<T> {
        id.ok_or(SkrifheimError::InvalidIdentifier)
    }

    fn authority(clearance: Classification) -> skrifheim_core::Result<AuthorityContext> {
        Ok(AuthorityContext::new(
            SubjectContext::new(clearance, Vec::new(), Vec::new())?,
            DeviceContext::new(
                id(DeviceId::from_u128(2))?,
                clearance,
                Vec::new(),
                Vec::new(),
            )?,
            WorkloadContext::new(
                id(WorkloadId::from_u128(3))?,
                clearance,
                Vec::new(),
                Vec::new(),
            )?,
        ))
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
            )?],
        };
        let authority = authority(Classification::Secret)?;
        let plan = request.plan(&authority)?;
        assert_eq!(plan.world(), id(WorldId::from_u128(1))?);
        assert_eq!(plan.intent(), &QueryIntent::BuildContextPack);
        assert_eq!(plan.decisions().len(), 1);
        assert!(plan.has_rejection());
        assert_eq!(
            plan.output_classification(),
            skrifheim_core::Classification::Public
        );
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
            )?],
        };
        let authority = authority(Classification::Secret)?;
        let plan = request.plan(&authority)?;
        assert!(plan.has_redaction());
        assert!(!plan.is_executable());
        assert_eq!(plan.proof().input_label_count(), 1);
        assert_eq!(plan.output_classification(), Classification::Public);
        Ok(())
    }

    #[test]
    fn plan_escalates_output_classification_for_joins() -> skrifheim_core::Result<()> {
        let request = QueryRequest {
            world: id(WorldId::from_u128(1))?,
            intent: QueryIntent::ReadFacts,
            requested_labels: vec![
                SecurityLabel::new(Classification::Public, Vec::new(), Vec::new())?,
                SecurityLabel::new(Classification::Secret, Vec::new(), Vec::new())?,
            ],
        };
        let authority = authority(Classification::TopSecret)?;
        let plan = request.plan(&authority)?;
        assert_eq!(plan.output_classification(), Classification::Secret);
        assert!(plan.is_executable());
        Ok(())
    }

    #[test]
    fn plan_rejects_too_many_requested_labels() -> skrifheim_core::Result<()> {
        let mut requested_labels = Vec::new();
        for _ in 0..=QUERY_REQUEST_LABEL_MAX_ITEMS {
            requested_labels.push(SecurityLabel::public());
        }
        let request = QueryRequest {
            world: id(WorldId::from_u128(1))?,
            intent: QueryIntent::ReadFacts,
            requested_labels,
        };
        let authority = authority(Classification::TopSecret)?;
        assert_eq!(
            request.plan(&authority),
            Err(SkrifheimError::InvalidQueryRequest)
        );
        Ok(())
    }
}
