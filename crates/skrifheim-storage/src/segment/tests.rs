use super::*;
use skrifheim_core::{Classification, SkrifheimError};
use skrifheim_crypto::{
    CompartmentKeyId, DigestPolicy, EncryptionDomainPurpose, RegionKeyId, SegmentKeyId,
};

fn id<T>(id: Option<T>) -> Result<T> {
    id.ok_or(SkrifheimError::InvalidIdentifier)
}

fn tenant_id() -> Result<TenantId> {
    id(TenantId::from_u128(1))
}

fn encryption_domain() -> Result<EncryptionDomain> {
    Ok(EncryptionDomain::segment(
        tenant_id()?,
        Some(id(RegionKeyId::from_u128(8))?),
        Classification::Restricted,
        id(CompartmentKeyId::from_u128(9))?,
        id(SegmentKeyId::from_u128(10))?,
    ))
}

fn header() -> Result<SegmentHeader> {
    SegmentHeader::new(header_input()?)
}

fn header_input() -> Result<SegmentHeaderInput> {
    Ok(SegmentHeaderInput {
        segment_kind: SegmentKind::Fact,
        tenant_id: tenant_id()?,
        min_tx: id(TxId::from_u128(1))?,
        max_tx: id(TxId::from_u128(2))?,
        policy_id: id(PolicyId::from_u128(3))?,
        encryption_key_id: id(KeyId::from_u128(4))?,
        crypto_epoch: CryptoEpoch::new(5),
        encryption_domain: encryption_domain()?,
        body_len: 6,
        body_crc64: BodyChecksum::Present(7),
        content_digest: ContentDigest::new(DigestPolicy::HIGH_SECURITY, &[11; 32])?,
    })
}

fn footer() -> Result<SegmentFooter> {
    SegmentFooter::from_header(&header()?)
}

#[test]
fn valid_header_passes() -> Result<()> {
    assert_eq!(header()?.validate(), Ok(()));
    Ok(())
}

#[test]
fn valid_footer_passes_and_matches_header() -> Result<()> {
    let header = header()?;
    let footer = SegmentFooter::from_header(&header)?;

    assert_eq!(footer.validate(), Ok(()));
    assert_eq!(footer.validate_against_header(&header), Ok(()));
    assert_eq!(footer.magic(), SEGMENT_FOOTER_MAGIC);
    assert_eq!(footer.version(), SEGMENT_FOOTER_VERSION_MAX);
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
    assert_eq!(header.encryption_key_id().get(), 4);
    assert_eq!(header.crypto_epoch().get(), 5);
    assert_eq!(
        header.encryption_domain().purpose(),
        EncryptionDomainPurpose::Segment
    );
    assert_eq!(header.body_len(), 6);
    assert_eq!(header.body_crc64(), BodyChecksum::Present(7));
    assert!(
        header
            .content_digest()
            .is_some_and(|digest| digest.digest_bytes() == [11; 32])
    );
    Ok(())
}

#[test]
fn footer_carries_matching_metadata() -> Result<()> {
    let footer = footer()?;

    assert_eq!(footer.tenant_id().get(), 1);
    assert_eq!(footer.min_tx().get(), 1);
    assert_eq!(footer.max_tx().get(), 2);
    assert_eq!(footer.policy_id().get(), 3);
    assert_eq!(footer.encryption_key_id().get(), 4);
    assert_eq!(footer.crypto_epoch().get(), 5);
    assert_eq!(
        footer.encryption_domain().purpose(),
        EncryptionDomainPurpose::Segment
    );
    assert_eq!(footer.body_len(), 6);
    assert_eq!(footer.body_crc64(), BodyChecksum::Present(7));
    assert!(
        footer
            .content_digest()
            .is_some_and(|digest| digest.digest_bytes() == [11; 32])
    );
    Ok(())
}

