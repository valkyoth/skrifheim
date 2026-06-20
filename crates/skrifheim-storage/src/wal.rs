use alloc::string::String;
use core::fmt;

use skrifheim_core::{Result, SkrifheimError, TenantId, TxId, WorldId};
use skrifheim_crypto::{
    CryptoEpoch, EncryptionDomain, EncryptionDomainPurpose, KeyId, RegionKeyId,
};

use crate::BodyChecksum;

pub const WAL_FRAME_MAGIC: [u8; 8] = *b"SKRIFWAL";
pub const WAL_FRAME_VERSION_MAX: u16 = 1;
pub const WAL_FRAME_HEADER_BYTES: usize = 120;
pub const WAL_FRAME_BODY_MAX_BYTES: u64 = 64 * 1024 * 1024;

const KIND_OFFSET: usize = 10;
const RESERVED_OFFSET: usize = 11;
const TENANT_OFFSET: usize = 16;
const REGION_OFFSET: usize = 32;
const WORLD_OFFSET: usize = 48;
const TX_OFFSET: usize = 64;
const EPOCH_OFFSET: usize = 80;
const KEY_OFFSET: usize = 88;
const BODY_LEN_OFFSET: usize = 104;
const BODY_CRC_OFFSET: usize = 112;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WalRecordKind {
    TransactionBegin,
    FactBatch,
    TransactionCommit,
    TransactionAbort,
    Checkpoint,
}

impl WalRecordKind {
    const fn code(self) -> u8 {
        match self {
            Self::TransactionBegin => 1,
            Self::FactBatch => 2,
            Self::TransactionCommit => 3,
            Self::TransactionAbort => 4,
            Self::Checkpoint => 5,
        }
    }

    fn from_code(code: u8) -> Result<Self> {
        match code {
            1 => Ok(Self::TransactionBegin),
            2 => Ok(Self::FactBatch),
            3 => Ok(Self::TransactionCommit),
            4 => Ok(Self::TransactionAbort),
            5 => Ok(Self::Checkpoint),
            _ => Err(invalid_frame("unknown WAL record kind")),
        }
    }
}

#[derive(Clone)]
pub struct WalFrameHeader {
    magic: [u8; 8],
    version: u16,
    record_kind: WalRecordKind,
    tenant_id: TenantId,
    tx_id: TxId,
    encryption_key_id: KeyId,
    crypto_epoch: CryptoEpoch,
    encryption_domain: EncryptionDomain,
    encrypted_body_len: u64,
    body_crc64: u64,
}

impl fmt::Debug for WalFrameHeader {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WalFrameHeader")
            .field("magic", &"<redacted>")
            .field("version", &self.version)
            .field("record_kind", &self.record_kind)
            .field("tenant_id", &"<redacted>")
            .field("tx_id", &"<redacted>")
            .field("encryption_key_id", &"<redacted>")
            .field("crypto_epoch", &"<redacted>")
            .field("encryption_domain", &"<redacted>")
            .field("encrypted_body_len", &self.encrypted_body_len)
            .field("body_crc64", &"<redacted>")
            .finish()
    }
}

#[derive(Clone)]
pub struct WalFrameHeaderInput {
    pub record_kind: WalRecordKind,
    pub tenant_id: TenantId,
    pub tx_id: TxId,
    pub encryption_key_id: KeyId,
    pub crypto_epoch: CryptoEpoch,
    pub encryption_domain: EncryptionDomain,
    pub encrypted_body_len: u64,
    pub body_crc64: BodyChecksum,
}

impl fmt::Debug for WalFrameHeaderInput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WalFrameHeaderInput")
            .field("record_kind", &self.record_kind)
            .field("tenant_id", &"<redacted>")
            .field("tx_id", &"<redacted>")
            .field("encryption_key_id", &"<redacted>")
            .field("crypto_epoch", &"<redacted>")
            .field("encryption_domain", &"<redacted>")
            .field("encrypted_body_len", &self.encrypted_body_len)
            .field("body_crc64", &"<redacted>")
            .finish()
    }
}

