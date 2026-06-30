use alloc::string::String;

use skrifheim_core::{Result, SkrifheimError, TenantId, TxId};
use skrifheim_crypto::{ContentDigest, CryptoEpoch, EncryptionDomain, EncryptionDomainPurpose};

use super::{BodyChecksum, SEGMENT_BODY_MAX_BYTES};

pub(super) struct SegmentValidationInput<'a> {
    pub tenant_id: TenantId,
    pub min_tx: TxId,
    pub max_tx: TxId,
    pub crypto_epoch: CryptoEpoch,
    pub encryption_domain: EncryptionDomain,
    pub body_len: u64,
    pub body_crc64: BodyChecksum,
    pub content_digest: Option<&'a ContentDigest>,
}

pub(super) fn validate_version(version: u16, version_max: u16, label: &'static str) -> Result<()> {
    if version == 0 {
        return Err(invalid_segment_with_label(
            label,
            "version must be non-zero",
        ));
    }
    if version > version_max {
        return Err(invalid_segment_with_label(
            label,
            "version is newer than this parser",
        ));
    }
    Ok(())
}

pub(super) fn validate_segment_metadata(input: SegmentValidationInput<'_>) -> Result<()> {
    if input.min_tx.get() > input.max_tx.get() {
        return Err(invalid_segment(
            "min transaction is greater than max transaction",
        ));
    }
    if input.crypto_epoch.get() == 0 {
        return Err(invalid_segment("segment crypto epoch must be non-zero"));
    }
    validate_segment_domain(input.tenant_id, input.encryption_domain)?;
    validate_body_len(input.body_len)?;
    validate_body_crc(input.body_crc64)?;
    validate_content_digest(input.content_digest)
}

pub(super) fn content_digest_matches(
    left: Option<&ContentDigest>,
    right: Option<&ContentDigest>,
) -> bool {
    match (left, right) {
        (Some(left), Some(right)) => left.structurally_equal_ct(right),
        (None, None) => true,
        _ => false,
    }
}

pub(super) fn invalid_segment(reason: &'static str) -> SkrifheimError {
    SkrifheimError::InvalidStorageHeader(String::from(reason))
}

fn validate_segment_domain(tenant_id: TenantId, encryption_domain: EncryptionDomain) -> Result<()> {
    if encryption_domain.purpose() != EncryptionDomainPurpose::Segment {
        return Err(invalid_segment(
            "segment encryption domain must have segment purpose",
        ));
    }
    if encryption_domain.tenant_id().get() != tenant_id.get() {
        return Err(invalid_segment("segment tenant/domain mismatch"));
    }
    if encryption_domain.classification_level().is_none() {
        return Err(invalid_segment(
            "segment encryption domain classification missing",
        ));
    }
    if encryption_domain.compartment_id().is_none() {
        return Err(invalid_segment(
            "segment encryption domain compartment missing",
        ));
    }
    if encryption_domain.segment_id().is_none() {
        return Err(invalid_segment("segment encryption domain id missing"));
    }
    Ok(())
}

fn validate_body_len(body_len: u64) -> Result<()> {
    if body_len == 0 {
        return Err(invalid_segment("segment body must not be empty"));
    }
    if body_len > SEGMENT_BODY_MAX_BYTES {
        return Err(invalid_segment("segment body exceeds maximum size"));
    }
    Ok(())
}

fn validate_body_crc(body_crc64: BodyChecksum) -> Result<()> {
    match body_crc64 {
        BodyChecksum::Missing => Err(invalid_segment("body CRC missing")),
        BodyChecksum::Present(0) => Err(invalid_segment("body CRC must not be zero")),
        BodyChecksum::Present(_) => Ok(()),
    }
}

fn validate_content_digest(content_digest: Option<&ContentDigest>) -> Result<()> {
    match content_digest {
        None => Err(invalid_segment("content digest missing")),
        Some(digest) if digest.digest_bytes().iter().all(|byte| *byte == 0) => {
            Err(invalid_segment("content digest must not be all-zero"))
        }
        Some(_) => Ok(()),
    }
}

fn invalid_segment_with_label(label: &'static str, reason: &'static str) -> SkrifheimError {
    let mut message = String::from(label);
    message.push(' ');
    message.push_str(reason);
    SkrifheimError::InvalidStorageHeader(message)
}
