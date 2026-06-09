#![no_std]
#![forbid(unsafe_code)]

extern crate alloc;

use alloc::string::String;
use skrifheim_core::{PolicyId, Result, SkrifheimError, TenantId, TxId};

pub const SEGMENT_MAGIC: [u8; 8] = *b"SKRIFSEG";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SegmentKind {
    Wal,
    Fact,
    Projection,
    Blob,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SegmentHeader {
    pub magic: [u8; 8],
    pub version: u16,
    pub segment_kind: SegmentKind,
    pub tenant_id: TenantId,
    pub min_tx: TxId,
    pub max_tx: TxId,
    pub policy_id: PolicyId,
    pub encryption_key_id: u128,
    pub body_len: u64,
    pub body_crc64: u64,
    pub content_hash: [u8; 32],
}

impl SegmentHeader {
    pub fn validate(&self) -> Result<()> {
        if self.magic != SEGMENT_MAGIC {
            return Err(SkrifheimError::InvalidStorageHeader(String::from(
                "segment magic mismatch",
            )));
        }
        if self.version == 0 {
            return Err(SkrifheimError::InvalidStorageHeader(String::from(
                "segment version must be non-zero",
            )));
        }
        if self.min_tx.0 > self.max_tx.0 {
            return Err(SkrifheimError::InvalidStorageHeader(String::from(
                "min transaction is greater than max transaction",
            )));
        }
        if self.body_len == 0 {
            return Err(SkrifheimError::InvalidStorageHeader(String::from(
                "segment body must not be empty",
            )));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn header() -> SegmentHeader {
        SegmentHeader {
            magic: SEGMENT_MAGIC,
            version: 1,
            segment_kind: SegmentKind::Fact,
            tenant_id: TenantId(1),
            min_tx: TxId(1),
            max_tx: TxId(2),
            policy_id: PolicyId(3),
            encryption_key_id: 4,
            body_len: 5,
            body_crc64: 6,
            content_hash: [7; 32],
        }
    }

    #[test]
    fn valid_header_passes() {
        assert_eq!(header().validate(), Ok(()));
    }

    #[test]
    fn header_rejects_bad_magic() {
        let mut header = header();
        header.magic = *b"WRONGSEG";
        assert!(matches!(
            header.validate(),
            Err(SkrifheimError::InvalidStorageHeader(_))
        ));
    }
}
