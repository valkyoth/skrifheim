#![no_std]
#![forbid(unsafe_code)]

extern crate alloc;

use alloc::vec::Vec;
use skrifheim_core::{
    ActorId, EntityId, FactId, PolicyId, PredicateId, Result, SecurityLabel, SkrifheimError,
    SourceId, TimeRange, TxId, Value, WorldId,
};
use skrifheim_crypto::SignatureSet;

#[derive(Clone, Debug, PartialEq)]
pub struct Confidence(pub f32);

impl Confidence {
    #[must_use]
    pub fn clamped(value: f32) -> Self {
        Self(value.clamp(0.0, 1.0))
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

    fn signature_set() -> SignatureSet {
        SignatureSet {
            signatures: vec![SignatureEnvelope::new(
                AlgorithmId::Ed25519,
                CryptoEpoch(1),
                "test-key",
                vec![1, 2, 3],
            )],
        }
    }

    fn fact() -> Fact {
        Fact {
            id: FactId(1),
            world_id: WorldId(2),
            subject: EntityId(3),
            predicate: PredicateId(4),
            object: Value::Boolean(true),
            valid_time: TimeRange::new(Timestamp(5), None),
            committed_at: TxId(6),
            asserted_by: ActorId(7),
            evidence: vec![SourceId(8)],
            confidence: Confidence::clamped(2.0),
            caused_by: vec![FactId(9)],
            supersedes: Vec::new(),
            invalidates: Vec::new(),
            policy_id: PolicyId(10),
            label: SecurityLabel {
                classification: Classification::Restricted,
                compartments: vec![String::from("EU-COMMAND")],
                releasable_to: vec![String::from("EU")],
            },
            signatures: signature_set(),
        }
    }

    #[test]
    fn valid_fact_passes_validation() {
        assert_eq!(fact().validate(), Ok(()));
    }

    #[test]
    fn fact_requires_evidence() {
        let mut fact = fact();
        fact.evidence.clear();
        assert_eq!(fact.validate(), Err(SkrifheimError::EmptyEvidence));
    }

    #[test]
    fn confidence_is_clamped() {
        assert_eq!(Confidence::clamped(2.0), Confidence(1.0));
        assert_eq!(Confidence::clamped(-2.0), Confidence(0.0));
    }
}
