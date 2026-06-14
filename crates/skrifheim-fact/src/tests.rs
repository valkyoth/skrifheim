use alloc::{string::String, vec, vec::Vec};
use skrifheim_core::{
    ActorId, Classification, EntityId, FactId, PolicyId, PredicateId, Result, SecurityLabel,
    SkrifheimError, SourceId, TimeRange, Timestamp, TxId, Value, WorldId,
};
use skrifheim_crypto::{AlgorithmId, CryptoEpoch, SignatureEnvelope, SignatureSet};

use super::{Confidence, FACT_LINK_LIST_MAX_ITEMS, FACT_OBJECT_MAX_BYTES, Fact, FactBuilder};

fn id<T>(id: Option<T>) -> Result<T> {
    id.ok_or(SkrifheimError::InvalidIdentifier)
}

fn signature_set() -> Result<SignatureSet> {
    SignatureSet::new(vec![SignatureEnvelope::new(
        AlgorithmId::Ed25519,
        CryptoEpoch(1),
        "test-key",
        vec![1; 64],
    )?])
}

fn invalid_fact_without_evidence() -> Result<Fact> {
    Ok(Fact {
        id: id(FactId::from_u128(1))?,
        world_id: id(WorldId::from_u128(2))?,
        subject: id(EntityId::from_u128(3))?,
        predicate: id(PredicateId::from_u128(4))?,
        object: Value::Boolean(true),
        valid_time: TimeRange::new(Timestamp(5), None)?,
        committed_at: id(TxId::from_u128(6))?,
        asserted_by: id(ActorId::from_u128(7))?,
        evidence: Vec::new(),
        confidence: Confidence::max(),
        caused_by: vec![id(FactId::from_u128(9))?],
        supersedes: Vec::new(),
        invalidates: Vec::new(),
        policy_id: id(PolicyId::from_u128(10))?,
        label: SecurityLabel::new(
            Classification::Restricted,
            vec![String::from("EU-COMMAND")],
            vec![String::from("EU")],
        )?,
        signatures: signature_set()?,
    })
}

fn fact() -> Result<Fact> {
    base_builder()?.build()
}

fn base_builder() -> Result<FactBuilder> {
    Ok(Fact::builder()
        .id(id(FactId::from_u128(1))?)
        .world_id(id(WorldId::from_u128(2))?)
        .subject(id(EntityId::from_u128(3))?)
        .predicate(id(PredicateId::from_u128(4))?)
        .object(Value::Boolean(true))?
        .valid_time(TimeRange::new(Timestamp(5), None)?)
        .committed_at(id(TxId::from_u128(6))?)
        .asserted_by(id(ActorId::from_u128(7))?)
        .add_evidence(id(SourceId::from_u128(8))?)?
        .confidence(Confidence::max())
        .add_caused_by(id(FactId::from_u128(9))?)?
        .policy_id(id(PolicyId::from_u128(10))?)
        .label(SecurityLabel::new(
            Classification::Restricted,
            vec![String::from("EU-COMMAND")],
            vec![String::from("EU")],
        )?)
        .signatures(signature_set()?))
}

#[test]
fn valid_fact_passes_validation() -> Result<()> {
    assert_eq!(fact()?.validate(), Ok(()));
    Ok(())
}

#[test]
fn builder_constructs_valid_fact() -> Result<()> {
    let fact = fact()?;
    assert_eq!(fact.confidence(), Confidence::max());
    assert_eq!(fact.evidence().len(), 1);
    assert!(fact.is_derived_from(id(FactId::from_u128(9))?));
    Ok(())
}

#[test]
fn debug_redacts_fact_payload_and_label_tokens() -> Result<()> {
    let builder = base_builder()?.object(Value::Text(String::from("classified payload")))?;
    let builder_debug = alloc::format!("{builder:?}");
    let fact = builder.build()?;
    let fact_debug = alloc::format!("{fact:?}");

    assert!(!builder_debug.contains("classified payload"));
    assert!(!builder_debug.contains("EU-COMMAND"));
    assert!(!fact_debug.contains("classified payload"));
    assert!(!fact_debug.contains("EU-COMMAND"));
    assert!(builder_debug.contains("<redacted>"));
    assert!(fact_debug.contains("<redacted>"));
    assert!(fact_debug.contains("Restricted"));
    Ok(())
}

