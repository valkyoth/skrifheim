#![no_std]
#![forbid(unsafe_code)]

extern crate alloc;

use alloc::{collections::BTreeSet, string::String, vec::Vec};
use skrifheim_core::{Classification, Result, SecurityLabel, SkrifheimError};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SubjectContext {
    clearance: Classification,
    compartments: BTreeSet<String>,
    releasable_to: BTreeSet<String>,
}

impl SubjectContext {
    #[must_use]
    pub fn new(
        clearance: Classification,
        compartments: Vec<String>,
        releasable_to: Vec<String>,
    ) -> Self {
        Self {
            clearance,
            compartments: canonical_set(compartments),
            releasable_to: canonical_set(releasable_to),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PlannerDecision {
    Allow,
    Redact { reason: String },
    Reject { reason: String },
}

#[must_use]
pub fn evaluate_read(subject: &SubjectContext, label: &SecurityLabel) -> PlannerDecision {
    if !subject.clearance.dominates(label.classification()) {
        return PlannerDecision::Reject {
            reason: access_denied(),
        };
    }

    for compartment in label.compartments() {
        if !subject.compartments.contains(compartment) {
            return PlannerDecision::Reject {
                reason: access_denied(),
            };
        }
    }

    for releasability in label.releasable_to() {
        if !subject.releasable_to.contains(releasability) {
            return PlannerDecision::Redact {
                reason: access_denied(),
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

fn canonical_set(values: Vec<String>) -> BTreeSet<String> {
    values.into_iter().map(canonical_token).collect()
}

fn canonical_token(mut value: String) -> String {
    value.make_ascii_uppercase();
    value
}

fn access_denied() -> String {
    String::from("access denied")
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::{string::String, vec};

    #[test]
    fn read_requires_clearance() {
        let subject = SubjectContext::new(
            Classification::Restricted,
            vec![String::from("A")],
            vec![String::from("EU")],
        );
        let label = SecurityLabel::new(
            Classification::Secret,
            vec![String::from("A")],
            vec![String::from("EU")],
        );
        assert!(matches!(
            evaluate_read(&subject, &label),
            PlannerDecision::Reject { .. }
        ));
    }

    #[test]
    fn missing_releasability_redacts_instead_of_allows() {
        let subject =
            SubjectContext::new(Classification::Secret, vec![String::from("A")], Vec::new());
        let label = SecurityLabel::new(
            Classification::Secret,
            vec![String::from("A")],
            vec![String::from("EU")],
        );
        assert!(matches!(
            evaluate_read(&subject, &label),
            PlannerDecision::Redact { .. }
        ));
    }

    #[test]
    fn denial_reasons_do_not_disclose_compartment_names() {
        let subject = SubjectContext::new(Classification::Secret, Vec::new(), Vec::new());
        let label = SecurityLabel::new(
            Classification::Secret,
            vec![String::from("SECRET-COMPARTMENT")],
            Vec::new(),
        );
        assert_eq!(
            evaluate_read(&subject, &label),
            PlannerDecision::Reject {
                reason: String::from("access denied")
            }
        );
    }
}
