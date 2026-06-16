#![no_std]
#![forbid(unsafe_code)]

extern crate alloc;

use alloc::{string::String, vec::Vec};
use core::{fmt, num::NonZeroU128};

mod policy_token;

pub use policy_token::{
    POLICY_TOKEN_MAX_BYTES, POLICY_TOKEN_SET_MAX_ITEMS, PolicyTokenSet, PolicyTokenSlot,
    canonical_policy_set, canonical_policy_token, contains_policy_token_ct,
    contains_policy_token_slot_ct,
};

macro_rules! nonzero_id {
    ($(#[$meta:meta])* $name:ident) => {
        $(#[$meta])*
        #[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
        pub struct $name(NonZeroU128);

        impl $name {
            #[must_use]
            pub const fn new(value: NonZeroU128) -> Self {
                Self(value)
            }

            #[must_use]
            pub const fn from_u128(value: u128) -> Option<Self> {
                match NonZeroU128::new(value) {
                    Some(value) => Some(Self(value)),
                    None => None,
                }
            }

            #[must_use]
            pub const fn get(self) -> u128 {
                self.0.get()
            }
        }
    };
}

nonzero_id!(TenantId);
nonzero_id!(
    /// Deterministic, non-secret world namespacing key.
    ///
    /// `WorldId` MUST NOT be used as a bearer capability or relied on for
    /// confidentiality. Authorization is always performed through
    /// `SecurityLabel` and authority context checks in `skrifheim-policy`.
    WorldId
);
nonzero_id!(FactId);
nonzero_id!(EntityId);
nonzero_id!(PredicateId);
nonzero_id!(PolicyId);
nonzero_id!(TxId);
nonzero_id!(ActorId);
nonzero_id!(UserId);
nonzero_id!(DeviceId);
nonzero_id!(WorkloadId);
nonzero_id!(ServiceId);
nonzero_id!(NodeId);
nonzero_id!(ReplicaId);
nonzero_id!(PluginId);
nonzero_id!(AiWorkerId);
nonzero_id!(BackupAgentId);
nonzero_id!(AdminToolId);
nonzero_id!(SourceId);
nonzero_id!(AttestationEvidenceId);
nonzero_id!(AuditEventId);

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Timestamp(u64);

impl Timestamp {
    /// Creates a timestamp value.
    ///
    /// Zero is valid and represents the beginning of the database time domain.
    #[must_use]
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TimeRange {
    start: Timestamp,
    end: Option<Timestamp>,
}

impl TimeRange {
    pub const fn new(start: Timestamp, end: Option<Timestamp>) -> Result<Self> {
        if let Some(end) = end
            && end.get() <= start.get()
        {
            return Err(SkrifheimError::InvalidTimeRange);
        }
        Ok(Self { start, end })
    }

    #[must_use]
    pub const fn start(self) -> Timestamp {
        self.start
    }

    #[must_use]
    pub const fn end(self) -> Option<Timestamp> {
        self.end
    }

    #[must_use]
    pub const fn contains(self, timestamp: Timestamp) -> bool {
        if timestamp.get() < self.start.get() {
            return false;
        }
        match self.end {
            Some(end) => timestamp.get() < end.get(),
            None => true,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum Classification {
    Public,
    Internal,
    Restricted,
    Secret,
    TopSecret,
}

impl Classification {
    #[must_use]
    pub const fn dominates(self, other: Self) -> bool {
        self.rank() >= other.rank()
    }

    const fn rank(self) -> u8 {
        match self {
            Self::Public => 0,
            Self::Internal => 1,
            Self::Restricted => 2,
            Self::Secret => 3,
            Self::TopSecret => 4,
        }
    }
}

#[derive(Clone)]
pub struct SecurityLabel {
    classification: Classification,
    compartments: PolicyTokenSet,
    releasable_to: PolicyTokenSet,
}

impl SecurityLabel {
    #[must_use]
    pub fn public() -> Self {
        Self {
            classification: Classification::Public,
            compartments: PolicyTokenSet::empty(),
            releasable_to: PolicyTokenSet::empty(),
        }
    }

    pub fn new(
        classification: Classification,
        compartments: Vec<String>,
        releasable_to: Vec<String>,
    ) -> Result<Self> {
        Ok(Self {
            classification,
            compartments: canonical_policy_set(compartments)?,
            releasable_to: canonical_policy_set(releasable_to)?,
        })
    }

    #[must_use]
    pub const fn classification(&self) -> Classification {
        self.classification
    }

    #[must_use]
    pub const fn compartments(&self) -> &PolicyTokenSet {
        &self.compartments
    }

    #[must_use]
    pub const fn releasable_to(&self) -> &PolicyTokenSet {
        &self.releasable_to
    }

    /// Non-constant-time structural comparison; not for policy decisions.
    #[must_use]
    pub fn structurally_equal(&self, other: &Self) -> bool {
        self.classification == other.classification
            && self.compartments.structurally_equal(&other.compartments)
            && self.releasable_to.structurally_equal(&other.releasable_to)
    }
}

impl fmt::Debug for SecurityLabel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SecurityLabel")
            .field("classification", &"<redacted>")
            .field("compartment_count", &"<redacted>")
            .field("releasable_to_count", &"<redacted>")
            .field("tokens", &"<redacted>")
            .finish()
    }
}

pub const SECURITY_LABEL_FIXED_STORAGE_BYTES: usize = core::mem::size_of::<SecurityLabel>();

#[derive(Clone, Eq, PartialEq)]
pub enum Value {
    Text(String),
    Integer(i128),
    Boolean(bool),
    Bytes(Vec<u8>),
    Ref(EntityId),
}

impl fmt::Debug for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Text(value) => f
                .debug_struct("Text")
                .field("bytes", &value.len())
                .field("content", &"<redacted>")
                .finish(),
            Self::Integer(_) => f.write_str("Integer(<redacted>)"),
            Self::Boolean(_) => f.write_str("Boolean(<redacted>)"),
            Self::Bytes(value) => f
                .debug_struct("Bytes")
                .field("bytes", &value.len())
                .field("content", &"<redacted>")
                .finish(),
            Self::Ref(_) => f.write_str("Ref(<redacted>)"),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AccessDeniedReason(());

impl AccessDeniedReason {
    #[must_use]
    pub const fn new() -> Self {
        Self(())
    }
}

impl Default for AccessDeniedReason {
    fn default() -> Self {
        Self::new()
    }
}

/// Internal error type for skrifheim crates.
///
/// `Display` is diagnostic and may include implementation details for internal
/// logs. Any message that crosses a trust, tenant, classification, process, or
/// network boundary must use [`SkrifheimError::public_message`] instead.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum SkrifheimError {
    InvalidTimeRange,
    InvalidIdentifier,
    InvalidConfidence,
    EmptyEvidence,
    EmptySignatureSet,
    TooManySignatures,
    DuplicateSignatureSigner,
    MissingFactField(&'static str),
    DuplicateEvidence,
    DuplicateFactLink,
    TooManyFactLinks,
    ValueTooLarge,
    InvalidSignatureEnvelope(&'static str),
    InvalidSignatureLength,
    InvalidSecurityToken,
    InvalidWorldName,
    InvalidWorldIdentity,
    InvalidQueryRequest,
    SelfReferentialFact,
    PolicyDenied(AccessDeniedReason),
    PolicyRedacted(AccessDeniedReason),
    InvalidStorageHeader(String),
    InvalidWorldDiff,
    InvalidKeyHierarchy,
    InvalidKeyLifecycle,
    InvalidProjectionPolicy,
    InvalidSecretMaterial,
    MissingAuditActor,
    InvalidAttestationEvidence,
    InvalidAuditEvent,
    InvalidAuditProtection,
}

impl fmt::Display for SkrifheimError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidTimeRange => write!(f, "valid time range ends before it starts"),
            Self::InvalidIdentifier => write!(f, "identifier must be non-zero"),
            Self::InvalidConfidence => write!(f, "confidence must be in range 0..=1000"),
            Self::EmptyEvidence => write!(f, "fact must carry at least one evidence source"),
            Self::EmptySignatureSet => write!(f, "commit or fact must carry signatures"),
            Self::TooManySignatures => write!(f, "signature set contains too many signatures"),
            Self::DuplicateSignatureSigner => write!(f, "signature set contains duplicate signers"),
            Self::MissingFactField(field) => write!(f, "fact builder is missing field: {field}"),
            Self::DuplicateEvidence => write!(f, "fact evidence must not contain duplicates"),
            Self::DuplicateFactLink => write!(f, "fact links must not contain duplicates"),
            Self::TooManyFactLinks => write!(f, "fact link list is too large"),
            Self::ValueTooLarge => write!(f, "fact value is too large"),
            Self::InvalidSignatureEnvelope(reason) => {
                write!(f, "invalid signature envelope: {reason}")
            }
            Self::InvalidSignatureLength => write!(f, "invalid signature length"),
            Self::InvalidSecurityToken => write!(f, "invalid security token"),
            Self::InvalidWorldName => write!(f, "invalid world name"),
            Self::InvalidWorldIdentity => write!(f, "invalid world identity"),
            Self::InvalidQueryRequest => write!(f, "invalid query request"),
            Self::SelfReferentialFact => write!(f, "fact cannot refer to itself causally"),
            Self::PolicyDenied(_) => write!(f, "policy denied operation: access denied"),
            Self::PolicyRedacted(_) => write!(f, "policy redacted operation: access denied"),
            Self::InvalidStorageHeader(reason) => write!(f, "invalid storage header: {reason}"),
            Self::InvalidWorldDiff => write!(f, "target world is not a child of source world"),
            Self::InvalidKeyHierarchy => write!(f, "invalid key hierarchy"),
            Self::InvalidKeyLifecycle => write!(f, "invalid key lifecycle"),
            Self::InvalidProjectionPolicy => write!(f, "invalid projection encryption policy"),
            Self::InvalidSecretMaterial => write!(f, "invalid secret material"),
            Self::MissingAuditActor => write!(f, "audit event is missing required actor"),
            Self::InvalidAttestationEvidence => write!(f, "invalid attestation evidence"),
            Self::InvalidAuditEvent => write!(f, "invalid audit event"),
            Self::InvalidAuditProtection => write!(f, "invalid audit protection"),
        }
    }
}

impl SkrifheimError {
    #[must_use]
    pub const fn public_message(&self) -> &'static str {
        match self {
            Self::PolicyDenied(_) | Self::PolicyRedacted(_) => "operation denied",
            Self::InvalidStorageHeader(_) => "invalid storage data",
            Self::InvalidSignatureEnvelope(_) => "invalid signature envelope",
            Self::MissingFactField(_) => "invalid fact",
            Self::InvalidTimeRange => "invalid time range",
            Self::InvalidIdentifier => "invalid identifier",
            Self::InvalidConfidence => "invalid confidence",
            Self::EmptyEvidence => "invalid fact",
            Self::EmptySignatureSet => "invalid signature set",
            Self::TooManySignatures => "invalid signature set",
            Self::DuplicateSignatureSigner => "invalid signature set",
            Self::DuplicateEvidence => "invalid fact",
            Self::DuplicateFactLink => "invalid fact",
            Self::TooManyFactLinks => "invalid fact",
            Self::ValueTooLarge => "invalid fact",
            Self::InvalidSignatureLength => "invalid signature length",
            Self::InvalidSecurityToken => "invalid security token",
            Self::InvalidWorldName => "invalid world name",
            Self::InvalidWorldIdentity => "invalid world identity",
            Self::InvalidQueryRequest => "invalid query request",
            Self::SelfReferentialFact => "invalid fact",
            Self::InvalidWorldDiff => "invalid world diff",
            Self::InvalidKeyHierarchy => "invalid key hierarchy",
            Self::InvalidKeyLifecycle => "invalid key lifecycle",
            Self::InvalidProjectionPolicy => "invalid projection policy",
            Self::InvalidSecretMaterial => "invalid secret material",
            Self::MissingAuditActor => "invalid audit event",
            Self::InvalidAttestationEvidence => "invalid attestation evidence",
            Self::InvalidAuditEvent => "invalid audit event",
            Self::InvalidAuditProtection => "invalid audit protection",
        }
    }
}

pub type Result<T> = core::result::Result<T, SkrifheimError>;

#[cfg(test)]
mod tests;
