use core::fmt;

use skrifheim_core::{PolicyId, Result, TenantId, TxId};
use skrifheim_crypto::{ContentDigest, CryptoEpoch, EncryptionDomain, KeyId};

mod encoding;
mod validation;

use encoding::{encode_footer, encode_header, parse_footer, parse_header};
use validation::{
    SegmentValidationInput, content_digest_matches, invalid_segment, validate_segment_metadata,
    validate_version,
};

pub const SEGMENT_MAGIC: [u8; 8] = *b"SKRIFSEG";
pub const SEGMENT_FOOTER_MAGIC: [u8; 8] = *b"SKRIFFTR";
pub const SEGMENT_VERSION_MAX: u16 = 1;
pub const SEGMENT_FOOTER_VERSION_MAX: u16 = 2;
pub const SEGMENT_HEADER_BYTES: usize = 256;
pub const SEGMENT_FOOTER_BYTES: usize = 256;
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

#[derive(Clone)]
pub struct SegmentHeader {
    magic: [u8; 8],
    version: u16,
    segment_kind: SegmentKind,
    tenant_id: TenantId,
    min_tx: TxId,
    max_tx: TxId,
    policy_id: PolicyId,
    encryption_key_id: KeyId,
    crypto_epoch: CryptoEpoch,
    encryption_domain: EncryptionDomain,
    body_len: u64,
    body_crc64: BodyChecksum,
    content_digest: Option<ContentDigest>,
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
            .field("crypto_epoch", &"<redacted>")
            .field("encryption_domain", &"<redacted>")
            .field("body_len", &self.body_len)
            .field("body_crc64", &"<redacted>")
            .field("content_digest", &"<redacted>")
            .finish()
    }
}

#[derive(Clone)]
pub struct SegmentHeaderInput {
    pub segment_kind: SegmentKind,
    pub tenant_id: TenantId,
    pub min_tx: TxId,
    pub max_tx: TxId,
    pub policy_id: PolicyId,
    pub encryption_key_id: KeyId,
    pub crypto_epoch: CryptoEpoch,
    pub encryption_domain: EncryptionDomain,
    pub body_len: u64,
    pub body_crc64: BodyChecksum,
    pub content_digest: ContentDigest,
}

impl fmt::Debug for SegmentHeaderInput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SegmentHeaderInput")
            .field("segment_kind", &self.segment_kind)
            .field("tenant_id", &"<redacted>")
            .field("tx_range", &"<redacted>")
            .field("policy_id", &"<redacted>")
            .field("encryption_key_id", &"<redacted>")
            .field("crypto_epoch", &"<redacted>")
            .field("encryption_domain", &"<redacted>")
            .field("body_len", &self.body_len)
            .field("body_crc64", &"<redacted>")
            .field("content_digest", &"<redacted>")
            .finish()
    }
}

#[derive(Clone)]
pub struct SegmentFooter {
    magic: [u8; 8],
    version: u16,
    segment_kind: SegmentKind,
    tenant_id: TenantId,
    min_tx: TxId,
    max_tx: TxId,
    policy_id: PolicyId,
    encryption_key_id: KeyId,
    crypto_epoch: CryptoEpoch,
    encryption_domain: EncryptionDomain,
    body_len: u64,
    body_crc64: BodyChecksum,
    content_digest: Option<ContentDigest>,
}

impl fmt::Debug for SegmentFooter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SegmentFooter")
            .field("magic", &"<redacted>")
            .field("version", &self.version)
            .field("segment_kind", &self.segment_kind)
            .field("tenant_id", &"<redacted>")
            .field("tx_range", &"<redacted>")
            .field("policy_id", &"<redacted>")
            .field("encryption_key_id", &"<redacted>")
            .field("crypto_epoch", &"<redacted>")
            .field("encryption_domain", &"<redacted>")
            .field("body_len", &self.body_len)
            .field("body_crc64", &"<redacted>")
            .field("content_digest", &"<redacted>")
            .finish()
    }
}

#[derive(Clone)]
pub struct SegmentFooterInput {
    pub segment_kind: SegmentKind,
    pub tenant_id: TenantId,
    pub min_tx: TxId,
    pub max_tx: TxId,
    pub policy_id: PolicyId,
    pub encryption_key_id: KeyId,
    pub crypto_epoch: CryptoEpoch,
    pub encryption_domain: EncryptionDomain,
    pub body_len: u64,
    pub body_crc64: BodyChecksum,
    pub content_digest: ContentDigest,
}

