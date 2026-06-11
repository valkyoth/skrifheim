use skrifheim_core::{
    AccessDeniedReason, Classification, Result, SecurityLabel, SkrifheimError,
    contains_policy_token_ct,
};

use crate::AuthorityContext;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PlannerDecision {
    Allow {
        proof: PolicyProof,
    },
    Redact {
        reason: AccessDeniedReason,
        proof: PolicyProof,
    },
    Reject {
        reason: AccessDeniedReason,
        proof: PolicyProof,
    },
}

impl PlannerDecision {
    #[must_use]
    pub const fn kind(&self) -> DecisionKind {
        match self {
            Self::Allow { .. } => DecisionKind::Allow,
            Self::Redact { .. } => DecisionKind::Redact,
            Self::Reject { .. } => DecisionKind::Reject,
        }
    }

    #[must_use]
    pub const fn proof(&self) -> &PolicyProof {
        match self {
            Self::Allow { proof } | Self::Redact { proof, .. } | Self::Reject { proof, .. } => {
                proof
            }
        }
    }

    #[must_use]
    pub const fn denial_reason(&self) -> Option<&AccessDeniedReason> {
        match self {
            Self::Allow { .. } => None,
            Self::Redact { reason, .. } | Self::Reject { reason, .. } => Some(reason),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DecisionKind {
    Allow,
    Redact,
    Reject,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PolicyProof {
    decision: DecisionKind,
    input_label_count: usize,
    output_classification: Classification,
}

impl PolicyProof {
    #[must_use]
    pub const fn new(
        decision: DecisionKind,
        input_label_count: usize,
        output_classification: Classification,
    ) -> Self {
        Self {
            decision,
            input_label_count,
            output_classification,
        }
    }

    #[must_use]
    pub const fn decision(&self) -> DecisionKind {
        self.decision
    }

    #[must_use]
    pub const fn input_label_count(&self) -> usize {
        self.input_label_count
    }

    #[must_use]
    pub const fn output_classification(&self) -> Classification {
        self.output_classification
    }
}

#[must_use]
pub fn evaluate_read(authority: &AuthorityContext, label: &SecurityLabel) -> PlannerDecision {
    evaluate_read_set(authority, core::slice::from_ref(label))
}

#[must_use]
pub fn evaluate_read_set(
    authority: &AuthorityContext,
    labels: &[SecurityLabel],
) -> PlannerDecision {
    let output_classification = calculate_output_classification(labels);
    let mut rejected = 0_u8;
    let mut redacted = 0_u8;

    for label in labels {
        match evaluate_label(authority, label) {
            DecisionKind::Allow => {}
            DecisionKind::Redact => redacted = 1,
            DecisionKind::Reject => rejected = 1,
        }
    }

    if rejected == 1 {
        return PlannerDecision::Reject {
            reason: AccessDeniedReason::new(),
            proof: PolicyProof::new(DecisionKind::Reject, labels.len(), output_classification),
        };
    }

    if redacted == 1 {
        return PlannerDecision::Redact {
            reason: AccessDeniedReason::new(),
            proof: PolicyProof::new(DecisionKind::Redact, labels.len(), output_classification),
        };
    }

    PlannerDecision::Allow {
        proof: PolicyProof::new(DecisionKind::Allow, labels.len(), output_classification),
    }
}

#[must_use]
pub fn calculate_output_classification(labels: &[SecurityLabel]) -> Classification {
    let mut output_classification = Classification::Public;
    for label in labels {
        if label.classification() > output_classification {
            output_classification = label.classification();
        }
    }
    output_classification
}

fn evaluate_label(authority: &AuthorityContext, label: &SecurityLabel) -> DecisionKind {
    let label_classification = label.classification();
    let mut clearance_allowed = 1_u8;
    clearance_allowed &= authority
        .subject()
        .clearance()
        .dominates(label_classification) as u8;
    clearance_allowed &= authority
        .device()
        .clearance()
        .dominates(label_classification) as u8;
    clearance_allowed &= authority
        .workload()
        .clearance()
        .dominates(label_classification) as u8;

    let mut compartment_allowed = 1_u8;
    for compartment in label.compartments() {
        compartment_allowed &=
            contains_policy_token_ct(authority.subject().compartments(), compartment) as u8;
        compartment_allowed &=
            contains_policy_token_ct(authority.device().compartments(), compartment) as u8;
        compartment_allowed &=
            contains_policy_token_ct(authority.workload().compartments(), compartment) as u8;
    }

    let mut releasability_allowed = 1_u8;
    for releasability in label.releasable_to() {
        releasability_allowed &=
            contains_policy_token_ct(authority.subject().releasable_to(), releasability) as u8;
        releasability_allowed &=
            contains_policy_token_ct(authority.device().releasable_to(), releasability) as u8;
        releasability_allowed &=
            contains_policy_token_ct(authority.workload().releasable_to(), releasability) as u8;
    }

    let mut rejected = 0_u8;
    rejected |= (clearance_allowed == 0) as u8;
    rejected |= (compartment_allowed == 0) as u8;

    if rejected == 1 {
        return DecisionKind::Reject;
    }

    if releasability_allowed == 0 {
        return DecisionKind::Redact;
    }

    DecisionKind::Allow
}

pub fn require_allowed(decision: PlannerDecision) -> Result<()> {
    match decision {
        PlannerDecision::Allow { .. } => Ok(()),
        PlannerDecision::Redact { reason, .. } | PlannerDecision::Reject { reason, .. } => {
            Err(SkrifheimError::PolicyDenied(reason))
        }
    }
}
