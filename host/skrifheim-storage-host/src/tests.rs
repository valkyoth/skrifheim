use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

use skrifheim_core::{Result as SkrifheimResult, SkrifheimError, TenantId, TxId, WorldId};
use skrifheim_crypto::{CryptoEpoch, EncryptionDomain, KeyId, RegionKeyId};
use skrifheim_storage::{BodyChecksum, WalFrameHeader, WalFrameHeaderInput, WalRecordKind};

use super::*;

fn id<T>(id: Option<T>) -> SkrifheimResult<T> {
    id.ok_or(SkrifheimError::InvalidIdentifier)
}

fn tenant() -> SkrifheimResult<TenantId> {
    id(TenantId::from_u128(1))
}

fn wal_domain() -> SkrifheimResult<EncryptionDomain> {
    Ok(EncryptionDomain::wal(
        tenant()?,
        Some(id(RegionKeyId::from_u128(2))?),
        Some(id(WorldId::from_u128(3))?),
    ))
}

fn header(tx: u128, body_len: u64) -> SkrifheimResult<WalFrameHeader> {
    WalFrameHeader::new(WalFrameHeaderInput {
        record_kind: WalRecordKind::FactBatch,
        tenant_id: tenant()?,
        tx_id: id(TxId::from_u128(tx))?,
        encryption_key_id: id(KeyId::from_u128(4))?,
        crypto_epoch: CryptoEpoch::new(5),
        encryption_domain: wal_domain()?,
        encrypted_body_len: body_len,
        body_crc64: BodyChecksum::Present(tx as u64 + 10),
    })
}

fn temp_path(name: &str) -> Result<PathBuf> {
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

#[test]
fn wal_writer_and_reader_round_trip_encrypted_frames() -> Result<()> {
    let path = temp_path("round-trip")?;
    let first_body = [11_u8; 4];
    let second_body = [12_u8; 5];
    {
        let mut writer = WalFileWriter::open_append(&path, WalAppendOptions::new(false))?;
        writer.append_frame(&header(10, first_body.len() as u64)?, &first_body)?;
        writer.append_frame(&header(11, second_body.len() as u64)?, &second_body)?;
    }

    let mut reader = WalFileReader::open(&path, wal_domain()?)?;
    let first = reader.next_frame()?.ok_or(WalFileError::PartialFrame)?;
    let second = reader.next_frame()?.ok_or(WalFileError::PartialFrame)?;

    assert_eq!(first.header().tx_id().get(), 10);
    assert_eq!(first.encrypted_body(), first_body);
    assert_eq!(second.header().tx_id().get(), 11);
    assert_eq!(second.encrypted_body(), second_body);
    assert!(reader.next_frame()?.is_none());
    fs::remove_file(path)?;
    Ok(())
}

#[test]
fn wal_writer_rejects_body_length_mismatch() -> Result<()> {
    let path = temp_path("length-mismatch")?;
    let mut writer = WalFileWriter::open_append(&path, WalAppendOptions::new(false))?;
    let result = writer.append_frame(&header(12, 4)?, &[1, 2, 3]);

    assert!(matches!(
        result,
        Err(WalFileError::InvalidFrame(SkrifheimError::InvalidWalFrame(
            _
        )))
    ));
    fs::remove_file(path)?;
    Ok(())
}

#[cfg(unix)]
#[test]
fn wal_writer_creates_owner_only_files() -> Result<()> {
    let path = temp_path("permissions")?;
    {
        let _writer = WalFileWriter::open_append(&path, WalAppendOptions::new(false))?;
    }
    let mode = fs::metadata(&path)?.permissions().mode() & 0o777;

    assert_eq!(mode, 0o600);
    fs::remove_file(path)?;
    Ok(())
}

#[test]
fn wal_reader_rejects_unexpected_domain() -> Result<()> {
    let path = temp_path("domain")?;
    let body = [21_u8; 4];
    {
        let mut writer = WalFileWriter::open_append(&path, WalAppendOptions::new(false))?;
        writer.append_frame(&header(13, body.len() as u64)?, &body)?;
    }
    let other_domain = EncryptionDomain::wal(tenant()?, None, None);
    let mut reader = WalFileReader::open(&path, other_domain)?;

    assert!(matches!(
        reader.next_frame(),
        Err(WalFileError::InvalidFrame(SkrifheimError::InvalidWalFrame(
            _
        )))
    ));
    fs::remove_file(path)?;
    Ok(())
}

#[test]
fn wal_reader_detects_partial_header_and_body() -> Result<()> {
    let header_path = temp_path("partial-header")?;
    fs::write(&header_path, [1_u8; 4])?;
    let mut reader = WalFileReader::open(&header_path, wal_domain()?)?;
    assert!(matches!(
        reader.next_frame(),
        Err(WalFileError::PartialFrame)
    ));
    fs::remove_file(header_path)?;

    let body_path = temp_path("partial-body")?;
    let header = header(14, 4)?;
    {
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&body_path)?;
        file.write_all(&header.encode())?;
        file.write_all(&[1_u8; 2])?;
        file.flush()?;
    }
    let mut reader = WalFileReader::open(&body_path, wal_domain()?)?;
    assert!(matches!(
        reader.next_frame(),
        Err(WalFileError::PartialFrame)
    ));
    fs::remove_file(body_path)?;
    Ok(())
}
