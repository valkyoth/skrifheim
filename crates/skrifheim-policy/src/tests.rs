use super::*;
use crate::decision::calculate_output_classification;
use alloc::{string::String, vec, vec::Vec};
use skrifheim_core::{
    AccessDeniedReason, Classification, DeviceId, Result, SecurityLabel, SkrifheimError, WorkloadId,
};

fn id<T>(id: Option<T>) -> Result<T> {
    id.ok_or(SkrifheimError::InvalidIdentifier)
}

fn authority(
    subject_clearance: Classification,
    device_clearance: Classification,
    workload_clearance: Classification,
    compartments: Vec<String>,
    releasable_to: Vec<String>,
) -> Result<AuthorityContext> {
    Ok(AuthorityContext::new(
        SubjectContext::new(
            subject_clearance,
            compartments.clone(),
            releasable_to.clone(),
        )?,
        DeviceContext::new(
            id(DeviceId::from_u128(1))?,
            device_clearance,
            compartments.clone(),
            releasable_to.clone(),
        )?,
        WorkloadContext::new(
            id(WorkloadId::from_u128(2))?,
            workload_clearance,
            compartments,
            releasable_to,
        )?,
    ))
}

#[test]
fn read_requires_clearance() -> Result<()> {
    let authority = authority(
        Classification::Restricted,
        Classification::Secret,
        Classification::Secret,
        vec![String::from("A")],
        vec![String::from("EU")],
    )?;
    let label = SecurityLabel::new(
        Classification::Secret,
        vec![String::from("A")],
        vec![String::from("EU")],
    )?;
    let decision = evaluate_read(&authority, &label);
    assert_eq!(decision.kind(), DecisionKind::Reject);
    assert_eq!(decision.proof().decision(), DecisionKind::Reject);
    assert_eq!(
        decision.proof().output_classification(),
        Classification::Public
    );
    Ok(())
}

#[test]
fn missing_releasability_redacts_instead_of_allows() -> Result<()> {
    let authority = authority(
        Classification::Secret,
        Classification::Secret,
        Classification::Secret,
        vec![String::from("A")],
        Vec::new(),
    )?;
    let label = SecurityLabel::new(
        Classification::Secret,
        vec![String::from("A")],
        vec![String::from("EU")],
    )?;
    let decision = evaluate_read(&authority, &label);
    assert_eq!(decision.kind(), DecisionKind::Redact);
    assert_eq!(decision.proof().decision(), DecisionKind::Redact);
    assert_eq!(
        decision.proof().output_classification(),
        Classification::Public
    );
    Ok(())
}

#[test]
fn denial_reasons_do_not_disclose_compartment_names() -> Result<()> {
    let authority = authority(
        Classification::Secret,
        Classification::Secret,
        Classification::Secret,
        Vec::new(),
        Vec::new(),
    )?;
    let label = SecurityLabel::new(
        Classification::Secret,
        vec![String::from("SECRET-COMPARTMENT")],
        Vec::new(),
    )?;
    let decision = evaluate_read(&authority, &label);
    assert_eq!(decision.kind(), DecisionKind::Reject);
    let reason = AccessDeniedReason::new();
    assert_eq!(decision.denial_reason(), Some(&reason));
    Ok(())
}

#[test]
fn subject_context_rejects_unicode_homograph_tokens() {
    assert!(matches!(
        SubjectContext::new(
            Classification::Secret,
            vec![String::from("ЕU-COMMAND")],
            Vec::new(),
        ),
        Err(SkrifheimError::InvalidSecurityToken)
    ));
}

#[test]
fn device_clearance_limits_reads() -> Result<()> {
    let authority = authority(
        Classification::Secret,
        Classification::Restricted,
        Classification::Secret,
        vec![String::from("A")],
        Vec::new(),
    )?;
    let label = SecurityLabel::new(Classification::Secret, vec![String::from("A")], Vec::new())?;
    assert_eq!(
        evaluate_read(&authority, &label).kind(),
        DecisionKind::Reject
    );
    Ok(())
}

#[test]
fn workload_clearance_limits_reads() -> Result<()> {
    let authority = authority(
        Classification::Secret,
        Classification::Secret,
        Classification::Restricted,
        vec![String::from("A")],
        Vec::new(),
    )?;
    let label = SecurityLabel::new(Classification::Secret, vec![String::from("A")], Vec::new())?;
    assert_eq!(
        evaluate_read(&authority, &label).kind(),
        DecisionKind::Reject
    );
    Ok(())
}

#[test]
fn all_contexts_must_hold_required_compartment() -> Result<()> {
    let authority = AuthorityContext::new(
        SubjectContext::new(Classification::Secret, vec![String::from("A")], Vec::new())?,
        DeviceContext::new(
            id(DeviceId::from_u128(1))?,
            Classification::Secret,
            Vec::new(),
            Vec::new(),
        )?,
        WorkloadContext::new(
            id(WorkloadId::from_u128(2))?,
            Classification::Secret,
            vec![String::from("A")],
            Vec::new(),
        )?,
    );
    let label = SecurityLabel::new(Classification::Secret, vec![String::from("A")], Vec::new())?;
    assert_eq!(
        evaluate_read(&authority, &label).kind(),
        DecisionKind::Reject
    );
    Ok(())
}

