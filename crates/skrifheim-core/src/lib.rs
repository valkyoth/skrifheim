#![no_std]
#![forbid(unsafe_code)]

extern crate alloc;

use alloc::{string::String, vec::Vec};
use core::fmt;
use core::num::NonZeroU128;

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
nonzero_id!(DeviceId);
nonzero_id!(WorkloadId);
nonzero_id!(SourceId);

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Timestamp(pub u64);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TimeRange {
    start: Timestamp,
    end: Option<Timestamp>,
}

impl TimeRange {
    pub const fn new(start: Timestamp, end: Option<Timestamp>) -> Result<Self> {
        if let Some(end) = end
            && end.0 < start.0
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
        if timestamp.0 < self.start.0 {
            return false;
        }
        match self.end {
            Some(end) => timestamp.0 < end.0,
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

#[derive(Clone, Debug, Eq, PartialEq)]
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
}

pub const SECURITY_LABEL_FIXED_STORAGE_BYTES: usize = core::mem::size_of::<SecurityLabel>();

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Value {
    Text(String),
    Integer(i128),
    Boolean(bool),
    Bytes(Vec<u8>),
    Ref(EntityId),
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

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum SkrifheimError {
    InvalidTimeRange,
    InvalidIdentifier,
    InvalidConfidence,
    EmptyEvidence,
    EmptySignatureSet,
    TooManySignatures,
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
    InvalidStorageHeader(String),
    InvalidWorldDiff,
    InvalidKeyHierarchy,
    InvalidKeyLifecycle,
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
            Self::InvalidStorageHeader(reason) => write!(f, "invalid storage header: {reason}"),
            Self::InvalidWorldDiff => write!(f, "target world is not a child of source world"),
            Self::InvalidKeyHierarchy => write!(f, "invalid key hierarchy"),
            Self::InvalidKeyLifecycle => write!(f, "invalid key lifecycle"),
        }
    }
}

impl SkrifheimError {
    #[must_use]
    pub const fn public_message(&self) -> &'static str {
        match self {
            Self::PolicyDenied(_) => "operation denied",
            Self::InvalidStorageHeader(_) => "invalid storage data",
            Self::InvalidSignatureEnvelope(_) => "invalid signature envelope",
            Self::MissingFactField(_) => "invalid fact",
            Self::InvalidTimeRange => "invalid time range",
            Self::InvalidIdentifier => "invalid identifier",
            Self::InvalidConfidence => "invalid confidence",
            Self::EmptyEvidence => "invalid fact",
            Self::EmptySignatureSet => "invalid signature set",
            Self::TooManySignatures => "invalid signature set",
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
        }
    }
}

pub type Result<T> = core::result::Result<T, SkrifheimError>;

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;

    #[test]
    fn open_time_range_contains_later_time() -> Result<()> {
        let range = TimeRange::new(Timestamp(10), None)?;
        assert!(range.contains(Timestamp(11)));
        Ok(())
    }

    #[test]
    fn closed_time_range_excludes_end() -> Result<()> {
        let range = TimeRange::new(Timestamp(10), Some(Timestamp(20)))?;
        assert!(range.contains(Timestamp(19)));
        assert!(!range.contains(Timestamp(20)));
        Ok(())
    }

    #[test]
    fn time_range_rejects_inverted_bounds() {
        assert_eq!(
            TimeRange::new(Timestamp(20), Some(Timestamp(10))),
            Err(SkrifheimError::InvalidTimeRange)
        );
    }

    #[test]
    fn higher_classification_dominates_lower_classification() {
        assert!(Classification::Secret.dominates(Classification::Restricted));
        assert!(!Classification::Internal.dominates(Classification::Secret));
    }

    #[test]
    fn ids_reject_zero_values() {
        assert_eq!(TenantId::from_u128(0), None);
        assert_eq!(TenantId::from_u128(1).map(TenantId::get), Some(1));
    }

    #[test]
    fn security_label_canonicalizes_sets() -> Result<()> {
        let label = SecurityLabel::new(
            Classification::Secret,
            vec![String::from("eu-command"), String::from("EU-COMMAND")],
            vec![String::from("eu")],
        )?;
        assert_eq!(label.compartments().len(), 1);
        assert!(label.compartments().contains("EU-COMMAND"));
        assert!(label.releasable_to().contains("EU"));
        Ok(())
    }

    #[test]
    fn security_label_rejects_oversized_policy_token_sets() {
        let mut compartments = Vec::new();
        for _ in 0..=POLICY_TOKEN_SET_MAX_ITEMS {
            compartments.push(String::from("C"));
        }
        assert_eq!(
            SecurityLabel::new(Classification::Secret, compartments, Vec::new()),
            Err(SkrifheimError::InvalidSecurityToken)
        );
    }

    #[test]
    fn security_label_rejects_unicode_homograph_tokens() {
        assert_eq!(
            SecurityLabel::new(
                Classification::Secret,
                vec![String::from("ЕU-COMMAND")],
                Vec::new(),
            ),
            Err(SkrifheimError::InvalidSecurityToken)
        );
    }

    #[test]
    fn constant_time_policy_token_lookup_is_case_insensitive() -> Result<()> {
        let label = SecurityLabel::new(
            Classification::Secret,
            vec![String::from("eu-command")],
            Vec::new(),
        )?;
        assert!(contains_policy_token_ct(label.compartments(), "EU-COMMAND"));
        assert!(!contains_policy_token_ct(
            label.compartments(),
            "EU-COMMAND-X"
        ));
        Ok(())
    }

    #[test]
    fn constant_time_policy_token_lookup_rejects_malformed_needles() -> Result<()> {
        let label = SecurityLabel::new(
            Classification::Secret,
            vec![String::from("eu-command")],
            Vec::new(),
        )?;
        assert!(!contains_policy_token_ct(
            label.compartments(),
            "EU COMMAND"
        ));
        Ok(())
    }

    #[test]
    fn constant_time_policy_token_lookup_scans_full_token_sets() -> Result<()> {
        let mut tokens = Vec::new();
        for index in 0..POLICY_TOKEN_SET_MAX_ITEMS {
            tokens.push(alloc::format!("TOKEN-{index}"));
        }
        let tokens = PolicyTokenSet::new(tokens)?;
        assert!(contains_policy_token_ct(&tokens, "TOKEN-63"));
        assert!(!contains_policy_token_ct(&tokens, "TOKEN-64"));
        Ok(())
    }

    #[test]
    fn public_message_hides_storage_header_detail() {
        let error = SkrifheimError::InvalidStorageHeader(String::from("segment magic mismatch"));
        assert_eq!(
            alloc::format!("{error}"),
            "invalid storage header: segment magic mismatch"
        );
        assert_eq!(error.public_message(), "invalid storage data");
    }
}