#[test]
fn header_and_footer_round_trip_fixed_bytes() -> Result<()> {
    let header = header()?;
    let footer = SegmentFooter::from_header(&header)?;

    let header_bytes = header.encode();
    let footer_bytes = footer.encode();
    let parsed_header = SegmentHeader::parse(&header_bytes)?;
    let parsed_footer = SegmentFooter::parse(&footer_bytes)?;

    assert_eq!(header_bytes.len(), SEGMENT_HEADER_BYTES);
    assert_eq!(footer_bytes.len(), SEGMENT_FOOTER_BYTES);
    assert_eq!(parsed_header.segment_kind(), SegmentKind::Fact);
    assert_eq!(parsed_header.tenant_id().get(), header.tenant_id().get());
    assert_eq!(parsed_header.min_tx().get(), header.min_tx().get());
    assert_eq!(parsed_header.max_tx().get(), header.max_tx().get());
    assert_eq!(
        parsed_header
            .content_digest()
            .ok_or(SkrifheimError::InvalidDigest)?
            .digest_bytes(),
        header
            .content_digest()
            .ok_or(SkrifheimError::InvalidDigest)?
            .digest_bytes()
    );
    assert_eq!(
        parsed_footer.validate_against_header(&parsed_header),
        Ok(())
    );
    Ok(())
}

#[test]
fn header_parse_for_domain_rejects_unexpected_domain() -> Result<()> {
    let header = header()?;
    let other_domain = EncryptionDomain::segment(
        tenant_id()?,
        Some(id(RegionKeyId::from_u128(8))?),
        Classification::Restricted,
        id(CompartmentKeyId::from_u128(9))?,
        id(SegmentKeyId::from_u128(11))?,
    );

    assert!(matches!(
        SegmentHeader::parse_for_domain(&header.encode(), other_domain),
        Err(SkrifheimError::InvalidStorageHeader(_))
    ));
    Ok(())
}

#[test]
fn parsers_reject_malformed_segment_metadata() -> Result<()> {
    assert!(matches!(
        SegmentHeader::parse(&header()?.encode()[..SEGMENT_HEADER_BYTES - 1]),
        Err(SkrifheimError::InvalidStorageHeader(_))
    ));

    let mut header_bytes = header()?.encode();
    header_bytes[11] = 1;
    assert!(matches!(
        SegmentHeader::parse(&header_bytes),
        Err(SkrifheimError::InvalidStorageHeader(_))
    ));

    let mut footer_bytes = footer()?.encode();
    footer_bytes[10] = 1;
    assert!(matches!(
        SegmentFooter::parse(&footer_bytes),
        Err(SkrifheimError::InvalidStorageHeader(_))
    ));
    Ok(())
}

