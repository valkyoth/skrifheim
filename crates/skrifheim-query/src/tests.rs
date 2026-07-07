use super::*;
use alloc::{string::String, vec};
use skrifheim_core::{
    Classification, DeviceId, POLICY_TOKEN_SET_MAX_ITEMS, SkrifheimError, WorkloadId,
};
use skrifheim_policy::{
    AiProcessingEligibility, ConfidenceThreshold, DeviceContext, PiiMarker, QueryResultInput,
    SovereigntyContainment, SubjectContext, WorkloadContext,
};

fn id<T>(id: Option<T>) -> skrifheim_core::Result<T> {
    id.ok_or(SkrifheimError::InvalidIdentifier)
}

fn authority(clearance: Classification) -> skrifheim_core::Result<AuthorityContext> {
    Ok(AuthorityContext::new(
        SubjectContext::new(clearance, Vec::new(), Vec::new())?,
        DeviceContext::new(
            id(DeviceId::from_u128(2))?,
            clearance,
            Vec::new(),
            Vec::new(),
        )?,
        WorkloadContext::new(
            id(WorkloadId::from_u128(3))?,
            clearance,
            Vec::new(),
            Vec::new(),
        )?,
    ))
}

#[test]
fn plan_records_rejection() -> skrifheim_core::Result<()> {
    let request = QueryRequest::new(
        id(WorldId::from_u128(1))?,
        QueryIntent::BuildContextPack,
        vec![SecurityLabel::new(
            Classification::TopSecret,
            Vec::new(),
            Vec::new(),
        )?],
    )?;
    let authority = authority(Classification::Secret)?;
    let plan = request.plan(&authority)?;
    assert_eq!(plan.world(), id(WorldId::from_u128(1))?);
    assert_eq!(plan.intent(), &QueryIntent::BuildContextPack);
    assert!(plan.has_rejection());
    assert_eq!(
        plan.output_classification(),
        skrifheim_core::Classification::Public
    );
    Ok(())
}

#[test]
fn redaction_blocks_executability() -> skrifheim_core::Result<()> {
    let request = QueryRequest::new(
        id(WorldId::from_u128(1))?,
        QueryIntent::ReadFacts,
        vec![SecurityLabel::new(
            Classification::Secret,
            Vec::new(),
            vec![String::from("EU")],
        )?],
    )?;
    let authority = authority(Classification::Secret)?;
    let plan = request.plan(&authority)?;
    assert!(plan.has_redaction());
    assert!(!plan.is_executable());
    assert_eq!(plan.proof().input_label_count(), 0);
    assert_eq!(plan.output_classification(), Classification::Public);
    Ok(())
}

#[test]
fn plan_escalates_output_classification_for_joins() -> skrifheim_core::Result<()> {
    let request = QueryRequest::new(
        id(WorldId::from_u128(1))?,
        QueryIntent::ReadFacts,
        vec![
            SecurityLabel::new(Classification::Public, Vec::new(), Vec::new())?,
            SecurityLabel::new(Classification::Secret, Vec::new(), Vec::new())?,
        ],
    )?;
    let authority = authority(Classification::TopSecret)?;
    let plan = request.plan(&authority)?;
    assert_eq!(plan.output_classification(), Classification::Secret);
    assert!(plan.is_executable());
    Ok(())
}

#[test]
fn plan_propagates_result_classification_metadata() -> skrifheim_core::Result<()> {
    let request = QueryRequest::with_result_inputs(
        id(WorldId::from_u128(1))?,
        QueryIntent::BuildContextPack,
        vec![
            QueryResultInput::new(
                SecurityLabel::new(Classification::Restricted, Vec::new(), Vec::new())?,
                vec![String::from("eu")],
                PiiMarker::NoPii,
                AiProcessingEligibility::Eligible,
                Some(ConfidenceThreshold::new(600)?),
            )?,
            QueryResultInput::new(
                SecurityLabel::new(Classification::Secret, Vec::new(), Vec::new())?,
                vec![String::from("se")],
                PiiMarker::ContainsPii,
                AiProcessingEligibility::NotEligible,
                Some(ConfidenceThreshold::new(800)?),
            )?,
        ],
    )?;
    let plan = request.plan(&authority(Classification::TopSecret)?)?;
    let result = plan.result_classification();

    assert_eq!(plan.output_classification(), Classification::Secret);
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
        Some(ConfidenceThreshold::new(800)?)
    );
    Ok(())
}

#[test]
fn plan_preserves_saturated_sovereignty_scope() -> skrifheim_core::Result<()> {
    let mut sovereignty = Vec::new();
    for index in 0..=POLICY_TOKEN_SET_MAX_ITEMS {
        sovereignty.push(alloc::format!("JURISDICTION-{index}"));
    }
    let request = QueryRequest::with_result_inputs(
        id(WorldId::from_u128(1))?,
        QueryIntent::BuildContextPack,
        vec![QueryResultInput::new(
            SecurityLabel::new(Classification::Restricted, Vec::new(), Vec::new())?,
            sovereignty,
            PiiMarker::NoPii,
            AiProcessingEligibility::Eligible,
            None,
        )?],
    )?;
    let plan = request.plan(&authority(Classification::TopSecret)?)?;

    assert!(plan.is_executable());
    assert!(
        plan.result_classification()
            .sovereignty()
            .is_multi_jurisdiction()
    );
    assert!(
        plan.result_classification()
            .sovereignty()
            .requires_restrictive_handling()
    );
    Ok(())
}

