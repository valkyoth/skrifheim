use super::*;
use alloc::{string::String, vec};

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
    assert!(matches!(
        evaluate_read(&authority, &label),
        PlannerDecision::Reject { .. }
    ));
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
    assert!(matches!(
        evaluate_read(&authority, &label),
        PlannerDecision::Redact { .. }
    ));
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
    assert_eq!(
        evaluate_read(&authority, &label),
        PlannerDecision::Reject {
            reason: AccessDeniedReason::new()
        }
    );
    Ok(())
}

#[test]
fn subject_context_rejects_unicode_homograph_tokens() {
    assert_eq!(
        SubjectContext::new(
            Classification::Secret,
            vec![String::from("ЕU-COMMAND")],
            Vec::new(),
        ),
        Err(SkrifheimError::InvalidSecurityToken)
    );
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
    assert!(matches!(
        evaluate_read(&authority, &label),
        PlannerDecision::Reject { .. }
    ));
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
    assert!(matches!(
        evaluate_read(&authority, &label),
        PlannerDecision::Reject { .. }
    ));
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
    assert!(matches!(
        evaluate_read(&authority, &label),
        PlannerDecision::Reject { .. }
    ));
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
    assert_eq!(evaluate_read(&authority, &label), PlannerDecision::Allow);
    Ok(())
}