#[test]
fn valid_authority_can_read_matching_label() -> Result<()> {
    let authority = authority(
        Classification::Secret,
        Classification::Secret,
        Classification::Secret,
        vec![String::from("A")],
        vec![String::from("EU")],
    )?;
    let label = SecurityLabel::new(
        Classification::Restricted,
        vec![String::from("A")],
        vec![String::from("EU")],
    )?;
    let decision = evaluate_read(&authority, &label);
    assert_eq!(decision.kind(), DecisionKind::Allow);
    assert_eq!(decision.denial_reason(), None);
    Ok(())
}

#[test]
fn output_classification_joins_to_highest_label() -> Result<()> {
    let labels = vec![
        SecurityLabel::new(Classification::Public, Vec::new(), Vec::new())?,
        SecurityLabel::new(Classification::Secret, Vec::new(), Vec::new())?,
        SecurityLabel::new(Classification::Restricted, Vec::new(), Vec::new())?,
    ];
    assert_eq!(
        calculate_output_classification(&labels),
        Classification::Secret
    );
    Ok(())
}

#[test]
fn aggregate_read_proof_counts_all_labels() -> Result<()> {
    let authority = authority(
        Classification::TopSecret,
        Classification::TopSecret,
        Classification::TopSecret,
        Vec::new(),
        Vec::new(),
    )?;
    let labels = vec![
        SecurityLabel::new(Classification::Public, Vec::new(), Vec::new())?,
        SecurityLabel::new(Classification::Secret, Vec::new(), Vec::new())?,
    ];
    let decision = evaluate_read_set(&authority, &labels);
    assert_eq!(decision.kind(), DecisionKind::Allow);
    assert_eq!(decision.proof().input_label_count(), 2);
    assert_eq!(
        decision.proof().output_classification(),
        Classification::Secret
    );
    Ok(())
}

#[test]
fn result_set_rejects_too_many_inputs() -> Result<()> {
    let authority = authority(
        Classification::TopSecret,
        Classification::TopSecret,
        Classification::TopSecret,
        Vec::new(),
        Vec::new(),
    )?;
    let mut inputs = Vec::new();
    for _ in 0..=RESULT_CLASSIFICATION_INPUT_MAX_ITEMS {
        inputs.push(QueryResultInput::label_only(SecurityLabel::public()));
    }
    assert!(matches!(
        evaluate_read_result_set(&authority, &inputs),
        Err(SkrifheimError::InvalidQueryRequest)
    ));
    Ok(())
}

#[test]
fn dangerous_join_rejects_when_output_exceeds_authority() -> Result<()> {
    let authority = authority(
        Classification::Restricted,
        Classification::Restricted,
        Classification::Restricted,
        Vec::new(),
        Vec::new(),
    )?;
    let labels = vec![
        SecurityLabel::new(Classification::Public, Vec::new(), Vec::new())?,
        SecurityLabel::new(Classification::Secret, Vec::new(), Vec::new())?,
    ];
    let decision = evaluate_read_set(&authority, &labels);
    assert_eq!(decision.kind(), DecisionKind::Reject);
    assert_eq!(
        decision.proof().output_classification(),
        Classification::Public
    );
    Ok(())
}

#[test]
fn aggregate_redaction_uses_constant_shape_denial() -> Result<()> {
    let authority = authority(
        Classification::Secret,
        Classification::Secret,
        Classification::Secret,
        Vec::new(),
        Vec::new(),
    )?;
    let labels = vec![SecurityLabel::new(
        Classification::Secret,
        Vec::new(),
        vec![String::from("EU")],
    )?];
    let decision = evaluate_read_set(&authority, &labels);
    assert_eq!(decision.kind(), DecisionKind::Redact);
    let reason = AccessDeniedReason::new();
    assert_eq!(decision.denial_reason(), Some(&reason));
    assert_eq!(decision.proof().decision(), DecisionKind::Redact);
    assert_eq!(
        decision.proof().output_classification(),
        Classification::Public
    );
    Ok(())
}

#[test]
fn releasability_requires_all_three_contexts_independently() -> Result<()> {
    // Subject, device, and workload releasability are all weakest-link controls.
    let authority = AuthorityContext::new(
        SubjectContext::new(Classification::Secret, Vec::new(), vec![String::from("EU")])?,
        DeviceContext::new(
            id(DeviceId::from_u128(1))?,
            Classification::Secret,
            Vec::new(),
            Vec::new(),
        )?,
        WorkloadContext::new(
            id(WorkloadId::from_u128(2))?,
            Classification::Secret,
            Vec::new(),
            vec![String::from("EU")],
        )?,
    );
    let label = SecurityLabel::new(Classification::Secret, Vec::new(), vec![String::from("EU")])?;
    assert_eq!(
        evaluate_read(&authority, &label).kind(),
        DecisionKind::Redact
    );
    Ok(())
}
