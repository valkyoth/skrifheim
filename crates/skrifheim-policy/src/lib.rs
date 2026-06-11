#![no_std]
#![forbid(unsafe_code)]

extern crate alloc;

mod context;
mod decision;

#[cfg(test)]
mod tests;

pub use context::{AuthorityContext, DeviceContext, SubjectContext, WorkloadContext};
pub use decision::{
    DecisionKind, PlannerDecision, PolicyProof, calculate_output_classification, evaluate_read,
    evaluate_read_set, require_allowed,
};
