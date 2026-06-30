use skrifheim_core::{Classification, PolicyId, Result, TenantId, TxId};
use skrifheim_crypto::{
    CompartmentKeyId, ContentDigest, CryptoEpoch, DigestPolicy, DigestStrength, EncryptionDomain,
    KeyId, RegionKeyId, SegmentKeyId,
};

use super::{
    BodyChecksum, SEGMENT_FOOTER_BYTES, SEGMENT_HEADER_BYTES, SegmentFooter, SegmentHeader,
    SegmentKind, validation::invalid_segment,
};

const VERSION_OFFSET: usize = 8;
const KIND_OFFSET: usize = 10;
const RESERVED_A_OFFSET: usize = 11;
const TENANT_OFFSET: usize = 16;
const MIN_TX_OFFSET: usize = 32;
const MAX_TX_OFFSET: usize = 48;
const POLICY_OFFSET: usize = 64;
const KEY_OFFSET: usize = 80;
const EPOCH_OFFSET: usize = 96;
const REGION_OFFSET: usize = 104;
const CLASSIFICATION_OFFSET: usize = 120;
const RESERVED_B_OFFSET: usize = 121;
const COMPARTMENT_OFFSET: usize = 128;
const SEGMENT_ID_OFFSET: usize = 144;
const BODY_LEN_OFFSET: usize = 160;
const BODY_CRC_OFFSET: usize = 168;
const DIGEST_STRENGTH_OFFSET: usize = 176;
const RESERVED_C_OFFSET: usize = 177;
const DIGEST_OFFSET: usize = 192;

pub(super) fn parse_header(bytes: &[u8]) -> Result<SegmentHeader> {
    if bytes.len() != SEGMENT_HEADER_BYTES {
        return Err(invalid_segment("segment header length mismatch"));
    }
    validate_reserved(bytes)?;
    let mut magic = [0_u8; 8];
    magic.copy_from_slice(&bytes[..8]);
    let version = u16::from_le_bytes(read_array(bytes, VERSION_OFFSET)?);
    let segment_kind = segment_kind_from_code(bytes[KIND_OFFSET])?;
    let metadata = parse_common_metadata(bytes)?;
    let header = SegmentHeader {
        magic,
        version,
        segment_kind,
        tenant_id: metadata.tenant_id,
        min_tx: metadata.min_tx,
        max_tx: metadata.max_tx,
        policy_id: metadata.policy_id,
        encryption_key_id: metadata.encryption_key_id,
        crypto_epoch: metadata.crypto_epoch,
        encryption_domain: metadata.encryption_domain,
        body_len: metadata.body_len,
        body_crc64: metadata.body_crc64,
        content_digest: Some(metadata.content_digest),
    };
    header.validate()?;
    Ok(header)
}

#[must_use]
pub(super) fn encode_header(header: &SegmentHeader) -> [u8; SEGMENT_HEADER_BYTES] {
    let mut bytes = [0_u8; SEGMENT_HEADER_BYTES];
    bytes[..8].copy_from_slice(&header.magic);
    bytes[VERSION_OFFSET..KIND_OFFSET].copy_from_slice(&header.version.to_le_bytes());
    bytes[KIND_OFFSET] = segment_kind_code(header.segment_kind);
    write_common_metadata(
        &mut bytes,
        CommonSegmentMetadataRef {
            tenant_id: header.tenant_id,
            min_tx: header.min_tx,
            max_tx: header.max_tx,
            policy_id: header.policy_id,
            encryption_key_id: header.encryption_key_id,
            crypto_epoch: header.crypto_epoch,
            encryption_domain: header.encryption_domain,
            body_len: header.body_len,
            body_crc64: header.body_crc64,
            content_digest: header.content_digest.as_ref(),
        },
    );
    bytes
}