impl WalFrameHeader {
    pub fn new(input: WalFrameHeaderInput) -> Result<Self> {
        let header = Self {
            magic: WAL_FRAME_MAGIC,
            version: WAL_FRAME_VERSION_MAX,
            record_kind: input.record_kind,
            tenant_id: input.tenant_id,
            tx_id: input.tx_id,
            encryption_key_id: input.encryption_key_id,
            crypto_epoch: input.crypto_epoch,
            encryption_domain: input.encryption_domain,
            encrypted_body_len: input.encrypted_body_len,
            body_crc64: require_body_crc64(input.body_crc64)?,
        };
        header.validate()?;
        Ok(header)
    }

    pub fn parse(bytes: &[u8]) -> Result<Self> {
        let header = Self::parse_unchecked_domain(bytes)?;
        header.validate()?;
        Ok(header)
    }

    pub fn parse_for_domain(bytes: &[u8], expected_domain: EncryptionDomain) -> Result<Self> {
        let header = Self::parse_unchecked_domain(bytes)?;
        header.validate_for_domain(expected_domain)?;
        Ok(header)
    }

    fn parse_unchecked_domain(bytes: &[u8]) -> Result<Self> {
        if bytes.len() != WAL_FRAME_HEADER_BYTES {
            return Err(invalid_frame("WAL frame header length mismatch"));
        }
        let mut magic = [0_u8; 8];
        magic.copy_from_slice(&bytes[..8]);
        let version = u16::from_le_bytes(read_array(bytes, 8)?);
        let record_kind = WalRecordKind::from_code(bytes[KIND_OFFSET])?;
        if bytes[RESERVED_OFFSET..TENANT_OFFSET]
            .iter()
            .any(|byte| *byte != 0)
        {
            return Err(invalid_frame("WAL frame reserved bytes are non-zero"));
        }
        let tenant_id = TenantId::from_u128(read_u128(bytes, TENANT_OFFSET)?)
            .ok_or_else(|| invalid_frame("WAL frame tenant identifier must be non-zero"))?;
        let region_id = optional_region(read_u128(bytes, REGION_OFFSET)?)?;
        let world_id = optional_world(read_u128(bytes, WORLD_OFFSET)?)?;
        let tx_id = TxId::from_u128(read_u128(bytes, TX_OFFSET)?)
            .ok_or_else(|| invalid_frame("WAL frame transaction identifier must be non-zero"))?;
        let crypto_epoch = CryptoEpoch::new(u64::from_le_bytes(read_array(bytes, EPOCH_OFFSET)?));
        let encryption_key_id = KeyId::from_u128(read_u128(bytes, KEY_OFFSET)?)
            .ok_or_else(|| invalid_frame("WAL frame encryption key identifier must be non-zero"))?;
        let encrypted_body_len = u64::from_le_bytes(read_array(bytes, BODY_LEN_OFFSET)?);
        let body_crc64_raw = u64::from_le_bytes(read_array(bytes, BODY_CRC_OFFSET)?);
        if body_crc64_raw == 0 {
            return Err(invalid_frame("WAL frame body CRC must not be zero"));
        }
        let body_crc64 = body_crc64_raw;
        let encryption_domain = EncryptionDomain::wal(tenant_id, region_id, world_id);
        Ok(Self {
            magic,
            version,
            record_kind,
            tenant_id,
            tx_id,
            encryption_key_id,
            crypto_epoch,
            encryption_domain,
            encrypted_body_len,
            body_crc64,
        })
    }