#[test]
fn decoded_footer_rejects_header_mismatch() -> Result<()> {
    let header = header()?;
    let mut footer_bytes = SegmentFooter::from_header(&header)?.encode();
    footer_bytes[160..168].copy_from_slice(&8_u64.to_le_bytes());
    let footer = SegmentFooter::parse(&footer_bytes)?;

    assert!(matches!(
        footer.validate_against_header(&header),
        Err(SkrifheimError::InvalidStorageHeader(_))
    ));
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
fn header_rejects_zero_crypto_epoch() -> Result<()> {
    let mut input = header_input()?;
    input.crypto_epoch = CryptoEpoch::new(0);

    assert!(matches!(
        SegmentHeader::new(input),
        Err(SkrifheimError::InvalidStorageHeader(_))
    ));
    Ok(())
}

#[test]
fn header_rejects_wrong_domain_purpose() -> Result<()> {
    let mut input = header_input()?;
    input.encryption_domain = EncryptionDomain::wal(tenant_id()?, None, None);

    assert!(matches!(
        SegmentHeader::new(input),
        Err(SkrifheimError::InvalidStorageHeader(_))
    ));
    Ok(())
}

#[test]
fn header_rejects_cross_tenant_domain() -> Result<()> {
    let mut input = header_input()?;
    input.encryption_domain = EncryptionDomain::segment(
        id(TenantId::from_u128(2))?,
        None,
        Classification::Restricted,
        id(CompartmentKeyId::from_u128(9))?,
        id(SegmentKeyId::from_u128(10))?,
    );

    assert!(matches!(
        SegmentHeader::new(input),
        Err(SkrifheimError::InvalidStorageHeader(_))
    ));
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
fn header_rejects_all_zero_content_digest() -> Result<()> {
    let mut input = header_input()?;
    input.content_digest = ContentDigest::new(DigestPolicy::HIGH_SECURITY, &[0; 32])?;

    assert!(matches!(
        SegmentHeader::new(input),
        Err(SkrifheimError::InvalidStorageHeader(_))
    ));
    Ok(())
}

#[test]
fn header_rejects_explicit_zero_body_crc() -> Result<()> {
    let mut input = header_input()?;
    input.body_crc64 = BodyChecksum::Present(0);
    assert!(matches!(
        SegmentHeader::new(input),
        Err(SkrifheimError::InvalidStorageHeader(_))
    ));
    Ok(())
}

#[test]
fn footer_rejects_header_mismatch() -> Result<()> {
    let header = header()?;
    let mut footer_input = SegmentFooterInput {
        tenant_id: header.tenant_id(),
        min_tx: header.min_tx(),
        max_tx: header.max_tx(),
        policy_id: header.policy_id(),
        encryption_key_id: header.encryption_key_id(),
        crypto_epoch: header.crypto_epoch(),
        encryption_domain: header.encryption_domain(),
        body_len: header.body_len(),
        body_crc64: header.body_crc64(),
        content_digest: header
            .content_digest()
            .cloned()
            .ok_or(SkrifheimError::InvalidDigest)?,
    };
    footer_input.body_len += 1;
    let footer = SegmentFooter::new(footer_input)?;

    assert!(matches!(
        footer.validate_against_header(&header),
        Err(SkrifheimError::InvalidStorageHeader(_))
    ));
    Ok(())
}

#[test]
fn debug_redacts_segment_header_and_footer_sensitive_metadata() -> Result<()> {
    let input = header_input()?;
    let input_debug = alloc::format!("{input:?}");
    let header = SegmentHeader::new(input)?;
    let header_debug = alloc::format!("{header:?}");
    let footer = SegmentFooter::from_header(&header)?;
    let footer_debug = alloc::format!("{footer:?}");

    assert!(input_debug.contains("encryption_key_id: \"<redacted>\""));
    assert!(input_debug.contains("crypto_epoch: \"<redacted>\""));
    assert!(input_debug.contains("encryption_domain: \"<redacted>\""));
    assert!(input_debug.contains("content_digest: \"<redacted>\""));
    assert!(header_debug.contains("encryption_key_id: \"<redacted>\""));
    assert!(header_debug.contains("crypto_epoch: \"<redacted>\""));
    assert!(header_debug.contains("encryption_domain: \"<redacted>\""));
    assert!(header_debug.contains("content_digest: \"<redacted>\""));
    assert!(footer_debug.contains("encryption_key_id: \"<redacted>\""));
    assert!(footer_debug.contains("crypto_epoch: \"<redacted>\""));
    assert!(footer_debug.contains("encryption_domain: \"<redacted>\""));
    assert!(footer_debug.contains("content_digest: \"<redacted>\""));
    assert!(!input_debug.contains("KeyId"));
    assert!(!input_debug.contains("CryptoEpoch"));
    assert!(!input_debug.contains("[11, 11"));
    assert!(!header_debug.contains("KeyId"));
    assert!(!header_debug.contains("CryptoEpoch"));
    assert!(!header_debug.contains("[11, 11"));
    assert!(!footer_debug.contains("KeyId"));
    assert!(!footer_debug.contains("CryptoEpoch"));
    assert!(!footer_debug.contains("[11, 11"));
    Ok(())
}
