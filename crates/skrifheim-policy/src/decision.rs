use core::fmt;

use skrifheim_core::{
    AccessDeniedReason, Classification, POLICY_TOKEN_SET_MAX_ITEMS, PolicyTokenSet, Result,
    SecurityLabel, SkrifheimError, contains_policy_token_slot_ct,
};

use crate::result::derive_result_classification;
use crate::{
    AuthorityContext, QueryResultInput, RESULT_CLASSIFICATION_INPUT_MAX_ITEMS, ResultClassification,
};

#[derive(Clone)]
pub struct PlannerDecision {
    kind: DecisionKind,
    reason: Option<AccessDeniedReason>,
    proof: PolicyProof,
}

impl PlannerDecision {
    fn allow(input_label_count: usize, result_classification: ResultClassification) -> Self {
        Self {
            kind: DecisionKind::Allow,
            reason: None,
            proof: PolicyProof::new(
                DecisionKind::Allow,
                input_label_count,
                result_classification,
            ),
        }
    }

    fn redact(input_label_count: usize) -> Self {
        Self {
            kind: DecisionKind::Redact,
            reason: Some(AccessDeniedReason::new()),
            proof: PolicyProof::new(
                DecisionKind::Redact,
                input_label_count,
                ResultClassification::public(),
            ),
        }
    }

    fn reject(input_label_count: usize) -> Self {
        Self {
            kind: DecisionKind::Reject,
            reason: Some(AccessDeniedReason::new()),
            proof: PolicyProof::new(
                DecisionKind::Reject,
                input_label_count,
                ResultClassification::public(),
            ),
        }
    }

    #[must_use]
    pub const fn kind(&self) -> DecisionKind {
        self.kind
    }

    #[must_use]
    pub const fn proof(&self) -> &PolicyProof {
        &self.proof
    }

    #[must_use]
    pub const fn denial_reason(&self) -> Option<&AccessDeniedReason> {
        self.reason.as_ref()
    }
}

impl fmt::Debug for PlannerDecision {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PlannerDecision")
            .field("kind", &self.kind)
            .field("denial_reason", &self.reason.as_ref().map(|_| "<redacted>"))
            .field("proof", &"<redacted>")
            .finish()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DecisionKind {
    Allow,
    Redact,
    Reject,
}

#[derive(Clone)]
pub struct PolicyProof {
    decision: DecisionKind,
    input_label_count: usize,
    result_classification: ResultClassification,
}

impl PolicyProof {
    #[must_use]
    pub(crate) fn new(
        decision: DecisionKind,
        input_label_count: usize,
        result_classification: ResultClassification,
    ) -> Self {
        Self {
            decision,
            input_label_count,
            result_classification,
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
        self.result_classification.output_classification()
    }

    #[must_use]
    pub const fn result_classification(&self) -> &ResultClassification {
        &self.result_classification
    }
}

impl fmt::Debug for PolicyProof {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PolicyProof")
            .field("decision", &self.decision)
            .field("input_label_count", &self.input_label_count)
            .field("result_classification", &"<redacted>")
            .finish()
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
        return PlannerDecision::reject(labels.len());
    }

    if redacted == 1 {
        return PlannerDecision::redact(labels.len());
    }

    let output_classification = calculate_output_classification(labels);
    PlannerDecision::allow(
        labels.len(),
        ResultClassification::classification_only(output_classification),
    )
}

pub fn evaluate_read_result_set(
    authority: &AuthorityContext,
    inputs: &[QueryResultInput],
) -> Result<PlannerDecision> {
    if inputs.is_empty() || inputs.len() > RESULT_CLASSIFICATION_INPUT_MAX_ITEMS {
        return Err(SkrifheimError::InvalidQueryRequest);
    }
    let mut rejected = 0_u8;
    let mut redacted = 0_u8;

    for input in inputs {
        match evaluate_label(authority, input.label()) {
            DecisionKind::Allow => {}
            DecisionKind::Redact => redacted = 1,
            DecisionKind::Reject => rejected = 1,
        }
    }

    if rejected == 1 {
        return Ok(PlannerDecision::reject(inputs.len()));
    }

    if redacted == 1 {
        return Ok(PlannerDecision::redact(inputs.len()));
    }

    Ok(PlannerDecision::allow(
        inputs.len(),
        derive_result_classification(inputs)?,
    ))
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
    required: &PolicyTokenSet,
    subject: &PolicyTokenSet,
    device: &PolicyTokenSet,
    workload: &PolicyTokenSet,
) -> u8 {
    debug_assert!(required.len() <= POLICY_TOKEN_SET_MAX_ITEMS);
    debug_assert!(subject.len() <= POLICY_TOKEN_SET_MAX_ITEMS);
    debug_assert!(device.len() <= POLICY_TOKEN_SET_MAX_ITEMS);
    debug_assert!(workload.len() <= POLICY_TOKEN_SET_MAX_ITEMS);

    let mut allowed = 1_u8;
    for token in required.slots() {
        let token = *token;
        let present = token.present_mask();
        let mut token_allowed = 1_u8;
        token_allowed &= contains_policy_token_slot_ct(subject, token) as u8;
        token_allowed &= contains_policy_token_slot_ct(device, token) as u8;
        token_allowed &= contains_policy_token_slot_ct(workload, token) as u8;
        allowed &= (present ^ 1) | token_allowed;
    }
    allowed
}

pub fn require_allowed(decision: PlannerDecision) -> Result<()> {
    if decision.kind() == DecisionKind::Allow {
        return Ok(());
    }
    Err(SkrifheimError::PolicyDenied(AccessDeniedReason::new()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::{format, vec::Vec};

    #[test]
    fn required_token_evaluation_uses_fixed_slots() -> Result<()> {
        let mut tokens = Vec::new();
        for index in 0..POLICY_TOKEN_SET_MAX_ITEMS {
            tokens.push(format!("TOKEN-{index}"));
        }
        let required = PolicyTokenSet::new(tokens.clone())?;
        let authority_tokens = PolicyTokenSet::new(tokens)?;

        assert_eq!(
            evaluate_required_tokens(
                &required,
                &authority_tokens,
                &authority_tokens,
                &authority_tokens,
            ),
            1
        );
        Ok(())
    }

    #[test]
    fn debug_redacts_decision_proof_result_metadata() -> Result<()> {
        let label = SecurityLabel::new(
            Classification::Secret,
            Vec::new(),
            alloc::vec![alloc::string::String::from("SE")],
        )?;
        let input = QueryResultInput::new(
            label,
            alloc::vec![alloc::string::String::from("SE")],
            crate::PiiMarker::ContainsPii,
            crate::AiProcessingEligibility::NotEligible,
            Some(crate::ConfidenceThreshold::new(900)?),
        )?;
        let classification = derive_result_classification(&[input])?;
        let decision = PlannerDecision::allow(1, classification);

        let decision_debug = format!("{decision:?}");
        assert!(!decision_debug.contains("Secret"));
        assert!(!decision_debug.contains("SE"));
        assert!(!decision_debug.contains("ContainsPii"));
        assert!(!decision_debug.contains("NotEligible"));
        assert!(!decision_debug.contains("900"));

        let proof_debug = format!("{:?}", decision.proof());
        assert!(!proof_debug.contains("Secret"));
        assert!(!proof_debug.contains("SE"));
        assert!(!proof_debug.contains("ContainsPii"));
        assert!(!proof_debug.contains("NotEligible"));
        assert!(!proof_debug.contains("900"));
        Ok(())
    }
}
