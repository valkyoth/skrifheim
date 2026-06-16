#![no_std]
#![forbid(unsafe_code)]

extern crate alloc;

use alloc::vec::Vec;
use core::fmt;
use skrifheim_core::{Result, SecurityLabel, SkrifheimError, WorldId};
use skrifheim_policy::{
    AuthorityContext, PolicyProof, QueryResultInput,
    RESULT_CLASSIFICATION_INPUT_FIXED_STORAGE_BYTES, RESULT_CLASSIFICATION_INPUT_MAX_ITEMS,
    ResultClassification, evaluate_read_result_set,
};

pub const QUERY_REQUEST_LABEL_MAX_ITEMS: usize = RESULT_CLASSIFICATION_INPUT_MAX_ITEMS;
pub const QUERY_REQUEST_INPUT_MEMORY_BUDGET_BYTES: usize =
    QUERY_REQUEST_LABEL_MAX_ITEMS * RESULT_CLASSIFICATION_INPUT_FIXED_STORAGE_BYTES;
const _: () = assert!(QUERY_REQUEST_INPUT_MEMORY_BUDGET_BYTES <= 2 * 1024 * 1024);

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum QueryIntent {
    ReadFacts,
    ExplainCausality,
    SimulateConsequences,
    BuildContextPack,
}

#[derive(Clone)]
pub struct QueryRequest {
    world: WorldId,
    intent: QueryIntent,
    result_inputs: Vec<QueryResultInput>,
}

#[derive(Clone)]
pub struct QueryPlan {
    world: WorldId,
    intent: QueryIntent,
    proof: PolicyProof,
}

impl QueryRequest {
    pub fn new(
        world: WorldId,
        intent: QueryIntent,
        requested_labels: Vec<SecurityLabel>,
    ) -> Result<Self> {
        if requested_labels.is_empty() || requested_labels.len() > QUERY_REQUEST_LABEL_MAX_ITEMS {
            return Err(SkrifheimError::InvalidQueryRequest);
        }
        let mut result_inputs = Vec::with_capacity(requested_labels.len());
        for label in requested_labels {
            result_inputs.push(QueryResultInput::label_only(label));
        }
        Ok(Self {
            world,
            intent,
            result_inputs,
        })
    }

    pub fn with_result_inputs(
        world: WorldId,
        intent: QueryIntent,
        result_inputs: Vec<QueryResultInput>,
    ) -> Result<Self> {
        if result_inputs.is_empty() || result_inputs.len() > QUERY_REQUEST_LABEL_MAX_ITEMS {
            return Err(SkrifheimError::InvalidQueryRequest);
        }
        let result_inputs = exact_capacity_result_inputs(result_inputs);
        Ok(Self {
            world,
            intent,
            result_inputs,
        })
    }

    #[must_use]
    pub const fn world(&self) -> WorldId {
        self.world
    }

    #[must_use]
    pub const fn intent(&self) -> &QueryIntent {
        &self.intent
    }

    #[must_use]
    pub fn requested_label_count(&self) -> usize {
        self.result_inputs.len()
    }

    #[must_use]
    pub fn result_inputs(&self) -> &[QueryResultInput] {
        &self.result_inputs
    }

    pub fn plan(&self, authority: &AuthorityContext) -> Result<QueryPlan> {
        if self.result_inputs.is_empty() || self.result_inputs.len() > QUERY_REQUEST_LABEL_MAX_ITEMS
        {
            return Err(SkrifheimError::InvalidQueryRequest);
        }
        let aggregate_decision = evaluate_read_result_set(authority, &self.result_inputs)?;
        Ok(QueryPlan {
            world: self.world,
            intent: self.intent.clone(),
            proof: aggregate_decision.proof().clone(),
        })
    }
}

impl fmt::Debug for QueryRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("QueryRequest")
            .field("world", &"<redacted>")
            .field("intent", &self.intent)
            .field("result_input_count", &self.result_inputs.len())
            .field("result_inputs", &"<redacted>")
            .finish()
    }
}

fn exact_capacity_result_inputs(inputs: Vec<QueryResultInput>) -> Vec<QueryResultInput> {
    let mut exact = Vec::with_capacity(inputs.len());
    for input in inputs {
        exact.push(input);
    }
    exact
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
    pub const fn result_classification(&self) -> &ResultClassification {
        self.proof.result_classification()
    }

    #[must_use]
    pub fn has_rejection(&self) -> bool {
        matches!(
            self.proof.decision(),
            skrifheim_policy::DecisionKind::Reject
        )
    }

    #[must_use]
    pub fn has_redaction(&self) -> bool {
        matches!(
            self.proof.decision(),
            skrifheim_policy::DecisionKind::Redact
        )
    }

    #[must_use]
    pub fn is_executable(&self) -> bool {
        !self.has_rejection() && !self.has_redaction()
    }
}

