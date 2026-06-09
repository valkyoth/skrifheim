#![no_std]
#![forbid(unsafe_code)]

extern crate alloc;

use alloc::vec::Vec;
use skrifheim_core::{
    ActorId, EntityId, FactId, PolicyId, PredicateId, Result, SecurityLabel, SkrifheimError,
    SourceId, TimeRange, TxId, Value, WorldId,
};
use skrifheim_crypto::SignatureSet;

mod builder;

pub use builder::FactBuilder;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Confidence(u16);

impl Confidence {
    #[must_use]
    pub const fn zero() -> Self {
        Self(0)
    }

    #[must_use]
    pub const fn max() -> Self {
        Self(1000)
    }

    pub const fn new(value: u16) -> Result<Self> {
        if value > 1000 {
            return Err(SkrifheimError::InvalidConfidence);
        }
        Ok(Self(value))
    }

    #[must_use]
    pub const fn clamped(value: u16) -> Self {
        Self(if value > 1000 { 1000 } else { value })
    }

    #[must_use]
    pub const fn get(self) -> u16 {
        self.0
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Fact {
    pub id: FactId,
    pub world_id: WorldId,
    pub subject: EntityId,
    pub predicate: PredicateId,
    pub object: Value,
    pub valid_time: TimeRange,
    pub committed_at: TxId,
    pub asserted_by: ActorId,
    pub evidence: Vec<SourceId>,
    pub confidence: Confidence,
    pub caused_by: Vec<FactId>,
    pub supersedes: Vec<FactId>,
    pub invalidates: Vec<FactId>,
    pub policy_id: PolicyId,
    pub label: SecurityLabel,
    pub signatures: SignatureSet,
}

impl Fact {
    #[must_use]
    pub fn builder() -> FactBuilder {
        FactBuilder::new()
    }

    pub fn validate(&self) -> Result<()> {
        if let Some(end) = self.valid_time.end
            && end.0 < self.valid_time.start.0
        {
            return Err(SkrifheimError::InvalidTimeRange);
        }
        if self.evidence.is_empty() {
            return Err(SkrifheimError::EmptyEvidence);
        }
        if self.caused_by.contains(&self.id)
            || self.supersedes.contains(&self.id)
            || self.invalidates.contains(&self.id)
        {
            return Err(SkrifheimError::SelfReferentialFact);
        }
        self.signatures.require_non_empty()
    }

    #[must_use]
    pub fn is_derived_from(&self, source: FactId) -> bool {
        self.caused_by.contains(&source)
            || self.supersedes.contains(&source)
            || self.invalidates.contains(&source)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::{string::String, vec};
    use skrifheim_core::{Classification, Timestamp};
    use skrifheim_crypto::{AlgorithmId, CryptoEpoch, SignatureEnvelope};

    fn id<T>(id: Option<T>) -> Result<T> {
        id.ok_or(SkrifheimError::InvalidIdentifier)
    }

    fn signature_set() -> Result<SignatureSet> {
        Ok(SignatureSet {
            signatures: vec![SignatureEnvelope::new(
                AlgorithmId::Ed25519,
                CryptoEpoch(1),
                "test-key",
                vec![1; 64],
            )?],
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
            .object(Value::Boolean(true))
            .valid_time(TimeRange::new(Timestamp(5), None))
            .committed_at(id(TxId::from_u128(6))?)
            .asserted_by(id(ActorId::from_u128(7))?)
            .add_evidence(id(SourceId::from_u128(8))?)
            .confidence_clamped(2000)
            .add_caused_by(id(FactId::from_u128(9))?)
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
        assert_eq!(fact.confidence, Confidence::max());
        assert_eq!(fact.evidence.len(), 1);
        assert!(fact.is_derived_from(id(FactId::from_u128(9))?));
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
        let result = base_builder()?.evidence(Vec::new()).build();
        assert_eq!(result, Err(SkrifheimError::EmptyEvidence));
        Ok(())
    }

    #[test]
    fn fact_validation_requires_evidence() -> Result<()> {
        let fact = Fact {
            evidence: Vec::new(),
            ..fact()?
        };
        assert_eq!(fact.validate(), Err(SkrifheimError::EmptyEvidence));
        Ok(())
    }

    #[test]
    fn confidence_is_clamped() {
        assert_eq!(Confidence::clamped(2000), Confidence::max());
        assert_eq!(Confidence::clamped(7).get(), 7);
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
        let result = base_builder()?.add_caused_by(fact.id).build();
        assert_eq!(result, Err(SkrifheimError::SelfReferentialFact));
        Ok(())
    }

    #[test]
    fn builder_rejects_invalid_valid_time() -> Result<()> {
        let result = base_builder()?
            .valid_time(TimeRange::new(Timestamp(20), Some(Timestamp(10))))
            .build();
        assert_eq!(result, Err(SkrifheimError::InvalidTimeRange));
        Ok(())
    }

    #[test]
    fn builder_rejects_empty_signature_set() -> Result<()> {
        let result = base_builder()?
            .signatures(SignatureSet {
                signatures: Vec::new(),
            })
            .build();
        assert_eq!(result, Err(SkrifheimError::EmptySignatureSet));
        Ok(())
    }
}
