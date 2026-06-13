use alloc::vec::Vec;
use skrifheim_core::{
    ActorId, EntityId, FactId, PolicyId, PredicateId, Result, SecurityLabel, SkrifheimError,
    SourceId, TimeRange, TxId, Value, WorldId,
};
use skrifheim_crypto::SignatureSet;

use crate::{Confidence, FACT_LINK_LIST_MAX_ITEMS, Fact};

#[derive(Clone, Debug, PartialEq)]
pub struct FactBuilder {
    id: Option<FactId>,
    world_id: Option<WorldId>,
    subject: Option<EntityId>,
    predicate: Option<PredicateId>,
    object: Option<Value>,
    valid_time: Option<TimeRange>,
    committed_at: Option<TxId>,
    asserted_by: Option<ActorId>,
    evidence: Vec<SourceId>,
    confidence: Confidence,
    caused_by: Vec<FactId>,
    supersedes: Vec<FactId>,
    invalidates: Vec<FactId>,
    policy_id: Option<PolicyId>,
    label: Option<SecurityLabel>,
    signatures: Option<SignatureSet>,
}

impl FactBuilder {
    #[must_use]
    pub fn new() -> Self {
        Self {
            id: None,
            world_id: None,
            subject: None,
            predicate: None,
            object: None,
            valid_time: None,
            committed_at: None,
            asserted_by: None,
            evidence: Vec::new(),
            confidence: Confidence::zero(),
            caused_by: Vec::new(),
            supersedes: Vec::new(),
            invalidates: Vec::new(),
            policy_id: None,
            label: None,
            signatures: None,
        }
    }

    #[must_use]
    pub const fn id(mut self, id: FactId) -> Self {
        self.id = Some(id);
        self
    }

    #[must_use]
    pub const fn world_id(mut self, world_id: WorldId) -> Self {
        self.world_id = Some(world_id);
        self
    }

    #[must_use]
    pub const fn subject(mut self, subject: EntityId) -> Self {
        self.subject = Some(subject);
        self
    }

    #[must_use]
    pub const fn predicate(mut self, predicate: PredicateId) -> Self {
        self.predicate = Some(predicate);
        self
    }

    #[must_use]
    pub fn object(mut self, object: Value) -> Self {
        self.object = Some(object);
        self
    }

    #[must_use]
    pub const fn valid_time(mut self, valid_time: TimeRange) -> Self {
        self.valid_time = Some(valid_time);
        self
    }

    #[must_use]
    pub const fn committed_at(mut self, committed_at: TxId) -> Self {
        self.committed_at = Some(committed_at);
        self
    }

    #[must_use]
    pub const fn asserted_by(mut self, asserted_by: ActorId) -> Self {
        self.asserted_by = Some(asserted_by);
        self
    }

    pub fn add_evidence(mut self, source: SourceId) -> Result<Self> {
        if self.evidence.len() >= FACT_LINK_LIST_MAX_ITEMS {
            return Err(SkrifheimError::TooManyFactLinks);
        }
        self.evidence.push(source);
        Ok(self)
    }

    pub fn evidence(mut self, evidence: Vec<SourceId>) -> Result<Self> {
        if evidence.len() > FACT_LINK_LIST_MAX_ITEMS {
            return Err(SkrifheimError::TooManyFactLinks);
        }
        self.evidence = dedup(evidence);
        Ok(self)
    }

    #[must_use]
    pub const fn confidence(mut self, confidence: Confidence) -> Self {
        self.confidence = confidence;
        self
    }

    pub fn add_caused_by(mut self, fact_id: FactId) -> Result<Self> {
        if self.caused_by.len() >= FACT_LINK_LIST_MAX_ITEMS {
            return Err(SkrifheimError::TooManyFactLinks);
        }
        self.caused_by.push(fact_id);
        Ok(self)
    }

    pub fn caused_by(mut self, caused_by: Vec<FactId>) -> Result<Self> {
        if caused_by.len() > FACT_LINK_LIST_MAX_ITEMS {
            return Err(SkrifheimError::TooManyFactLinks);
        }
        self.caused_by = dedup(caused_by);
        Ok(self)
    }

    pub fn supersedes(mut self, supersedes: Vec<FactId>) -> Result<Self> {
        if supersedes.len() > FACT_LINK_LIST_MAX_ITEMS {
            return Err(SkrifheimError::TooManyFactLinks);
        }
        self.supersedes = dedup(supersedes);
        Ok(self)
    }

    pub fn invalidates(mut self, invalidates: Vec<FactId>) -> Result<Self> {
        if invalidates.len() > FACT_LINK_LIST_MAX_ITEMS {
            return Err(SkrifheimError::TooManyFactLinks);
        }
        self.invalidates = dedup(invalidates);
        Ok(self)
    }

    #[must_use]
    pub const fn policy_id(mut self, policy_id: PolicyId) -> Self {
        self.policy_id = Some(policy_id);
        self
    }

    #[must_use]
    pub fn label(mut self, label: SecurityLabel) -> Self {
        self.label = Some(label);
        self
    }

    #[must_use]
    pub fn signatures(mut self, signatures: SignatureSet) -> Self {
        self.signatures = Some(signatures);
        self
    }

    pub fn build(self) -> Result<Fact> {
        let fact = Fact {
            id: self.id.ok_or(missing_field("id"))?,
            world_id: self.world_id.ok_or(missing_field("world_id"))?,
            subject: self.subject.ok_or(missing_field("subject"))?,
            predicate: self.predicate.ok_or(missing_field("predicate"))?,
            object: self.object.ok_or(missing_field("object"))?,
            valid_time: self.valid_time.ok_or(missing_field("valid_time"))?,
            committed_at: self.committed_at.ok_or(missing_field("committed_at"))?,
            asserted_by: self.asserted_by.ok_or(missing_field("asserted_by"))?,
            evidence: dedup(self.evidence),
            confidence: self.confidence,
            caused_by: dedup(self.caused_by),
            supersedes: dedup(self.supersedes),
            invalidates: dedup(self.invalidates),
            policy_id: self.policy_id.ok_or(missing_field("policy_id"))?,
            label: self.label.ok_or(missing_field("label"))?,
            signatures: self.signatures.ok_or(missing_field("signatures"))?,
        };
        fact.validate()?;
        Ok(fact)
    }
}

impl Default for FactBuilder {
    fn default() -> Self {
        Self::new()
    }
}

fn missing_field(field: &'static str) -> SkrifheimError {
    SkrifheimError::MissingFactField(field)
}

fn dedup<T: Copy + Ord>(mut values: Vec<T>) -> Vec<T> {
    values.sort();
    values.dedup();
    values
}