impl fmt::Debug for SegmentFooterInput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SegmentFooterInput")
            .field("segment_kind", &self.segment_kind)
            .field("tenant_id", &"<redacted>")
            .field("tx_range", &"<redacted>")
            .field("policy_id", &"<redacted>")
            .field("encryption_key_id", &"<redacted>")
            .field("crypto_epoch", &"<redacted>")
            .field("encryption_domain", &"<redacted>")
            .field("body_len", &self.body_len)
            .field("body_crc64", &"<redacted>")
            .field("content_digest", &"<redacted>")
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
            crypto_epoch: input.crypto_epoch,
            encryption_domain: input.encryption_domain,
            body_len: input.body_len,
            body_crc64: input.body_crc64,
            content_digest: Some(input.content_digest),
        };
        header.validate()?;
        Ok(header)
    }

    pub fn parse(bytes: &[u8]) -> Result<Self> {
        parse_header(bytes)
    }

    pub fn parse_for_domain(bytes: &[u8], expected_domain: EncryptionDomain) -> Result<Self> {
        let header = Self::parse(bytes)?;
        header.validate_for_domain(expected_domain)?;
        Ok(header)
    }

    #[must_use]
    pub fn encode(&self) -> [u8; SEGMENT_HEADER_BYTES] {
        encode_header(self)
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
    pub const fn crypto_epoch(&self) -> CryptoEpoch {
        self.crypto_epoch
    }

    #[must_use]
    pub const fn encryption_domain(&self) -> EncryptionDomain {
        self.encryption_domain
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
    pub const fn content_digest(&self) -> Option<&ContentDigest> {
        self.content_digest.as_ref()
    }

    /// Validates only the segment header's structural metadata.
    ///
    /// This does not recompute or compare the body CRC/hash against segment body
    /// bytes. Future readers with access to the body must verify both before
    /// accepting body contents as tamper-evident.
    pub fn validate(&self) -> Result<()> {
        if self.magic != SEGMENT_MAGIC {
            return Err(invalid_segment("segment magic mismatch"));
        }
        validate_version(self.version, SEGMENT_VERSION_MAX, "segment")?;
        validate_segment_metadata(SegmentValidationInput {
            tenant_id: self.tenant_id,
            min_tx: self.min_tx,
            max_tx: self.max_tx,
            crypto_epoch: self.crypto_epoch,
            encryption_domain: self.encryption_domain,
            body_len: self.body_len,
            body_crc64: self.body_crc64,
            content_digest: self.content_digest.as_ref(),
        })
    }

    pub fn validate_for_domain(&self, expected_domain: EncryptionDomain) -> Result<()> {
        self.validate()?;
        if !self
            .encryption_domain
            .structurally_equal_ct(&expected_domain)
        {
            return Err(invalid_segment("segment encryption domain mismatch"));
        }
        Ok(())
    }
}

impl SegmentFooter {
    pub fn new(input: SegmentFooterInput) -> Result<Self> {
        let footer = Self {
            magic: SEGMENT_FOOTER_MAGIC,
            version: SEGMENT_FOOTER_VERSION_MAX,
            segment_kind: input.segment_kind,
            tenant_id: input.tenant_id,
            min_tx: input.min_tx,
            max_tx: input.max_tx,
            policy_id: input.policy_id,
            encryption_key_id: input.encryption_key_id,
            crypto_epoch: input.crypto_epoch,
            encryption_domain: input.encryption_domain,
            body_len: input.body_len,
            body_crc64: input.body_crc64,
            content_digest: Some(input.content_digest),
        };
        footer.validate()?;
        Ok(footer)
    }

    pub fn from_header(header: &SegmentHeader) -> Result<Self> {
        let content_digest = header
            .content_digest
            .clone()
            .ok_or_else(|| invalid_segment("content digest missing"))?;
        Self::new(SegmentFooterInput {
            segment_kind: header.segment_kind,
            tenant_id: header.tenant_id,
            min_tx: header.min_tx,
            max_tx: header.max_tx,
            policy_id: header.policy_id,
            encryption_key_id: header.encryption_key_id,
            crypto_epoch: header.crypto_epoch,
            encryption_domain: header.encryption_domain,
            body_len: header.body_len,
            body_crc64: header.body_crc64,
            content_digest,
        })
    }

    pub fn parse(bytes: &[u8]) -> Result<Self> {
        parse_footer(bytes)
    }

    #[must_use]
    pub fn encode(&self) -> [u8; SEGMENT_FOOTER_BYTES] {
        encode_footer(self)
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
    pub const fn crypto_epoch(&self) -> CryptoEpoch {
        self.crypto_epoch
    }

    #[must_use]
    pub const fn encryption_domain(&self) -> EncryptionDomain {
        self.encryption_domain
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
    pub const fn content_digest(&self) -> Option<&ContentDigest> {
        self.content_digest.as_ref()
    }

    /// Validates only the segment footer's structural metadata.
    ///
    /// Call [`Self::validate_against_header`] after reading both ends of a
    /// segment so the immutable format is bound by matching metadata.
    pub fn validate(&self) -> Result<()> {
        if self.magic != SEGMENT_FOOTER_MAGIC {
            return Err(invalid_segment("segment footer magic mismatch"));
        }
        validate_version(self.version, SEGMENT_FOOTER_VERSION_MAX, "segment footer")?;
        validate_segment_metadata(SegmentValidationInput {
            tenant_id: self.tenant_id,
            min_tx: self.min_tx,
            max_tx: self.max_tx,
            crypto_epoch: self.crypto_epoch,
            encryption_domain: self.encryption_domain,
            body_len: self.body_len,
            body_crc64: self.body_crc64,
            content_digest: self.content_digest.as_ref(),
        })
    }

    pub fn validate_against_header(&self, header: &SegmentHeader) -> Result<()> {
        self.validate()?;
        header.validate()?;
        if self.tenant_id.get() != header.tenant_id.get()
            || self.segment_kind != header.segment_kind
            || self.min_tx.get() != header.min_tx.get()
            || self.max_tx.get() != header.max_tx.get()
            || self.policy_id.get() != header.policy_id.get()
            || self.encryption_key_id.get() != header.encryption_key_id.get()
            || self.crypto_epoch.get() != header.crypto_epoch.get()
            || !self
                .encryption_domain
                .structurally_equal_ct(&header.encryption_domain)
            || self.body_len != header.body_len
            || self.body_crc64 != header.body_crc64
            || !content_digest_matches(self.content_digest.as_ref(), header.content_digest.as_ref())
        {
            return Err(invalid_segment("segment footer/header metadata mismatch"));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests;
