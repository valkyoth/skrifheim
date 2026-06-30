#![no_std]
#![forbid(unsafe_code)]

extern crate alloc;

use alloc::{collections::BTreeSet, vec::Vec};
use core::fmt;
use skrifheim_core::{
    ActorId, EntityId, FactId, PolicyId, PredicateId, Result, SecurityLabel, SkrifheimError,
    SourceId, TimeRange, TxId, Value, WorldId,
};
use skrifheim_crypto::SignatureSet;

pub const FACT_LINK_LIST_MAX_ITEMS: usize = 1024;
pub const FACT_OBJECT_MAX_BYTES: usize = 64 * 1024;

mod builder;
#[cfg(test)]
mod tests;

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
    pub const fn get(self) -> u16 {
        self.0
    }
}

#[derive(Clone)]
pub struct Fact {
    id: FactId,
    world_id: WorldId,
    subject: EntityId,
    predicate: PredicateId,
    object: Value,
    valid_time: TimeRange,
    committed_at: TxId,
    asserted_by: ActorId,
    evidence: Vec<SourceId>,
    confidence: Confidence,
    caused_by: Vec<FactId>,
    supersedes: Vec<FactId>,
    invalidates: Vec<FactId>,
    policy_id: PolicyId,
    label: SecurityLabel,
    signatures: SignatureSet,
}

impl fmt::Debug for Fact {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Fact")
            .field("id", &"<redacted>")
            .field("world_id", &"<redacted>")
            .field("subject", &"<redacted>")
            .field("predicate", &"<redacted>")
            .field("object", &"<redacted>")
            .field("valid_time", &"<redacted>")
            .field("committed_at", &"<redacted>")
            .field("asserted_by", &"<redacted>")
            .field("evidence_count", &self.evidence.len())
            .field("confidence", &"<redacted>")
            .field("causal_link_sets", &"<redacted>")
            .field("policy_id", &"<redacted>")
            .field("classification", &"<redacted>")
            .field("signature_count", &self.signatures.envelopes().len())
            .finish()
    }
}

impl Fact {
    #[must_use]
    pub fn builder() -> FactBuilder {
        FactBuilder::new()
    }

    /// Validates structural invariants only; this is not cryptographic
    /// authenticity verification.
    ///
    /// Callers that accept facts from untrusted sources, such as future segment
    /// deserialization or network ingestion paths, must separately verify
    /// signatures against the key registry before treating a fact as committed.
    pub fn validate_structure(&self) -> Result<()> {
        validate_object_size(&self.object)?;
        if self.evidence.is_empty() {
            return Err(SkrifheimError::EmptyEvidence);
        }
        if self.evidence.len() > FACT_LINK_LIST_MAX_ITEMS
            || self.caused_by.len() > FACT_LINK_LIST_MAX_ITEMS
            || self.supersedes.len() > FACT_LINK_LIST_MAX_ITEMS
            || self.invalidates.len() > FACT_LINK_LIST_MAX_ITEMS
        {
            return Err(SkrifheimError::TooManyFactLinks);
        }
        if has_duplicates(&self.evidence) {
            return Err(SkrifheimError::DuplicateEvidence);
        }
        if has_duplicates(&self.caused_by)
            || has_duplicates(&self.supersedes)
            || has_duplicates(&self.invalidates)
        {
            return Err(SkrifheimError::DuplicateFactLink);
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

    #[must_use]
    pub const fn id(&self) -> FactId {
        self.id
    }

    #[must_use]
    pub const fn world_id(&self) -> WorldId {
        self.world_id
    }

    #[must_use]
    pub const fn subject(&self) -> EntityId {
        self.subject
    }

    #[must_use]
    pub const fn predicate(&self) -> PredicateId {
        self.predicate
    }

    #[must_use]
    pub const fn object(&self) -> &Value {
        &self.object
    }

    #[must_use]
    pub const fn valid_time(&self) -> TimeRange {
        self.valid_time
    }

    #[must_use]
    pub const fn committed_at(&self) -> TxId {
        self.committed_at
    }

    #[must_use]
    pub const fn asserted_by(&self) -> ActorId {
        self.asserted_by
    }

    #[must_use]
    pub fn evidence(&self) -> &[SourceId] {
        &self.evidence
    }

    #[must_use]
    pub const fn confidence(&self) -> Confidence {
        self.confidence
    }

    #[must_use]
    pub fn caused_by(&self) -> &[FactId] {
        &self.caused_by
    }

    #[must_use]
    pub fn supersedes(&self) -> &[FactId] {
        &self.supersedes
    }

    #[must_use]
    pub fn invalidates(&self) -> &[FactId] {
        &self.invalidates
    }

    #[must_use]
    pub const fn policy_id(&self) -> PolicyId {
        self.policy_id
    }

    #[must_use]
    pub const fn label(&self) -> &SecurityLabel {
        &self.label
    }

    #[must_use]
    pub const fn signatures(&self) -> &SignatureSet {
        &self.signatures
    }
}

pub(crate) fn validate_object_size(object: &Value) -> Result<()> {
    let len = match object {
        Value::Text(value) => value.len(),
        Value::Bytes(value) => value.len(),
        Value::Integer(_) | Value::Boolean(_) | Value::Ref(_) => 0,
    };
    if len > FACT_OBJECT_MAX_BYTES {
        return Err(SkrifheimError::ValueTooLarge);
    }
    Ok(())
}

fn has_duplicates<T: Copy + Ord>(values: &[T]) -> bool {
    let mut seen = BTreeSet::new();
    for value in values {
        if !seen.insert(*value) {
            return true;
        }
    }
    false
}
