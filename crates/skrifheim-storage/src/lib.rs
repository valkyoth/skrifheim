#![no_std]
#![forbid(unsafe_code)]

extern crate alloc;

use alloc::string::String;
use skrifheim_core::{PolicyId, Result, SkrifheimError, TenantId, TxId};

pub const SEGMENT_MAGIC: [u8; 8] = *b"SKRIFSEG";
pub const SEGMENT_VERSION_MAX: u16 = 1;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SegmentKind {
    Wal,
    Fact,
    Projection,
    Blob,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BodyChecksum {
    Missing,
    Present(u64),
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
    pub body_crc64: BodyChecksum,
    pub content_hash: Option<[u8; 32]>,
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
        if self.version > SEGMENT_VERSION_MAX {
            return Err(SkrifheimError::InvalidStorageHeader(String::from(
                "segment version is newer than this parser",
            )));
        }
        if self.min_tx.get() > self.max_tx.get() {
            return Err(SkrifheimError::InvalidStorageHeader(String::from(
                "min transaction is greater than max transaction",
            )));
        }
        if self.body_len == 0 {
            return Err(SkrifheimError::InvalidStorageHeader(String::from(
                "segment body must not be empty",
            )));
        }
        if self.body_crc64 == BodyChecksum::Missing {
            return Err(SkrifheimError::InvalidStorageHeader(String::from(
                "body CRC missing",
            )));
        }
        if self.content_hash.is_none() {
            return Err(SkrifheimError::InvalidStorageHeader(String::from(
                "content hash missing",
            )));
        }
        if self.encryption_key_id == 0 {
            return Err(SkrifheimError::InvalidStorageHeader(String::from(
                "encryption key ID must be non-zero",
            )));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id<T>(id: Option<T>) -> Result<T> {
        id.ok_or(SkrifheimError::InvalidIdentifier)
    }

    fn header() -> Result<SegmentHeader> {
        Ok(SegmentHeader {
            magic: SEGMENT_MAGIC,
            version: 1,
            segment_kind: SegmentKind::Fact,
            tenant_id: id(TenantId::from_u128(1))?,
            min_tx: id(TxId::from_u128(1))?,
            max_tx: id(TxId::from_u128(2))?,
            policy_id: id(PolicyId::from_u128(3))?,
            encryption_key_id: 4,
            body_len: 5,
            body_crc64: BodyChecksum::Present(6),
            content_hash: Some([7; 32]),
        })
    }

    #[test]
    fn valid_header_passes() -> Result<()> {
        assert_eq!(header()?.validate(), Ok(()));
        Ok(())
    }

    #[test]
    fn header_rejects_bad_magic() -> Result<()> {
        let mut header = header()?;
        header.magic = *b"WRONGSEG";
        assert!(matches!(
            header.validate(),
            Err(SkrifheimError::InvalidStorageHeader(_))
        ));
        Ok(())
    }

    #[test]
    fn header_rejects_unknown_version() -> Result<()> {
        let mut header = header()?;
        header.version = SEGMENT_VERSION_MAX + 1;
        assert!(matches!(
            header.validate(),
            Err(SkrifheimError::InvalidStorageHeader(_))
        ));
        Ok(())
    }

    #[test]
    fn header_rejects_missing_integrity_fields() -> Result<()> {
        let mut missing_crc = header()?;
        missing_crc.body_crc64 = BodyChecksum::Missing;
        assert!(matches!(
            missing_crc.validate(),
            Err(SkrifheimError::InvalidStorageHeader(_))
        ));

        let mut missing_hash = header()?;
        missing_hash.content_hash = None;
        assert!(matches!(
            missing_hash.validate(),
            Err(SkrifheimError::InvalidStorageHeader(_))
        ));

        let mut missing_key = header()?;
        missing_key.encryption_key_id = 0;
        assert!(matches!(
            missing_key.validate(),
            Err(SkrifheimError::InvalidStorageHeader(_))
        ));
        Ok(())
    }

    #[test]
    fn header_accepts_explicit_zero_content_hash() -> Result<()> {
        let mut header = header()?;
        header.content_hash = Some([0; 32]);
        assert_eq!(header.validate(), Ok(()));
        Ok(())
    }

    #[test]
    fn header_accepts_explicit_zero_body_crc() -> Result<()> {
        let mut header = header()?;
        header.body_crc64 = BodyChecksum::Present(0);
        assert_eq!(header.validate(), Ok(()));
        Ok(())
    }
}