pub(super) fn parse_footer(bytes: &[u8]) -> Result<SegmentFooter> {
    if bytes.len() != SEGMENT_FOOTER_BYTES {
        return Err(invalid_segment("segment footer length mismatch"));
    }
    validate_reserved(bytes)?;
    if bytes[KIND_OFFSET] != 0 {
        return Err(invalid_segment("segment footer reserved kind is non-zero"));
    }
    let mut magic = [0_u8; 8];
    magic.copy_from_slice(&bytes[..8]);
    let version = u16::from_le_bytes(read_array(bytes, VERSION_OFFSET)?);
    let metadata = parse_common_metadata(bytes)?;
    let footer = SegmentFooter {
        magic,
        version,
        tenant_id: metadata.tenant_id,
        min_tx: metadata.min_tx,
        max_tx: metadata.max_tx,
        policy_id: metadata.policy_id,
        encryption_key_id: metadata.encryption_key_id,
        crypto_epoch: metadata.crypto_epoch,
        encryption_domain: metadata.encryption_domain,
        body_len: metadata.body_len,
        body_crc64: metadata.body_crc64,
        content_digest: Some(metadata.content_digest),
    };
    footer.validate()?;
    Ok(footer)
}

#[must_use]
pub(super) fn encode_footer(footer: &SegmentFooter) -> [u8; SEGMENT_FOOTER_BYTES] {
    let mut bytes = [0_u8; SEGMENT_FOOTER_BYTES];
    bytes[..8].copy_from_slice(&footer.magic);
    bytes[VERSION_OFFSET..KIND_OFFSET].copy_from_slice(&footer.version.to_le_bytes());
    write_common_metadata(
        &mut bytes,
        CommonSegmentMetadataRef {
            tenant_id: footer.tenant_id,
            min_tx: footer.min_tx,
            max_tx: footer.max_tx,
            policy_id: footer.policy_id,
            encryption_key_id: footer.encryption_key_id,
            crypto_epoch: footer.crypto_epoch,
            encryption_domain: footer.encryption_domain,
            body_len: footer.body_len,
            body_crc64: footer.body_crc64,
            content_digest: footer.content_digest.as_ref(),
        },
    );
    bytes
}

struct CommonSegmentMetadata {
    tenant_id: TenantId,
    min_tx: TxId,
    max_tx: TxId,
    policy_id: PolicyId,
    encryption_key_id: KeyId,
    crypto_epoch: CryptoEpoch,
    encryption_domain: EncryptionDomain,
    body_len: u64,
    body_crc64: BodyChecksum,
    content_digest: ContentDigest,
}

struct CommonSegmentMetadataRef<'a> {
    tenant_id: TenantId,
    min_tx: TxId,
    max_tx: TxId,
    policy_id: PolicyId,
    encryption_key_id: KeyId,
    crypto_epoch: CryptoEpoch,
    encryption_domain: EncryptionDomain,
    body_len: u64,
    body_crc64: BodyChecksum,
    content_digest: Option<&'a ContentDigest>,
}

