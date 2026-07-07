use super::*;
use alloc::{string::String, vec, vec::Vec};
use skrifheim_core::POLICY_TOKEN_SET_MAX_ITEMS;

fn input(
    classification: Classification,
    sovereignty: Vec<String>,
    pii: PiiMarker,
    ai_processing: AiProcessingEligibility,
    threshold: Option<u16>,
) -> Result<QueryResultInput> {
    QueryResultInput::new(
        SecurityLabel::new(classification, Vec::new(), Vec::new())?,
        sovereignty,
        pii,
        ai_processing,
        match threshold {
            Some(value) => Some(ConfidenceThreshold::new(value)?),
            None => None,
        },
    )
}

#[test]
fn confidence_threshold_is_bounded() {
    assert!(matches!(
        ConfidenceThreshold::new(ConfidenceThreshold::MAX + 1),
        Err(SkrifheimError::InvalidConfidence)
    ));
}

#[test]
fn result_classification_joins_all_metadata() -> Result<()> {
    let result = derive_result_classification(&[
        input(
            Classification::Restricted,
            vec![String::from("eu")],
            PiiMarker::NoPii,
            AiProcessingEligibility::Eligible,
            Some(500),
        )?,
        input(
            Classification::Secret,
            vec![String::from("se")],
            PiiMarker::ContainsPii,
            AiProcessingEligibility::NotEligible,
            Some(900),
        )?,
    ])?;

    assert_eq!(result.output_classification(), Classification::Secret);
    assert!(result.sovereignty().is_exact());
    assert_eq!(result.sovereignty().len(), 2);
    assert_eq!(
        result.sovereignty().containment("EU"),
        SovereigntyContainment::Present
    );
    assert_eq!(
        result.sovereignty().containment("SE"),
        SovereigntyContainment::Present
    );
    assert_eq!(result.pii(), PiiMarker::ContainsPii);
    assert_eq!(result.ai_processing(), AiProcessingEligibility::NotEligible);
    assert_eq!(
        result.confidence_threshold(),
        Some(ConfidenceThreshold::new(900)?)
    );
    Ok(())
}

#[test]
fn result_classification_rejects_empty_input_sets() {
    assert!(matches!(
        derive_result_classification(&[]),
        Err(SkrifheimError::InvalidQueryRequest)
    ));
}

#[test]
fn result_classification_rejects_too_many_inputs() -> Result<()> {
    let mut inputs = Vec::new();
    for index in 0..=RESULT_CLASSIFICATION_INPUT_MAX_ITEMS {
        inputs.push(input(
            Classification::Public,
            vec![alloc::format!("JURISDICTION-{index}")],
            PiiMarker::NoPii,
            AiProcessingEligibility::Eligible,
            None,
        )?);
    }

    assert!(matches!(
        derive_result_classification(&inputs),
        Err(SkrifheimError::InvalidQueryRequest)
    ));
    Ok(())
}

#[test]
fn result_classification_saturates_sovereignty_overflow() -> Result<()> {
    let mut first = Vec::new();
    let mut second = Vec::new();
    for index in 0..POLICY_TOKEN_SET_MAX_ITEMS {
        first.push(alloc::format!("FIRST-{index}"));
        second.push(alloc::format!("SECOND-{index}"));
    }
    let inputs = vec![
        input(
            Classification::Public,
            first,
            PiiMarker::NoPii,
            AiProcessingEligibility::Eligible,
            None,
        )?,
        input(
            Classification::Public,
            second,
            PiiMarker::NoPii,
            AiProcessingEligibility::Eligible,
            None,
        )?,
    ];

    let result = derive_result_classification(&inputs)?;

    assert!(result.sovereignty().is_multi_jurisdiction());
    assert!(result.sovereignty().requires_restrictive_handling());
    assert!(result.sovereignty().exact_tokens().is_none());
    assert_eq!(
        result.sovereignty().containment("FIRST-0"),
        SovereigntyContainment::Indeterminate
    );
    Ok(())
}

#[test]
fn single_input_sovereignty_overflow_saturates_after_validation() -> Result<()> {
    let mut tokens = Vec::new();
    for index in 0..=POLICY_TOKEN_SET_MAX_ITEMS {
        tokens.push(alloc::format!("JURISDICTION-{index}"));
    }
    let input = input(
        Classification::Public,
        tokens,
        PiiMarker::NoPii,
        AiProcessingEligibility::Eligible,
        None,
    )?;

    let result = derive_result_classification(&[input])?;

    assert!(result.sovereignty().is_multi_jurisdiction());
    Ok(())
}

#[test]
fn sovereignty_overflow_still_rejects_invalid_tokens() {
    let mut tokens = Vec::new();
    for index in 0..=POLICY_TOKEN_SET_MAX_ITEMS {
        tokens.push(alloc::format!("JURISDICTION-{index}"));
    }
    tokens.push(String::from("bad token"));

    assert!(matches!(
        input(
            Classification::Public,
            tokens,
            PiiMarker::NoPii,
            AiProcessingEligibility::Eligible,
            None,
        ),
        Err(SkrifheimError::InvalidSecurityToken)
    ));
}

#[test]
fn sovereignty_scope_rejects_unbounded_input_before_scanning() {
    let mut tokens = Vec::new();
    for index in 0..=SOVEREIGNTY_SCOPE_INPUT_MAX_ITEMS {
        tokens.push(alloc::format!("JURISDICTION-{index}"));
    }

    assert!(matches!(
        input(
            Classification::Public,
            tokens,
            PiiMarker::NoPii,
            AiProcessingEligibility::Eligible,
            None,
        ),
        Err(SkrifheimError::InvalidQueryRequest)
    ));
}

#[test]
fn sovereignty_scope_debug_redacts_exact_and_saturated_state() -> Result<()> {
    let exact = SovereigntyScope::from_tokens(vec![String::from("se")])?;
    let saturated = SovereigntyScope::multi_jurisdiction();

    assert!(!alloc::format!("{exact:?}").contains("SE"));
    assert!(!alloc::format!("{saturated:?}").contains("MultiJurisdiction"));
    Ok(())
}

#[test]
fn debug_redacts_query_result_input_and_classification_metadata() -> Result<()> {
    let input = input(
        Classification::Secret,
        vec![String::from("se")],
        PiiMarker::ContainsPii,
        AiProcessingEligibility::NotEligible,
        Some(900),
    )?;
    let input_debug = alloc::format!("{input:?}");
    assert!(!input_debug.contains("Secret"));
    assert!(!input_debug.contains("SE"));
    assert!(!input_debug.contains("ContainsPii"));
    assert!(!input_debug.contains("NotEligible"));
    assert!(!input_debug.contains("900"));

    let result = derive_result_classification(&[input])?;
    let result_debug = alloc::format!("{result:?}");
    assert!(!result_debug.contains("Secret"));
    assert!(!result_debug.contains("SE"));
    assert!(!result_debug.contains("ContainsPii"));
    assert!(!result_debug.contains("NotEligible"));
    assert!(!result_debug.contains("900"));
    Ok(())
}
