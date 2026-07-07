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
        self.result_input_count()
    }

    #[must_use]
    pub fn result_input_count(&self) -> usize {
        self.result_inputs.len()
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
            .field("intent", &"<redacted>")
            .field("result_input_count", &"<redacted>")
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
            .field("intent", &"<redacted>")
            .field("decision", &"<redacted>")
            .field("proof", &"<redacted>")
            .finish()
    }
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
