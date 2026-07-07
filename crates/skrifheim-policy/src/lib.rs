#![no_std]
#![forbid(unsafe_code)]

extern crate alloc;

mod context;
mod decision;
mod result;

#[cfg(test)]
mod tests;

pub use context::{AuthorityContext, DeviceContext, SubjectContext, WorkloadContext};
pub use decision::{
    DecisionKind, PlannerDecision, PolicyProof, evaluate_read, evaluate_read_result_set,
    evaluate_read_set, require_allowed,
};
pub use result::{
    AiProcessingEligibility, ConfidenceThreshold, PiiMarker, QueryResultInput,
    RESULT_CLASSIFICATION_INPUT_FIXED_STORAGE_BYTES, RESULT_CLASSIFICATION_INPUT_MAX_ITEMS,
    ResultClassification, SovereigntyScope,
};
