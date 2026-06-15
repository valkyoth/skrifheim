#![no_std]
#![forbid(unsafe_code)]

extern crate alloc;

use alloc::string::String;
use skrifheim_core::{PolicyId, Result, SkrifheimError, TenantId, TxId};
use skrifheim_crypto::KeyId;

pub const SEGMENT_MAGIC: [u8; 8] = *b"SKRIFSEG";
pub const SEGMENT_VERSION_MAX: u16 = 1;
pub const SEGMENT_BODY_MAX_BYTES: u64 = 1024 * 1024 * 1024;

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
    pub encryption_key_id: KeyId,
    pub body_len: u64,
    pub body_crc64: BodyChecksum,
    pub content_hash: Option<[u8; 32]>,
}

impl SegmentHeader {
    /// Validates only the segment header's structural metadata.
    ///
    /// This does not recompute or compare the body CRC/hash against segment body
    /// bytes. Future readers with access to the body must verify both before
    /// accepting body contents as tamper-evident.
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
        if self.body_len > SEGMENT_BODY_MAX_BYTES {
            return Err(SkrifheimError::InvalidStorageHeader(String::from(
                "segment body exceeds maximum size",
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
            encryption_key_id: id(KeyId::from_u128(4))?,
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

        Ok(())
    }

    #[test]
    fn header_encryption_key_id_is_typed_nonzero() -> Result<()> {
        assert_eq!(KeyId::from_u128(0), None);
        assert_eq!(header()?.encryption_key_id.get(), 4);
        Ok(())
    }

    #[test]
    fn header_rejects_oversized_body() -> Result<()> {
        let mut header = header()?;
        header.body_len = SEGMENT_BODY_MAX_BYTES + 1;
        assert!(matches!(
            header.validate(),
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
