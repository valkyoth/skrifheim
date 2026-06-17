#![no_std]
#![forbid(unsafe_code)]

extern crate alloc;

use alloc::string::String;
use core::fmt;
use skrifheim_core::{PolicyId, Result, SkrifheimError, TenantId, TxId};
use skrifheim_crypto::KeyId;

mod wal;

pub use wal::{
    WAL_FRAME_BODY_MAX_BYTES, WAL_FRAME_HEADER_BYTES, WAL_FRAME_MAGIC, WAL_FRAME_VERSION_MAX,
    WalFrameHeader, WalFrameHeaderInput, WalRecordKind,
};

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

#[derive(Clone, Eq, PartialEq)]
pub struct SegmentHeader {
    magic: [u8; 8],
    version: u16,
    segment_kind: SegmentKind,
    tenant_id: TenantId,
    min_tx: TxId,
    max_tx: TxId,
    policy_id: PolicyId,
    encryption_key_id: KeyId,
    body_len: u64,
    body_crc64: BodyChecksum,
    content_hash: Option<[u8; 32]>,
}

impl fmt::Debug for SegmentHeader {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SegmentHeader")
            .field("magic", &"<redacted>")
            .field("version", &self.version)
            .field("segment_kind", &self.segment_kind)
            .field("tenant_id", &"<redacted>")
            .field("tx_range", &"<redacted>")
            .field("policy_id", &"<redacted>")
            .field("encryption_key_id", &"<redacted>")
            .field("body_len", &self.body_len)
            .field("body_crc64", &"<redacted>")
            .field("content_hash", &"<redacted>")
            .finish()
    }
}

#[derive(Clone, Eq, PartialEq)]
pub struct SegmentHeaderInput {
    pub segment_kind: SegmentKind,
    pub tenant_id: TenantId,
    pub min_tx: TxId,
    pub max_tx: TxId,
    pub policy_id: PolicyId,
    pub encryption_key_id: KeyId,
    pub body_len: u64,
    pub body_crc64: BodyChecksum,
    pub content_hash: [u8; 32],
}

impl fmt::Debug for SegmentHeaderInput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SegmentHeaderInput")
            .field("segment_kind", &self.segment_kind)
            .field("tenant_id", &"<redacted>")
            .field("tx_range", &"<redacted>")
            .field("policy_id", &"<redacted>")
            .field("encryption_key_id", &"<redacted>")
            .field("body_len", &self.body_len)
            .field("body_crc64", &"<redacted>")
            .field("content_hash", &"<redacted>")
            .finish()
    }
}

impl SegmentHeader {
    pub fn new(input: SegmentHeaderInput) -> Result<Self> {
        let header = Self {
            magic: SEGMENT_MAGIC,
            version: SEGMENT_VERSION_MAX,
            segment_kind: input.segment_kind,
            tenant_id: input.tenant_id,
            min_tx: input.min_tx,
            max_tx: input.max_tx,
            policy_id: input.policy_id,
            encryption_key_id: input.encryption_key_id,
            body_len: input.body_len,
            body_crc64: input.body_crc64,
            content_hash: Some(input.content_hash),
        };
        header.validate()?;
        Ok(header)
    }

    #[must_use]
    pub const fn magic(&self) -> [u8; 8] {
        self.magic
    }

    #[must_use]
    pub const fn version(&self) -> u16 {
        self.version
    }

    #[must_use]
    pub const fn segment_kind(&self) -> SegmentKind {
        self.segment_kind
    }

    #[must_use]
    pub const fn tenant_id(&self) -> TenantId {
        self.tenant_id
    }

    #[must_use]
    pub const fn min_tx(&self) -> TxId {
        self.min_tx
    }

    #[must_use]
    pub const fn max_tx(&self) -> TxId {
        self.max_tx
    }

    #[must_use]
    pub const fn policy_id(&self) -> PolicyId {
        self.policy_id
    }

    #[must_use]
    pub const fn encryption_key_id(&self) -> KeyId {
        self.encryption_key_id
    }

    #[must_use]
    pub const fn body_len(&self) -> u64 {
        self.body_len
    }

    #[must_use]
    pub const fn body_crc64(&self) -> BodyChecksum {
        self.body_crc64
    }