impl fmt::Debug for QueryPlan {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("QueryPlan")
            .field("world", &"<redacted>")
            .field("intent", &self.intent)
            .field("decision", &self.proof.decision())
            .field("proof", &"<redacted>")
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::{string::String, vec};
    use skrifheim_core::{Classification, DeviceId, SkrifheimError, WorkloadId};
    use skrifheim_policy::{
        AiProcessingEligibility, ConfidenceThreshold, DeviceContext, PiiMarker, QueryResultInput,
        SubjectContext, WorkloadContext,
    };

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
        let request = QueryRequest::new(
            id(WorldId::from_u128(1))?,
            QueryIntent::BuildContextPack,
            vec![SecurityLabel::new(
                Classification::TopSecret,
                Vec::new(),
                Vec::new(),
            )?],
        )?;
        let authority = authority(Classification::Secret)?;
        let plan = request.plan(&authority)?;
        assert_eq!(plan.world(), id(WorldId::from_u128(1))?);
        assert_eq!(plan.intent(), &QueryIntent::BuildContextPack);
        assert!(plan.has_rejection());
        assert_eq!(
            plan.output_classification(),
            skrifheim_core::Classification::Public
        );
        Ok(())
    }

    #[test]
    fn redaction_blocks_executability() -> skrifheim_core::Result<()> {
        let request = QueryRequest::new(
            id(WorldId::from_u128(1))?,
            QueryIntent::ReadFacts,
            vec![SecurityLabel::new(
                Classification::Secret,
                Vec::new(),
                vec![String::from("EU")],
            )?],
        )?;
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
        let request = QueryRequest::new(
            id(WorldId::from_u128(1))?,
            QueryIntent::ReadFacts,
            vec![
                SecurityLabel::new(Classification::Public, Vec::new(), Vec::new())?,
                SecurityLabel::new(Classification::Secret, Vec::new(), Vec::new())?,
            ],
        )?;
        let authority = authority(Classification::TopSecret)?;
        let plan = request.plan(&authority)?;
        assert_eq!(plan.output_classification(), Classification::Secret);
        assert!(plan.is_executable());
        Ok(())
    }

    #[test]
    fn plan_propagates_result_classification_metadata() -> skrifheim_core::Result<()> {
        let request = QueryRequest::with_result_inputs(
            id(WorldId::from_u128(1))?,
            QueryIntent::BuildContextPack,
            vec![
                QueryResultInput::new(
                    SecurityLabel::new(Classification::Restricted, Vec::new(), Vec::new())?,
                    vec![String::from("eu")],
                    PiiMarker::NoPii,
                    AiProcessingEligibility::Eligible,
                    Some(ConfidenceThreshold::new(600)?),
                )?,
                QueryResultInput::new(
                    SecurityLabel::new(Classification::Secret, Vec::new(), Vec::new())?,
                    vec![String::from("se")],
                    PiiMarker::ContainsPii,
                    AiProcessingEligibility::NotEligible,
                    Some(ConfidenceThreshold::new(800)?),
                )?,
            ],
        )?;
        let plan = request.plan(&authority(Classification::TopSecret)?)?;
        let result = plan.result_classification();

        assert_eq!(plan.output_classification(), Classification::Secret);
        assert_eq!(result.sovereignty().len(), 2);
        assert!(result.sovereignty().contains("EU"));
        assert!(result.sovereignty().contains("SE"));
        assert_eq!(result.pii(), PiiMarker::ContainsPii);
        assert_eq!(result.ai_processing(), AiProcessingEligibility::NotEligible);
        assert_eq!(
            result.confidence_threshold(),
            Some(ConfidenceThreshold::new(800)?)
        );
        Ok(())
    }

    #[test]
    fn non_allow_plan_masks_result_classification_metadata() -> skrifheim_core::Result<()> {
        let request = QueryRequest::with_result_inputs(
            id(WorldId::from_u128(1))?,
            QueryIntent::BuildContextPack,
            vec![QueryResultInput::new(
                SecurityLabel::new(Classification::TopSecret, Vec::new(), Vec::new())?,
                vec![String::from("se")],
                PiiMarker::ContainsPii,
                AiProcessingEligibility::NotEligible,
                Some(ConfidenceThreshold::new(900)?),
            )?],
        )?;
        let plan = request.plan(&authority(Classification::Secret)?)?;
        let result = plan.result_classification();

        assert!(plan.has_rejection());
        assert_eq!(result.output_classification(), Classification::Public);
        assert_eq!(result.sovereignty().len(), 0);
        assert_eq!(result.pii(), PiiMarker::NoPii);
        assert_eq!(result.ai_processing(), AiProcessingEligibility::Eligible);
        assert_eq!(result.confidence_threshold(), None);
        Ok(())
    }

    #[test]
    fn plan_rejects_too_many_requested_labels() -> skrifheim_core::Result<()> {
        let mut requested_labels = Vec::new();
        for _ in 0..=QUERY_REQUEST_LABEL_MAX_ITEMS {
            requested_labels.push(SecurityLabel::public());
        }
        assert!(matches!(
            QueryRequest::new(
                id(WorldId::from_u128(1))?,
                QueryIntent::ReadFacts,
                requested_labels,
            ),
            Err(SkrifheimError::InvalidQueryRequest)
        ));
        Ok(())
    }

    #[test]
    fn plan_rejects_empty_requested_labels() -> skrifheim_core::Result<()> {
        let world = id(WorldId::from_u128(1))?;
        assert!(matches!(
            QueryRequest::new(world, QueryIntent::ReadFacts, Vec::new()),
            Err(SkrifheimError::InvalidQueryRequest)
        ));
        let request = QueryRequest {
            world,
            intent: QueryIntent::ReadFacts,
            result_inputs: Vec::new(),
        };
        assert!(matches!(
            request.plan(&authority(Classification::TopSecret)?),
            Err(SkrifheimError::InvalidQueryRequest)
        ));
        Ok(())
    }

    #[test]
    fn label_only_constructor_stores_result_inputs_once() -> skrifheim_core::Result<()> {
        let request = QueryRequest::new(
            id(WorldId::from_u128(1))?,
            QueryIntent::ReadFacts,
            vec![
                SecurityLabel::new(Classification::Public, Vec::new(), Vec::new())?,
                SecurityLabel::new(Classification::Secret, Vec::new(), Vec::new())?,
            ],
        )?;

        assert_eq!(request.requested_label_count(), 2);
        assert_eq!(request.result_inputs().len(), 2);
        assert_eq!(
            request.result_inputs()[1].label().classification(),
            Classification::Secret
        );
        Ok(())
    }

    #[test]
    fn result_input_constructor_drops_excess_vec_capacity() -> skrifheim_core::Result<()> {
        let mut inputs = Vec::with_capacity(QUERY_REQUEST_LABEL_MAX_ITEMS);
        inputs.push(QueryResultInput::new(
            SecurityLabel::new(Classification::Secret, Vec::new(), Vec::new())?,
            Vec::new(),
            PiiMarker::NoPii,
            AiProcessingEligibility::Eligible,
            None,
        )?);

        let request = QueryRequest::with_result_inputs(
            id(WorldId::from_u128(1))?,
            QueryIntent::ReadFacts,
            inputs,
        )?;

        assert_eq!(request.result_inputs.len(), 1);
        assert_eq!(request.result_inputs.capacity(), 1);
        Ok(())
    }

    #[test]
    fn query_result_input_memory_budget_is_explicit() {
        assert_eq!(
            QUERY_REQUEST_INPUT_MEMORY_BUDGET_BYTES,
            QUERY_REQUEST_LABEL_MAX_ITEMS * RESULT_CLASSIFICATION_INPUT_FIXED_STORAGE_BYTES
        );
    }

    #[test]
    fn debug_redacts_query_request_and_plan_metadata() -> skrifheim_core::Result<()> {
        let request = QueryRequest::with_result_inputs(
            id(WorldId::from_u128(1))?,
            QueryIntent::BuildContextPack,
            vec![QueryResultInput::new(
                SecurityLabel::new(Classification::Secret, Vec::new(), Vec::new())?,
                vec![String::from("se")],
                PiiMarker::ContainsPii,
                AiProcessingEligibility::NotEligible,
                Some(ConfidenceThreshold::new(900)?),
            )?],
        )?;
        let request_debug = alloc::format!("{request:?}");
        assert!(!request_debug.contains("Secret"));
        assert!(!request_debug.contains("SE"));
        assert!(!request_debug.contains("ContainsPii"));
        assert!(!request_debug.contains("NotEligible"));
        assert!(!request_debug.contains("900"));

        let plan = request.plan(&authority(Classification::TopSecret)?)?;
        let plan_debug = alloc::format!("{plan:?}");
        assert!(!plan_debug.contains("Secret"));
        assert!(!plan_debug.contains("SE"));
        assert!(!plan_debug.contains("ContainsPii"));
        assert!(!plan_debug.contains("NotEligible"));
        assert!(!plan_debug.contains("900"));
        Ok(())
    }
}
