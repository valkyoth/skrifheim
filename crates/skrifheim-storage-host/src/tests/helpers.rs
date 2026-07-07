use std::{
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

use skrifheim_core::{Result as SkrifheimResult, SkrifheimError, TenantId, TxId, WorldId};
use skrifheim_crypto::{
    CompartmentKeyId, ContentDigest, CryptoEpoch, DigestPolicy, EncryptionDomain, KeyId,
    RegionKeyId, SegmentKeyId,
};
use skrifheim_storage::{
    BodyChecksum, SegmentHeader, SegmentHeaderInput, SegmentKind, WalFrameHeader,
    WalFrameHeaderInput, WalRecordKind, wal_body_crc64,
};

use crate::{SegmentContentVerifier, SegmentFileError, WalFileError};

pub(super) type WalResult<T> = core::result::Result<T, WalFileError>;
pub(super) type SegmentResult<T> = core::result::Result<T, SegmentFileError>;

pub(super) fn id<T>(id: Option<T>) -> SkrifheimResult<T> {
    id.ok_or(SkrifheimError::InvalidIdentifier)
}

pub(super) fn tenant() -> SkrifheimResult<TenantId> {
    id(TenantId::from_u128(1))
}

pub(super) fn wal_domain() -> SkrifheimResult<EncryptionDomain> {
    Ok(EncryptionDomain::wal(
        tenant()?,
        Some(id(RegionKeyId::from_u128(2))?),
        Some(id(WorldId::from_u128(3))?),
    ))
}

pub(super) fn segment_domain() -> SkrifheimResult<EncryptionDomain> {
    Ok(EncryptionDomain::segment(
        tenant()?,
        Some(id(RegionKeyId::from_u128(8))?),
        skrifheim_core::Classification::Restricted,
        id(CompartmentKeyId::from_u128(9))?,
        id(SegmentKeyId::from_u128(10))?,
    ))
}

pub(super) fn header(tx: u128, body: &[u8]) -> SkrifheimResult<WalFrameHeader> {
    WalFrameHeader::new(WalFrameHeaderInput {
        record_kind: WalRecordKind::FactBatch,
        tenant_id: tenant()?,
        tx_id: id(TxId::from_u128(tx))?,
        encryption_key_id: id(KeyId::from_u128(4))?,
        crypto_epoch: CryptoEpoch::new(5),
        encryption_domain: wal_domain()?,
        encrypted_body_len: body.len() as u64,
        body_crc64: BodyChecksum::Present(wal_body_crc64(body)),
    })
}

pub(super) fn segment_header(body: &[u8]) -> SkrifheimResult<SegmentHeader> {
    SegmentHeader::new(SegmentHeaderInput {
        segment_kind: SegmentKind::Fact,
        tenant_id: tenant()?,
        min_tx: id(TxId::from_u128(20))?,
        max_tx: id(TxId::from_u128(21))?,
        policy_id: id(skrifheim_core::PolicyId::from_u128(22))?,
        encryption_key_id: id(KeyId::from_u128(23))?,
        crypto_epoch: CryptoEpoch::new(24),
        encryption_domain: segment_domain()?,
        body_len: body.len() as u64,
        body_crc64: BodyChecksum::Present(wal_body_crc64(body)),
        content_digest: ContentDigest::new(DigestPolicy::HIGH_SECURITY, &[31; 32])?,
    })
}

pub(super) fn temp_path(name: &str) -> WalResult<PathBuf> {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| WalFileError::Io(std::io::Error::other(error)))?
        .as_nanos();
    let path = std::env::temp_dir().join(format!(
        "skrifheim-wal-{name}-{}-{timestamp}.wal",
        std::process::id()
    ));
    Ok(path)
}

pub(super) fn wal_to_segment_error(error: WalFileError) -> SegmentFileError {
    match error {
        WalFileError::Io(error) => SegmentFileError::Io(error),
        WalFileError::InvalidFrame(error) => SegmentFileError::InvalidSegment(error),
        WalFileError::BodyLengthMismatch => SegmentFileError::BodyLengthMismatch,
        WalFileError::PartialFrame => SegmentFileError::PartialSegment,
    }
}

pub(super) struct AcceptDigest;

impl SegmentContentVerifier for AcceptDigest {
    fn verify_content_digest(
        &self,
        header: &SegmentHeader,
        _encrypted_body: &[u8],
    ) -> SkrifheimResult<()> {
        header
            .content_digest()
            .ok_or(SkrifheimError::InvalidDigest)?
            .require_policy(DigestPolicy::HIGH_SECURITY)
    }
}

pub(super) struct RejectDigest;

impl SegmentContentVerifier for RejectDigest {
    fn verify_content_digest(
        &self,
        _header: &SegmentHeader,
        _encrypted_body: &[u8],
    ) -> SkrifheimResult<()> {
        Err(SkrifheimError::InvalidDigest)
    }
}