    pub fn encode(&self) -> [u8; WAL_FRAME_HEADER_BYTES] {
        let mut bytes = [0_u8; WAL_FRAME_HEADER_BYTES];
        bytes[..8].copy_from_slice(&self.magic);
        bytes[8..10].copy_from_slice(&self.version.to_le_bytes());
        bytes[KIND_OFFSET] = self.record_kind.code();
        bytes[TENANT_OFFSET..REGION_OFFSET].copy_from_slice(&self.tenant_id.get().to_le_bytes());
        let region_id = self
            .encryption_domain
            .region_id()
            .map_or(0, RegionKeyId::get);
        bytes[REGION_OFFSET..WORLD_OFFSET].copy_from_slice(&region_id.to_le_bytes());
        let world_id = self.encryption_domain.world_id().map_or(0, WorldId::get);
        bytes[WORLD_OFFSET..TX_OFFSET].copy_from_slice(&world_id.to_le_bytes());
        bytes[TX_OFFSET..EPOCH_OFFSET].copy_from_slice(&self.tx_id.get().to_le_bytes());
        bytes[EPOCH_OFFSET..KEY_OFFSET].copy_from_slice(&self.crypto_epoch.get().to_le_bytes());
        bytes[KEY_OFFSET..BODY_LEN_OFFSET]
            .copy_from_slice(&self.encryption_key_id.get().to_le_bytes());
        bytes[BODY_LEN_OFFSET..BODY_CRC_OFFSET]
            .copy_from_slice(&self.encrypted_body_len.to_le_bytes());
        bytes[BODY_CRC_OFFSET..WAL_FRAME_HEADER_BYTES]
            .copy_from_slice(&self.body_crc64.to_le_bytes());
        bytes
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
    pub const fn record_kind(&self) -> WalRecordKind {
        self.record_kind
    }

    #[must_use]
    pub const fn tenant_id(&self) -> TenantId {
        self.tenant_id
    }

    #[must_use]
    pub const fn tx_id(&self) -> TxId {
        self.tx_id
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
    pub const fn encrypted_body_len(&self) -> u64 {
        self.encrypted_body_len
    }

    #[must_use]
    pub const fn body_crc64(&self) -> BodyChecksum {
        BodyChecksum::Present(self.body_crc64)
    }

    pub fn validate(&self) -> Result<()> {
        if self.magic != WAL_FRAME_MAGIC {
            return Err(invalid_frame("WAL frame magic mismatch"));
        }
        if self.version == 0 {
            return Err(invalid_frame("WAL frame version must be non-zero"));
        }
        if self.version > WAL_FRAME_VERSION_MAX {
            return Err(invalid_frame("WAL frame version is newer than this parser"));
        }
        if self.encrypted_body_len == 0 {
            return Err(invalid_frame("WAL frame encrypted body must not be empty"));
        }
        if self.encrypted_body_len > WAL_FRAME_BODY_MAX_BYTES {
            return Err(invalid_frame(
                "WAL frame encrypted body exceeds maximum size",
            ));
        }
        if self.body_crc64 == 0 {
            return Err(invalid_frame("WAL frame body CRC must not be zero"));
        }
        if self.encryption_domain.purpose() != EncryptionDomainPurpose::Wal {
            return Err(invalid_frame("WAL frame encryption domain must be WAL"));
        }
        if self.encryption_domain.tenant_id().get() != self.tenant_id.get() {
            return Err(invalid_frame("WAL frame tenant/domain mismatch"));
        }
        Ok(())
    }

    pub fn validate_for_domain(&self, expected_domain: EncryptionDomain) -> Result<()> {
        self.validate()?;
        if !self.encryption_domain.structurally_equal(&expected_domain) {
            return Err(invalid_frame("WAL frame encryption domain mismatch"));
        }
        Ok(())
    }
}

fn read_array<const N: usize>(bytes: &[u8], offset: usize) -> Result<[u8; N]> {
    let end = offset
        .checked_add(N)
        .ok_or_else(|| invalid_frame("WAL frame header offset overflow"))?;
    let slice = bytes
        .get(offset..end)
        .ok_or_else(|| invalid_frame("WAL frame header field is truncated"))?;
    let mut output = [0_u8; N];
    output.copy_from_slice(slice);
    Ok(output)
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
            .ok_or_else(|| invalid_frame("WAL frame region identifier is invalid"))
    }
}

fn optional_world(value: u128) -> Result<Option<WorldId>> {
    if value == 0 {
        Ok(None)
    } else {
        WorldId::from_u128(value)
            .map(Some)
            .ok_or_else(|| invalid_frame("WAL frame world identifier is invalid"))
    }
}

fn invalid_frame(reason: &'static str) -> SkrifheimError {
    SkrifheimError::InvalidWalFrame(String::from(reason))
}

fn require_body_crc64(body_crc64: BodyChecksum) -> Result<u64> {
    match body_crc64 {
        BodyChecksum::Present(value) if value != 0 => Ok(value),
        BodyChecksum::Present(_) => Err(invalid_frame("WAL frame body CRC must not be zero")),
        BodyChecksum::Missing => Err(invalid_frame("WAL frame body CRC missing")),
    }
}

#[cfg(test)]
mod tests;
