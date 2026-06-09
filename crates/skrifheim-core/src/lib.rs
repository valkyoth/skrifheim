#![no_std]
#![forbid(unsafe_code)]

extern crate alloc;

use alloc::{string::String, vec::Vec};
use core::fmt;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct TenantId(pub u128);

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct WorldId(pub u128);

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct FactId(pub u128);

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct EntityId(pub u128);

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct PredicateId(pub u128);

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct PolicyId(pub u128);

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct TxId(pub u128);

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct ActorId(pub u128);

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct SourceId(pub u128);

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
    pub classification: Classification,
    pub compartments: Vec<String>,
    pub releasable_to: Vec<String>,
}

impl SecurityLabel {
    #[must_use]
    pub fn public() -> Self {
        Self {
            classification: Classification::Public,
            compartments: Vec::new(),
            releasable_to: Vec::new(),
        }
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

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum SkrifheimError {
    InvalidTimeRange,
    EmptyEvidence,
    EmptySignatureSet,
    PolicyDenied(String),
    InvalidStorageHeader(String),
}

impl fmt::Display for SkrifheimError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidTimeRange => write!(f, "valid time range ends before it starts"),
            Self::EmptyEvidence => write!(f, "fact must carry at least one evidence source"),
            Self::EmptySignatureSet => write!(f, "commit or fact must carry signatures"),
            Self::PolicyDenied(reason) => write!(f, "policy denied operation: {reason}"),
            Self::InvalidStorageHeader(reason) => write!(f, "invalid storage header: {reason}"),
        }
    }
}

pub type Result<T> = core::result::Result<T, SkrifheimError>;

#[cfg(test)]
mod tests {
    use super::*;

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
}