fn write_common_metadata(bytes: &mut [u8], metadata: CommonSegmentMetadataRef<'_>) {
    bytes[TENANT_OFFSET..MIN_TX_OFFSET].copy_from_slice(&metadata.tenant_id.get().to_le_bytes());
    bytes[MIN_TX_OFFSET..MAX_TX_OFFSET].copy_from_slice(&metadata.min_tx.get().to_le_bytes());
    bytes[MAX_TX_OFFSET..POLICY_OFFSET].copy_from_slice(&metadata.max_tx.get().to_le_bytes());
    bytes[POLICY_OFFSET..KEY_OFFSET].copy_from_slice(&metadata.policy_id.get().to_le_bytes());
    bytes[KEY_OFFSET..EPOCH_OFFSET]
        .copy_from_slice(&metadata.encryption_key_id.get().to_le_bytes());
    bytes[EPOCH_OFFSET..REGION_OFFSET].copy_from_slice(&metadata.crypto_epoch.get().to_le_bytes());
    let region_id = metadata
        .encryption_domain
        .region_id()
        .map_or(0, RegionKeyId::get);
    bytes[REGION_OFFSET..CLASSIFICATION_OFFSET].copy_from_slice(&region_id.to_le_bytes());
    bytes[CLASSIFICATION_OFFSET] = metadata
        .encryption_domain
        .classification_level()
        .map_or(0, classification_tag);
    let compartment_id = metadata
        .encryption_domain
        .compartment_id()
        .map_or(0, CompartmentKeyId::get);
    bytes[COMPARTMENT_OFFSET..SEGMENT_ID_OFFSET].copy_from_slice(&compartment_id.to_le_bytes());
    let segment_id = metadata
        .encryption_domain
        .segment_id()
        .map_or(0, SegmentKeyId::get);
    bytes[SEGMENT_ID_OFFSET..BODY_LEN_OFFSET].copy_from_slice(&segment_id.to_le_bytes());
    bytes[BODY_LEN_OFFSET..BODY_CRC_OFFSET].copy_from_slice(&metadata.body_len.to_le_bytes());
    bytes[BODY_CRC_OFFSET..DIGEST_STRENGTH_OFFSET]
        .copy_from_slice(&require_raw_crc(metadata.body_crc64).to_le_bytes());
    if let Some(content_digest) = metadata.content_digest {
        bytes[DIGEST_STRENGTH_OFFSET] = digest_strength_tag(content_digest.strength());
        let digest = content_digest.digest_bytes();
        bytes[DIGEST_OFFSET..DIGEST_OFFSET + digest.len()].copy_from_slice(digest);
    }
}

fn parse_common_metadata(bytes: &[u8]) -> Result<CommonSegmentMetadata> {
    let tenant_id = TenantId::from_u128(read_u128(bytes, TENANT_OFFSET)?)
        .ok_or_else(|| invalid_segment("segment tenant identifier must be non-zero"))?;
    let min_tx = TxId::from_u128(read_u128(bytes, MIN_TX_OFFSET)?)
        .ok_or_else(|| invalid_segment("segment min transaction must be non-zero"))?;
    let max_tx = TxId::from_u128(read_u128(bytes, MAX_TX_OFFSET)?)
        .ok_or_else(|| invalid_segment("segment max transaction must be non-zero"))?;
    let policy_id = PolicyId::from_u128(read_u128(bytes, POLICY_OFFSET)?)
        .ok_or_else(|| invalid_segment("segment policy identifier must be non-zero"))?;
    let encryption_key_id = KeyId::from_u128(read_u128(bytes, KEY_OFFSET)?)
        .ok_or_else(|| invalid_segment("segment key identifier must be non-zero"))?;
    let crypto_epoch = CryptoEpoch::new(u64::from_le_bytes(read_array(bytes, EPOCH_OFFSET)?));
    let region_id = optional_region(read_u128(bytes, REGION_OFFSET)?)?;
    let classification = classification_from_tag(bytes[CLASSIFICATION_OFFSET])?;
    let compartment_id = CompartmentKeyId::from_u128(read_u128(bytes, COMPARTMENT_OFFSET)?)
        .ok_or_else(|| invalid_segment("segment compartment identifier must be non-zero"))?;
    let segment_id = SegmentKeyId::from_u128(read_u128(bytes, SEGMENT_ID_OFFSET)?)
        .ok_or_else(|| invalid_segment("segment domain identifier must be non-zero"))?;
    let body_len = u64::from_le_bytes(read_array(bytes, BODY_LEN_OFFSET)?);
    let body_crc64 = BodyChecksum::Present(u64::from_le_bytes(read_array(bytes, BODY_CRC_OFFSET)?));
    let strength = digest_strength_from_tag(bytes[DIGEST_STRENGTH_OFFSET])?;
    let digest_len = strength.output_bytes();
    let content_digest = ContentDigest::new(
        DigestPolicy::new(strength),
        &bytes[DIGEST_OFFSET..DIGEST_OFFSET + digest_len],
    )?;
    let encryption_domain = EncryptionDomain::segment(
        tenant_id,
        region_id,
        classification,
        compartment_id,
        segment_id,
    );
    Ok(CommonSegmentMetadata {
        tenant_id,
        min_tx,
        max_tx,
        policy_id,
        encryption_key_id,
        crypto_epoch,
        encryption_domain,
        body_len,
        body_crc64,
        content_digest,
    })
}

