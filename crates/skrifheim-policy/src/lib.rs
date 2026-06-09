#![no_std]
#![forbid(unsafe_code)]

extern crate alloc;

use alloc::{format, string::String, vec::Vec};
use skrifheim_core::{Classification, Result, SecurityLabel, SkrifheimError};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SubjectContext {
    pub clearance: Classification,
    pub compartments: Vec<String>,
    pub releasable_to: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PlannerDecision {
    Allow,
    Redact { reason: String },
    Reject { reason: String },
}

#[must_use]
pub fn evaluate_read(subject: &SubjectContext, label: &SecurityLabel) -> PlannerDecision {
    if !subject.clearance.dominates(label.classification) {
        return PlannerDecision::Reject {
            reason: String::from("subject clearance is below fact classification"),
        };
    }

    for compartment in &label.compartments {
        if !subject.compartments.contains(compartment) {
            return PlannerDecision::Reject {
                reason: format!("subject lacks compartment {compartment}"),
            };
        }
    }

    for releasability in &label.releasable_to {
        if !subject.releasable_to.contains(releasability) {
            return PlannerDecision::Redact {
                reason: format!("subject lacks releasability {releasability}"),
            };
        }
    }

    PlannerDecision::Allow
}

pub fn require_allowed(decision: PlannerDecision) -> Result<()> {
    match decision {
        PlannerDecision::Allow => Ok(()),
        PlannerDecision::Redact { reason } | PlannerDecision::Reject { reason } => {
            Err(SkrifheimError::PolicyDenied(reason))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::{string::String, vec};

    #[test]
    fn read_requires_clearance() {
        let subject = SubjectContext {
            clearance: Classification::Restricted,
            compartments: vec![String::from("A")],
            releasable_to: vec![String::from("EU")],
        };
        let label = SecurityLabel {
            classification: Classification::Secret,
            compartments: vec![String::from("A")],
            releasable_to: vec![String::from("EU")],
        };
        assert!(matches!(
            evaluate_read(&subject, &label),
            PlannerDecision::Reject { .. }
        ));
    }

    #[test]
    fn missing_releasability_redacts_instead_of_allows() {
        let subject = SubjectContext {
            clearance: Classification::Secret,
            compartments: vec![String::from("A")],
            releasable_to: Vec::new(),
        };
        let label = SecurityLabel {
            classification: Classification::Secret,
            compartments: vec![String::from("A")],
            releasable_to: vec![String::from("EU")],
        };
        assert!(matches!(
            evaluate_read(&subject, &label),
            PlannerDecision::Redact { .. }
        ));
    }
}