#[test]
fn builder_requires_all_required_fields() {
    assert_eq!(
        Fact::builder().build(),
        Err(SkrifheimError::MissingFactField("id"))
    );
}

#[test]
fn builder_requires_evidence() -> Result<()> {
    let result = base_builder()?.evidence(Vec::new())?.build();
    assert_eq!(result, Err(SkrifheimError::EmptyEvidence));
    Ok(())
}

#[test]
fn fact_validation_requires_evidence() -> Result<()> {
    let fact = invalid_fact_without_evidence()?;
    assert_eq!(fact.validate(), Err(SkrifheimError::EmptyEvidence));
    Ok(())
}

#[test]
fn confidence_rejects_out_of_range_values() {
    assert_eq!(
        Confidence::new(1001),
        Err(SkrifheimError::InvalidConfidence)
    );
}

#[test]
fn fact_rejects_self_referential_causality() -> Result<()> {
    let fact = fact()?;
    let result = base_builder()?.add_caused_by(fact.id())?.build();
    assert_eq!(result, Err(SkrifheimError::SelfReferentialFact));
    Ok(())
}

#[test]
fn builder_rejects_invalid_valid_time() -> Result<()> {
    let result = TimeRange::new(Timestamp(20), Some(Timestamp(10)));
    assert_eq!(result, Err(SkrifheimError::InvalidTimeRange));
    Ok(())
}

#[test]
fn builder_deduplicates_evidence_and_causal_links() -> Result<()> {
    let source = id(SourceId::from_u128(8))?;
    let cause = id(FactId::from_u128(9))?;
    let fact = base_builder()?
        .add_evidence(source)?
        .add_caused_by(cause)?
        .build()?;
    assert_eq!(fact.evidence(), &[source]);
    assert_eq!(fact.caused_by(), &[cause]);
    Ok(())
}

#[test]
fn builder_deduplicates_bulk_fact_links() -> Result<()> {
    let cause = id(FactId::from_u128(9))?;
    let fact = base_builder()?.caused_by(vec![cause, cause])?.build()?;
    assert_eq!(fact.caused_by(), &[cause]);
    Ok(())
}

#[test]
fn validation_rejects_oversized_fact_link_lists() -> Result<()> {
    let mut links = Vec::new();
    for index in 0..=FACT_LINK_LIST_MAX_ITEMS {
        links.push(id(FactId::from_u128((index + 20) as u128))?);
    }
    let result = base_builder()?.caused_by(links);
    assert_eq!(result, Err(SkrifheimError::TooManyFactLinks));
    Ok(())
}

#[test]
fn validation_rejects_oversized_evidence_lists() -> Result<()> {
    let mut evidence = Vec::new();
    for index in 0..=FACT_LINK_LIST_MAX_ITEMS {
        evidence.push(id(SourceId::from_u128((index + 20) as u128))?);
    }
    let result = base_builder()?.evidence(evidence);
    assert_eq!(result, Err(SkrifheimError::TooManyFactLinks));
    Ok(())
}

#[test]
fn builder_rejects_incremental_evidence_before_unbounded_growth() -> Result<()> {
    let mut builder = base_builder()?.evidence(Vec::new())?;
    for index in 0..FACT_LINK_LIST_MAX_ITEMS {
        builder = builder.add_evidence(id(SourceId::from_u128((index + 20) as u128))?)?;
    }
    assert_eq!(
        builder.add_evidence(id(SourceId::from_u128(5000))?),
        Err(SkrifheimError::TooManyFactLinks)
    );
    Ok(())
}

#[test]
fn validation_rejects_oversized_text_objects() -> Result<()> {
    let oversized = String::from_utf8(vec![b'a'; FACT_OBJECT_MAX_BYTES + 1])
        .map_err(|_| SkrifheimError::ValueTooLarge)?;
    let result = base_builder()?.object(Value::Text(oversized));
    assert_eq!(result, Err(SkrifheimError::ValueTooLarge));
    Ok(())
}

#[test]
fn validation_rejects_oversized_byte_objects() -> Result<()> {
    let result = base_builder()?.object(Value::Bytes(vec![1; FACT_OBJECT_MAX_BYTES + 1]));
    assert_eq!(result, Err(SkrifheimError::ValueTooLarge));
    Ok(())
}
