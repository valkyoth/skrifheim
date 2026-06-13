use alloc::{collections::BTreeSet, string::String};
use skrifheim_core::{
    AccessDeniedReason, Classification, POLICY_TOKEN_SET_MAX_ITEMS, Result, SecurityLabel,
    SkrifheimError, contains_policy_token_ct,
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
            proof: PolicyProof::new(DecisionKind::Reject, labels.len(), Classification::Public),
        };
    }

    if redacted == 1 {
        return PlannerDecision::Redact {
            reason: AccessDeniedReason::new(),
            proof: PolicyProof::new(DecisionKind::Redact, labels.len(), Classification::Public),
        };
    }

    let output_classification = calculate_output_classification(labels);
    PlannerDecision::Allow {
        proof: PolicyProof::new(DecisionKind::Allow, labels.len(), output_classification),
    }
}

#[must_use]
pub(crate) fn calculate_output_classification(labels: &[SecurityLabel]) -> Classification {
    let mut output_classification = Classification::Public;
    for label in labels {
        if label.classification() > output_classification {
            output_classification = label.classification();
        }
    }
    output_classification
}

/// All authority components must independently authorize clearance,
/// compartments, and releasability. This is intentional defense in depth:
/// subject, device, and workload are all weakest-link controls.
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

    let compartment_allowed = evaluate_required_tokens(
        label.compartments(),
        authority.subject().compartments(),
        authority.device().compartments(),
        authority.workload().compartments(),
    );

    let releasability_allowed = evaluate_required_tokens(
        label.releasable_to(),
        authority.subject().releasable_to(),
        authority.device().releasable_to(),
        authority.workload().releasable_to(),
    );

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

fn evaluate_required_tokens(
    required: &BTreeSet<String>,
    subject: &BTreeSet<String>,
    device: &BTreeSet<String>,
    workload: &BTreeSet<String>,
) -> u8 {
    let mut allowed = 1_u8;
    let mut required = required.iter();
    let mut index = 0;
    while index < POLICY_TOKEN_SET_MAX_ITEMS {
        let token = required.next();
        let present = token.is_some() as u8;
        let token = match token {
            Some(token) => token.as_str(),
            None => "SKRIFHEIM-NOOP",
        };
        let mut token_allowed = 1_u8;
        token_allowed &= contains_policy_token_ct(subject, token) as u8;
        token_allowed &= contains_policy_token_ct(device, token) as u8;
        token_allowed &= contains_policy_token_ct(workload, token) as u8;
        allowed &= (present ^ 1) | token_allowed;
        index += 1;
    }
    allowed
}

pub fn require_allowed(decision: PlannerDecision) -> Result<()> {
    match decision {
        PlannerDecision::Allow { .. } => Ok(()),
        PlannerDecision::Redact { reason, .. } | PlannerDecision::Reject { reason, .. } => {
            Err(SkrifheimError::PolicyDenied(reason))
        }
    }
}
