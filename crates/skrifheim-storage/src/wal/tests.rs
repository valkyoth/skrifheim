use alloc::format;

use skrifheim_core::{Result, SkrifheimError, TenantId, TxId, WorldId};
use skrifheim_crypto::{CryptoEpoch, EncryptionDomain, KeyId, RegionKeyId};

use super::*;

fn id<T>(id: Option<T>) -> Result<T> {
    id.ok_or(SkrifheimError::InvalidIdentifier)
}

fn tenant() -> Result<TenantId> {
    id(TenantId::from_u128(1))
}

fn header_input() -> Result<WalFrameHeaderInput> {
    Ok(WalFrameHeaderInput {
        record_kind: WalRecordKind::FactBatch,
        tenant_id: tenant()?,
        tx_id: id(TxId::from_u128(2))?,
        encryption_key_id: id(KeyId::from_u128(3))?,
        crypto_epoch: CryptoEpoch::new(4),
        encryption_domain: EncryptionDomain::wal(
            tenant()?,
            Some(id(RegionKeyId::from_u128(5))?),
            Some(id(WorldId::from_u128(6))?),
        ),
        encrypted_body_len: 7,
        body_crc64: BodyChecksum::Present(8),
    })
}

fn header() -> Result<WalFrameHeader> {
    WalFrameHeader::new(header_input()?)
}

#[test]
fn wal_frame_header_accepts_valid_metadata() -> Result<()> {
    let header = header()?;

    assert_eq!(header.magic(), WAL_FRAME_MAGIC);
    assert_eq!(header.version(), WAL_FRAME_VERSION_MAX);
    assert_eq!(header.record_kind(), WalRecordKind::FactBatch);
    assert_eq!(header.tenant_id().get(), 1);
    assert_eq!(header.tx_id().get(), 2);
    assert_eq!(header.encryption_key_id().get(), 3);
    assert_eq!(header.crypto_epoch().get(), 4);
    assert_eq!(header.encrypted_body_len(), 7);
    assert_eq!(header.body_crc64(), BodyChecksum::Present(8));
    Ok(())
}

#[test]
fn wal_frame_header_round_trips_fixed_bytes() -> Result<()> {
    let header = header()?;
    let parsed = WalFrameHeader::parse(&header.encode())?;

    assert_eq!(parsed.record_kind(), header.record_kind());
    assert_eq!(parsed.tenant_id().get(), header.tenant_id().get());
    assert_eq!(parsed.tx_id().get(), header.tx_id().get());
    assert_eq!(parsed.crypto_epoch().get(), header.crypto_epoch().get());
    assert_eq!(parsed.encrypted_body_len(), header.encrypted_body_len());
    assert_eq!(parsed.body_crc64(), header.body_crc64());
    assert!(
        parsed
            .encryption_domain()
            .structurally_equal(&header.encryption_domain())
    );
    Ok(())
}

#[test]
fn wal_frame_parser_rejects_malformed_headers() -> Result<()> {
    let mut bytes = header()?.encode();
    bytes[0] = b'X';
    assert!(matches!(
        WalFrameHeader::parse(&bytes),
        Err(SkrifheimError::InvalidWalFrame(_))
    ));

    let mut bytes = header()?.encode();
    bytes[KIND_OFFSET] = 99;
    assert!(matches!(
        WalFrameHeader::parse(&bytes),
        Err(SkrifheimError::InvalidWalFrame(_))
    ));

    let mut bytes = header()?.encode();
    bytes[RESERVED_OFFSET] = 1;
    assert!(matches!(
        WalFrameHeader::parse(&bytes),
        Err(SkrifheimError::InvalidWalFrame(_))
    ));

    assert!(matches!(
        WalFrameHeader::parse(&header()?.encode()[..WAL_FRAME_HEADER_BYTES - 1]),
        Err(SkrifheimError::InvalidWalFrame(_))
    ));
    Ok(())
}

#[test]
fn wal_frame_validation_rejects_unencrypted_or_unbounded_body_metadata() -> Result<()> {
    let mut input = header_input()?;
    input.encrypted_body_len = 0;
    assert!(matches!(
        WalFrameHeader::new(input),
        Err(SkrifheimError::InvalidWalFrame(_))
    ));

    let mut input = header_input()?;
    input.encrypted_body_len = WAL_FRAME_BODY_MAX_BYTES + 1;
    assert!(matches!(
        WalFrameHeader::new(input),
        Err(SkrifheimError::InvalidWalFrame(_))
    ));

    let mut input = header_input()?;
    input.body_crc64 = BodyChecksum::Missing;
    assert!(matches!(
        WalFrameHeader::new(input),
        Err(SkrifheimError::InvalidWalFrame(_))
    ));
    Ok(())
}

#[test]
fn wal_frame_validation_rejects_non_wal_or_cross_tenant_domain() -> Result<()> {
    let mut input = header_input()?;
    input.encryption_domain = EncryptionDomain::tenant(tenant()?);
    assert!(matches!(
        WalFrameHeader::new(input),
        Err(SkrifheimError::InvalidWalFrame(_))
    ));

    let mut input = header_input()?;
    input.encryption_domain = EncryptionDomain::wal(id(TenantId::from_u128(9))?, None, None);
    assert!(matches!(
        WalFrameHeader::new(input),
        Err(SkrifheimError::InvalidWalFrame(_))
    ));
    Ok(())
}

#[test]
fn wal_frame_debug_redacts_sensitive_metadata() -> Result<()> {
    let input = header_input()?;
    let input_debug = format!("{input:?}");
    let header_debug = format!("{:?}", WalFrameHeader::new(input)?);

    assert!(input_debug.contains("encryption_key_id: \"<redacted>\""));
    assert!(input_debug.contains("encryption_domain: \"<redacted>\""));
    assert!(header_debug.contains("crypto_epoch: \"<redacted>\""));
    assert!(header_debug.contains("body_crc64: \"<redacted>\""));
    assert!(!input_debug.contains("KeyId"));
    assert!(!header_debug.contains("RegionKeyId"));
    assert!(!header_debug.contains("WorldId"));
    Ok(())
}
