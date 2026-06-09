#![no_std]
#![forbid(unsafe_code)]

extern crate alloc;

use alloc::vec::Vec;
use skrifheim_core::{
    ActorId, EntityId, FactId, PolicyId, PredicateId, Result, SecurityLabel, SkrifheimError,
    SourceId, TimeRange, TxId, Value, WorldId,
};
use skrifheim_crypto::SignatureSet;

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
        Ok(Fact {
            id: id(FactId::from_u128(1))?,
            world_id: id(WorldId::from_u128(2))?,
            subject: id(EntityId::from_u128(3))?,
            predicate: id(PredicateId::from_u128(4))?,
            object: Value::Boolean(true),
            valid_time: TimeRange::new(Timestamp(5), None),
            committed_at: id(TxId::from_u128(6))?,
            asserted_by: id(ActorId::from_u128(7))?,
            evidence: vec![id(SourceId::from_u128(8))?],
            confidence: Confidence::clamped(2000),
            caused_by: vec![id(FactId::from_u128(9))?],
            supersedes: Vec::new(),
            invalidates: Vec::new(),
            policy_id: id(PolicyId::from_u128(10))?,
            label: SecurityLabel::new(
                Classification::Restricted,
                vec![String::from("EU-COMMAND")],
                vec![String::from("EU")],
            ),
            signatures: signature_set()?,
        })
    }

    #[test]
    fn valid_fact_passes_validation() -> Result<()> {
        assert_eq!(fact()?.validate(), Ok(()));
        Ok(())
    }

    #[test]
    fn fact_requires_evidence() -> Result<()> {
        let mut fact = fact()?;
        fact.evidence.clear();
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
        let mut fact = fact()?;
        fact.caused_by.push(fact.id);
        assert_eq!(fact.validate(), Err(SkrifheimError::SelfReferentialFact));
        Ok(())
    }
}