    #[must_use]
    pub const fn content_hash(&self) -> Option<[u8; 32]> {
        self.content_hash
    }

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
        SegmentHeader::new(header_input()?)
    }

    fn header_input() -> Result<SegmentHeaderInput> {
        Ok(SegmentHeaderInput {
            segment_kind: SegmentKind::Fact,
            tenant_id: id(TenantId::from_u128(1))?,
            min_tx: id(TxId::from_u128(1))?,
            max_tx: id(TxId::from_u128(2))?,
            policy_id: id(PolicyId::from_u128(3))?,
            encryption_key_id: id(KeyId::from_u128(4))?,
            body_len: 5,
            body_crc64: BodyChecksum::Present(6),
            content_hash: [7; 32],
        })
    }

    #[test]
    fn valid_header_passes() -> Result<()> {
        assert_eq!(header()?.validate(), Ok(()));
        Ok(())
    }

    #[test]
    fn constructor_sets_header_identity_fields() -> Result<()> {
        let header = header()?;

        assert_eq!(header.magic(), SEGMENT_MAGIC);
        assert_eq!(header.version(), SEGMENT_VERSION_MAX);
        assert_eq!(header.segment_kind(), SegmentKind::Fact);
        assert_eq!(header.tenant_id().get(), 1);
        assert_eq!(header.min_tx().get(), 1);
        assert_eq!(header.max_tx().get(), 2);
        assert_eq!(header.policy_id().get(), 3);
        assert_eq!(header.body_len(), 5);
        assert_eq!(header.body_crc64(), BodyChecksum::Present(6));
        assert_eq!(header.content_hash(), Some([7; 32]));
        Ok(())
    }

    #[test]
    fn header_rejects_inverted_transaction_range() -> Result<()> {
        let mut input = header_input()?;
        input.min_tx = id(TxId::from_u128(3))?;
        input.max_tx = id(TxId::from_u128(2))?;

        assert!(matches!(
            SegmentHeader::new(input),
            Err(SkrifheimError::InvalidStorageHeader(_))
        ));
        Ok(())
    }

    #[test]
    fn header_rejects_missing_integrity_fields() -> Result<()> {
        let mut input = header_input()?;
        input.body_crc64 = BodyChecksum::Missing;

        assert!(matches!(
            SegmentHeader::new(input),
            Err(SkrifheimError::InvalidStorageHeader(_))
        ));

        Ok(())
    }

    #[test]
    fn header_encryption_key_id_is_typed_nonzero() -> Result<()> {
        assert_eq!(KeyId::from_u128(0), None);
        assert_eq!(header()?.encryption_key_id().get(), 4);
        Ok(())
    }

    #[test]
    fn header_rejects_oversized_body() -> Result<()> {
        let mut input = header_input()?;
        input.body_len = SEGMENT_BODY_MAX_BYTES + 1;

        assert!(matches!(
            SegmentHeader::new(input),
            Err(SkrifheimError::InvalidStorageHeader(_))
        ));
        Ok(())
    }

    #[test]
    fn header_accepts_explicit_zero_content_hash() -> Result<()> {
        let mut input = header_input()?;
        input.content_hash = [0; 32];
        let header = SegmentHeader::new(input)?;
        assert_eq!(header.validate(), Ok(()));
        Ok(())
    }

    #[test]
    fn header_accepts_explicit_zero_body_crc() -> Result<()> {
        let mut input = header_input()?;
        input.body_crc64 = BodyChecksum::Present(0);
        let header = SegmentHeader::new(input)?;
        assert_eq!(header.validate(), Ok(()));
        Ok(())
    }

    #[test]
    fn debug_redacts_segment_header_sensitive_metadata() -> Result<()> {
        let input = header_input()?;
        let input_debug = alloc::format!("{input:?}");
        let header_debug = alloc::format!("{:?}", SegmentHeader::new(input)?);

        assert!(input_debug.contains("encryption_key_id: \"<redacted>\""));
        assert!(input_debug.contains("content_hash: \"<redacted>\""));
        assert!(header_debug.contains("encryption_key_id: \"<redacted>\""));
        assert!(header_debug.contains("content_hash: \"<redacted>\""));
        assert!(!input_debug.contains("KeyId"));
        assert!(!input_debug.contains("[7, 7"));
        assert!(!header_debug.contains("KeyId"));
        assert!(!header_debug.contains("[7, 7"));
        Ok(())
    }
}