fn validate_reserved(bytes: &[u8]) -> Result<()> {
    if bytes[RESERVED_A_OFFSET..TENANT_OFFSET]
        .iter()
        .any(|byte| *byte != 0)
        || bytes[RESERVED_B_OFFSET..COMPARTMENT_OFFSET]
            .iter()
            .any(|byte| *byte != 0)
        || bytes[RESERVED_C_OFFSET..DIGEST_OFFSET]
            .iter()
            .any(|byte| *byte != 0)
    {
        return Err(invalid_segment("segment reserved bytes are non-zero"));
    }
    Ok(())
}

fn read_array<const N: usize>(bytes: &[u8], offset: usize) -> Result<[u8; N]> {
    bytes
        .get(offset..offset + N)
        .and_then(|value| value.try_into().ok())
        .ok_or_else(|| invalid_segment("segment metadata field is truncated"))
}

fn read_u128(bytes: &[u8], offset: usize) -> Result<u128> {
    Ok(u128::from_le_bytes(read_array(bytes, offset)?))
}

fn optional_region(value: u128) -> Result<Option<RegionKeyId>> {
    if value == 0 {
        Ok(None)
    } else {
        RegionKeyId::from_u128(value)
            .map(Some)
            .ok_or_else(|| invalid_segment("segment region identifier is invalid"))
    }
}

const fn segment_kind_code(kind: SegmentKind) -> u8 {
    match kind {
        SegmentKind::Wal => 1,
        SegmentKind::Fact => 2,
        SegmentKind::Projection => 3,
        SegmentKind::Blob => 4,
    }
}

fn segment_kind_from_code(code: u8) -> Result<SegmentKind> {
    match code {
        1 => Ok(SegmentKind::Wal),
        2 => Ok(SegmentKind::Fact),
        3 => Ok(SegmentKind::Projection),
        4 => Ok(SegmentKind::Blob),
        _ => Err(invalid_segment("unknown segment kind")),
    }
}

fn classification_tag(classification: Classification) -> u8 {
    match classification {
        Classification::Public => 1,
        Classification::Internal => 2,
        Classification::Restricted => 3,
        Classification::Secret => 4,
        Classification::TopSecret => 5,
    }
}

fn classification_from_tag(tag: u8) -> Result<Classification> {
    match tag {
        1 => Ok(Classification::Public),
        2 => Ok(Classification::Internal),
        3 => Ok(Classification::Restricted),
        4 => Ok(Classification::Secret),
        5 => Ok(Classification::TopSecret),
        _ => Err(invalid_segment("segment classification tag is invalid")),
    }
}

fn digest_strength_tag(strength: DigestStrength) -> u8 {
    match strength {
        DigestStrength::Sha3_256 => 1,
        DigestStrength::Sha3_384 => 2,
        DigestStrength::Sha3_512 => 3,
        DigestStrength::Shake256_256 => 4,
        DigestStrength::Shake256_512 => 5,
    }
}

fn digest_strength_from_tag(tag: u8) -> Result<DigestStrength> {
    match tag {
        1 => Ok(DigestStrength::Sha3_256),
        2 => Ok(DigestStrength::Sha3_384),
        3 => Ok(DigestStrength::Sha3_512),
        4 => Ok(DigestStrength::Shake256_256),
        5 => Ok(DigestStrength::Shake256_512),
        _ => Err(invalid_segment("segment digest strength tag is invalid")),
    }
}

const fn require_raw_crc(body_crc64: BodyChecksum) -> u64 {
    match body_crc64 {
        BodyChecksum::Present(value) => value,
        BodyChecksum::Missing => 0,
    }
}