#[test]
fn non_allow_plan_masks_result_classification_metadata() -> skrifheim_core::Result<()> {
    let mut sovereignty = Vec::new();
    for index in 0..=POLICY_TOKEN_SET_MAX_ITEMS {
        sovereignty.push(alloc::format!("JURISDICTION-{index}"));
    }
    let request = QueryRequest::with_result_inputs(
        id(WorldId::from_u128(1))?,
        QueryIntent::BuildContextPack,
        vec![QueryResultInput::new(
            SecurityLabel::new(Classification::TopSecret, Vec::new(), Vec::new())?,
            sovereignty,
            PiiMarker::ContainsPii,
            AiProcessingEligibility::NotEligible,
            Some(ConfidenceThreshold::new(900)?),
        )?],
    )?;
    let plan = request.plan(&authority(Classification::Secret)?)?;
    let result = plan.result_classification();

    assert!(plan.has_rejection());
    assert_eq!(result.output_classification(), Classification::Public);
    assert!(result.sovereignty().is_exact());
    assert_eq!(result.sovereignty().len(), 0);
    assert_eq!(result.pii(), PiiMarker::NoPii);
    assert_eq!(result.ai_processing(), AiProcessingEligibility::Eligible);
    assert_eq!(result.confidence_threshold(), None);
    Ok(())
}

#[test]
fn plan_rejects_too_many_requested_labels() -> skrifheim_core::Result<()> {
    let mut requested_labels = Vec::new();
    for _ in 0..=QUERY_REQUEST_LABEL_MAX_ITEMS {
        requested_labels.push(SecurityLabel::public());
    }
    assert!(matches!(
        QueryRequest::new(
            id(WorldId::from_u128(1))?,
            QueryIntent::ReadFacts,
            requested_labels,
        ),
        Err(SkrifheimError::InvalidQueryRequest)
    ));
    Ok(())
}

#[test]
fn plan_rejects_empty_requested_labels() -> skrifheim_core::Result<()> {
    let world = id(WorldId::from_u128(1))?;
    assert!(matches!(
        QueryRequest::new(world, QueryIntent::ReadFacts, Vec::new()),
        Err(SkrifheimError::InvalidQueryRequest)
    ));
    let request = QueryRequest {
        world,
        intent: QueryIntent::ReadFacts,
        result_inputs: Vec::new(),
    };
    assert!(matches!(
        request.plan(&authority(Classification::TopSecret)?),
        Err(SkrifheimError::InvalidQueryRequest)
    ));
    Ok(())
}

#[test]
fn label_only_constructor_stores_result_inputs_once() -> skrifheim_core::Result<()> {
    let request = QueryRequest::new(
        id(WorldId::from_u128(1))?,
        QueryIntent::ReadFacts,
        vec![
            SecurityLabel::new(Classification::Public, Vec::new(), Vec::new())?,
            SecurityLabel::new(Classification::Secret, Vec::new(), Vec::new())?,
        ],
    )?;

    assert_eq!(request.requested_label_count(), 2);
    assert_eq!(request.result_input_count(), 2);
    let plan = request.plan(&authority(Classification::TopSecret)?)?;
    assert_eq!(plan.output_classification(), Classification::Secret);
    Ok(())
}

#[test]
fn result_input_constructor_drops_excess_vec_capacity() -> skrifheim_core::Result<()> {
    let mut inputs = Vec::with_capacity(QUERY_REQUEST_LABEL_MAX_ITEMS);
    inputs.push(QueryResultInput::new(
        SecurityLabel::new(Classification::Secret, Vec::new(), Vec::new())?,
        Vec::new(),
        PiiMarker::NoPii,
        AiProcessingEligibility::Eligible,
        None,
    )?);

    let request = QueryRequest::with_result_inputs(
        id(WorldId::from_u128(1))?,
        QueryIntent::ReadFacts,
        inputs,
    )?;

    assert_eq!(request.result_inputs.len(), 1);
    assert_eq!(request.result_inputs.capacity(), 1);
    Ok(())
}

#[test]
fn query_result_input_memory_budget_is_explicit() {
    assert_eq!(
        QUERY_REQUEST_INPUT_MEMORY_BUDGET_BYTES,
        QUERY_REQUEST_LABEL_MAX_ITEMS * RESULT_CLASSIFICATION_INPUT_FIXED_STORAGE_BYTES
    );
}

#[test]
fn debug_redacts_query_request_and_plan_metadata() -> skrifheim_core::Result<()> {
    let request = QueryRequest::with_result_inputs(
        id(WorldId::from_u128(1))?,
        QueryIntent::BuildContextPack,
        vec![QueryResultInput::new(
            SecurityLabel::new(Classification::Secret, Vec::new(), Vec::new())?,
            vec![String::from("se")],
            PiiMarker::ContainsPii,
            AiProcessingEligibility::NotEligible,
            Some(ConfidenceThreshold::new(900)?),
        )?],
    )?;
    let request_debug = alloc::format!("{request:?}");
    assert!(!request_debug.contains("Secret"));
    assert!(!request_debug.contains("SE"));
    assert!(!request_debug.contains("ContainsPii"));
    assert!(!request_debug.contains("NotEligible"));
    assert!(!request_debug.contains("900"));
    assert!(!request_debug.contains("BuildContextPack"));
    assert!(!request_debug.contains("result_input_count: 1"));

    let plan = request.plan(&authority(Classification::TopSecret)?)?;
    let plan_debug = alloc::format!("{plan:?}");
    assert!(!plan_debug.contains("Secret"));
    assert!(!plan_debug.contains("SE"));
    assert!(!plan_debug.contains("ContainsPii"));
    assert!(!plan_debug.contains("NotEligible"));
    assert!(!plan_debug.contains("900"));
    assert!(!plan_debug.contains("BuildContextPack"));
    assert!(!plan_debug.contains("Allow"));
    Ok(())
}
