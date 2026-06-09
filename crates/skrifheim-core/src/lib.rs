#![no_std]
#![forbid(unsafe_code)]

extern crate alloc;

use alloc::{collections::BTreeSet, string::String, vec::Vec};
use core::fmt;
use core::num::NonZeroU128;

mod policy_token;

pub use policy_token::{
    POLICY_TOKEN_MAX_BYTES, canonical_policy_set, canonical_policy_token, contains_policy_token_ct,
};

macro_rules! nonzero_id {
    ($name:ident) => {
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
nonzero_id!(WorldId);
nonzero_id!(FactId);
nonzero_id!(EntityId);
nonzero_id!(PredicateId);
nonzero_id!(PolicyId);
nonzero_id!(TxId);
nonzero_id!(ActorId);
nonzero_id!(SourceId);

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Timestamp(pub u64);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TimeRange {
    pub start: Timestamp,
    pub end: Option<Timestamp>,
}

impl TimeRange {
    #[must_use]
    pub const fn new(start: Timestamp, end: Option<Timestamp>) -> Self {
        Self { start, end }
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
    compartments: BTreeSet<String>,
    releasable_to: BTreeSet<String>,
}

impl SecurityLabel {
    #[must_use]
    pub fn public() -> Self {
        Self {
            classification: Classification::Public,
            compartments: BTreeSet::new(),
            releasable_to: BTreeSet::new(),
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
    pub const fn compartments(&self) -> &BTreeSet<String> {
        &self.compartments
    }

    #[must_use]
    pub const fn releasable_to(&self) -> &BTreeSet<String> {
        &self.releasable_to
    }
}

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
    MissingFactField(&'static str),
    DuplicateEvidence,
    DuplicateFactLink,
    TooManyFactLinks,
    InvalidSignatureEnvelope(&'static str),
    InvalidSignatureLength,
    InvalidSecurityToken,
    InvalidWorldName,
    SelfReferentialFact,
    PolicyDenied(AccessDeniedReason),
    InvalidStorageHeader(String),
    InvalidWorldDiff,
}

impl fmt::Display for SkrifheimError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidTimeRange => write!(f, "valid time range ends before it starts"),
            Self::InvalidIdentifier => write!(f, "identifier must be non-zero"),
            Self::InvalidConfidence => write!(f, "confidence must be in range 0..=1000"),
            Self::EmptyEvidence => write!(f, "fact must carry at least one evidence source"),
            Self::EmptySignatureSet => write!(f, "commit or fact must carry signatures"),
            Self::MissingFactField(field) => write!(f, "fact builder is missing field: {field}"),
            Self::DuplicateEvidence => write!(f, "fact evidence must not contain duplicates"),
            Self::DuplicateFactLink => write!(f, "fact links must not contain duplicates"),
            Self::TooManyFactLinks => write!(f, "fact link list is too large"),
            Self::InvalidSignatureEnvelope(reason) => {
                write!(f, "invalid signature envelope: {reason}")
            }
            Self::InvalidSignatureLength => write!(f, "invalid signature length"),
            Self::InvalidSecurityToken => write!(f, "invalid security token"),
            Self::InvalidWorldName => write!(f, "invalid world name"),
            Self::SelfReferentialFact => write!(f, "fact cannot refer to itself causally"),
            Self::PolicyDenied(_) => write!(f, "policy denied operation: access denied"),
            Self::InvalidStorageHeader(reason) => write!(f, "invalid storage header: {reason}"),
            Self::InvalidWorldDiff => write!(f, "target world is not a child of source world"),
        }
    }
}

pub type Result<T> = core::result::Result<T, SkrifheimError>;

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;

    #[test]
    fn open_time_range_contains_later_time() {
        let range = TimeRange::new(Timestamp(10), None);
        assert!(range.contains(Timestamp(11)));
    }

    #[test]
    fn closed_time_range_excludes_end() {
        let range = TimeRange::new(Timestamp(10), Some(Timestamp(20)));
        assert!(range.contains(Timestamp(19)));
        assert!(!range.contains(Timestamp(20)));
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
}
