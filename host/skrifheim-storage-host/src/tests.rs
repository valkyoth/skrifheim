use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

#[cfg(unix)]
use std::os::unix::fs::{PermissionsExt, symlink};

use skrifheim_core::{Result as SkrifheimResult, SkrifheimError, TenantId, TxId, WorldId};
use skrifheim_crypto::{
    CompartmentKeyId, ContentDigest, CryptoEpoch, DigestPolicy, EncryptionDomain, KeyId,
    RegionKeyId, SegmentKeyId,
};
use skrifheim_storage::{
    BodyChecksum, SegmentFooter, SegmentHeader, SegmentHeaderInput, SegmentKind, WalFrameHeader,
    WalFrameHeaderInput, WalRecordKind, wal_body_crc64,
};

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

fn segment_domain() -> SkrifheimResult<EncryptionDomain> {
    Ok(EncryptionDomain::segment(
        tenant()?,
        Some(id(RegionKeyId::from_u128(8))?),
        skrifheim_core::Classification::Restricted,
        id(CompartmentKeyId::from_u128(9))?,
        id(SegmentKeyId::from_u128(10))?,
    ))
}

fn header(tx: u128, body: &[u8]) -> SkrifheimResult<WalFrameHeader> {
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

fn segment_header(body: &[u8]) -> SkrifheimResult<SegmentHeader> {
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

fn wal_to_segment_error(error: WalFileError) -> SegmentFileError {
    match error {
        WalFileError::Io(error) => SegmentFileError::Io(error),
        WalFileError::InvalidFrame(error) => SegmentFileError::InvalidSegment(error),
        WalFileError::BodyLengthMismatch => SegmentFileError::BodyLengthMismatch,
        WalFileError::PartialFrame => SegmentFileError::PartialSegment,
    }
}

struct AcceptDigest;

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

struct RejectDigest;

impl SegmentContentVerifier for RejectDigest {
    fn verify_content_digest(
        &self,
        _header: &SegmentHeader,
        _encrypted_body: &[u8],
    ) -> SkrifheimResult<()> {
        Err(SkrifheimError::InvalidDigest)
    }
}

#[test]
fn wal_writer_and_reader_round_trip_encrypted_frames() -> Result<()> {
    let path = temp_path("round-trip")?;
    let first_body = [11_u8; 4];
    let second_body = [12_u8; 5];
    {
        let mut writer = WalFileWriter::open_append(&path, WalAppendOptions::new(false))?;
        writer.append_frame(&header(10, &first_body)?, &first_body)?;
        writer.append_frame(&header(11, &second_body)?, &second_body)?;
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
fn segment_writer_and_reader_round_trip_encrypted_segment()
-> core::result::Result<(), SegmentFileError> {
    let path = temp_path("segment-round-trip").map_err(wal_to_segment_error)?;
    let body = [41_u8; 16];
    let header = segment_header(&body)?;
    {
        let mut writer = SegmentFileWriter::create(&path, SegmentWriteOptions::new(false))?;
        writer.write_segment(&header, &body, &AcceptDigest)?;
    }

    let mut reader = SegmentFileReader::open(&path, segment_domain()?)?;
    let segment = reader.read_segment(&AcceptDigest)?;

    assert_eq!(segment.header().min_tx().get(), 20);
    assert_eq!(segment.footer().max_tx().get(), 21);
    assert_eq!(segment.encrypted_body(), body);
    fs::remove_file(path)?;
    Ok(())
}

#[test]
fn segment_writer_rejects_body_length_mismatch() -> core::result::Result<(), SegmentFileError> {
    let path = temp_path("segment-length-mismatch").map_err(wal_to_segment_error)?;
    let mut writer = SegmentFileWriter::create(&path, SegmentWriteOptions::new(false))?;
    let result = writer.write_segment(&segment_header(&[1, 2, 3, 4])?, &[1, 2, 3], &AcceptDigest);

    assert!(matches!(result, Err(SegmentFileError::BodyLengthMismatch)));
    fs::remove_file(path)?;
    Ok(())
}

#[test]
fn segment_writer_requires_content_digest_verifier() -> core::result::Result<(), SegmentFileError> {
    let path = temp_path("segment-write-digest-verifier").map_err(wal_to_segment_error)?;
    let body = [6_u8; 4];
    let mut writer = SegmentFileWriter::create(&path, SegmentWriteOptions::new(false))?;
    let result = writer.write_segment(&segment_header(&body)?, &body, &RejectDigest);

    assert!(matches!(
        result,
        Err(SegmentFileError::ContentDigestRejected(
            SkrifheimError::InvalidDigest
        ))
    ));
    fs::remove_file(path)?;
    Ok(())
}

#[test]
fn segment_reader_rejects_body_crc_mismatch() -> core::result::Result<(), SegmentFileError> {
    let path = temp_path("segment-crc-mismatch").map_err(wal_to_segment_error)?;
    let original_body = [1_u8, 2, 3, 4];
    let tampered_body = [1_u8, 2, 3, 5];
    let header = segment_header(&original_body)?;
    let footer = SegmentFooter::from_header(&header)?;
    {
        let mut file = OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&path)?;
        file.write_all(&header.encode())?;
        file.write_all(&tampered_body)?;
        file.write_all(&footer.encode())?;
        file.flush()?;
    }
    let mut reader = SegmentFileReader::open(&path, segment_domain()?)?;

    assert!(matches!(
        reader.read_segment(&AcceptDigest),
        Err(SegmentFileError::InvalidSegment(
            SkrifheimError::InvalidStorageHeader(_)
        ))
    ));
    fs::remove_file(path)?;
    Ok(())
}

#[test]
fn segment_reader_rejects_footer_header_mismatch() -> core::result::Result<(), SegmentFileError> {
    let path = temp_path("segment-footer-mismatch").map_err(wal_to_segment_error)?;
    let body = [7_u8; 8];
    let header = segment_header(&body)?;
    let different_body = [8_u8; 8];
    let footer = SegmentFooter::from_header(&segment_header(&different_body)?)?;
    {
        let mut file = OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&path)?;
        file.write_all(&header.encode())?;
        file.write_all(&body)?;
        file.write_all(&footer.encode())?;
        file.flush()?;
    }
    let mut reader = SegmentFileReader::open(&path, segment_domain()?)?;

    assert!(matches!(
        reader.read_segment(&AcceptDigest),
        Err(SegmentFileError::InvalidSegment(
            SkrifheimError::InvalidStorageHeader(_)
        ))
    ));
    fs::remove_file(path)?;
    Ok(())
}

#[test]
fn segment_reader_rejects_unexpected_domain() -> core::result::Result<(), SegmentFileError> {
    let path = temp_path("segment-domain").map_err(wal_to_segment_error)?;
    let body = [21_u8; 4];
    {
        let mut writer = SegmentFileWriter::create(&path, SegmentWriteOptions::new(false))?;
        writer.write_segment(&segment_header(&body)?, &body, &AcceptDigest)?;
    }
    let other_domain = EncryptionDomain::segment(
        tenant()?,
        None,
        skrifheim_core::Classification::Restricted,
        id(CompartmentKeyId::from_u128(9))?,
        id(SegmentKeyId::from_u128(10))?,
    );
    let mut reader = SegmentFileReader::open(&path, other_domain)?;

    assert!(matches!(
        reader.read_segment(&AcceptDigest),
        Err(SegmentFileError::InvalidSegment(
            SkrifheimError::InvalidStorageHeader(_)
        ))
    ));
    fs::remove_file(path)?;
    Ok(())
}

#[test]
fn segment_reader_rejects_partial_segment_and_trailing_bytes()
-> core::result::Result<(), SegmentFileError> {
    let partial_path = temp_path("segment-partial").map_err(wal_to_segment_error)?;
    fs::write(&partial_path, [1_u8; 4])?;
    let mut reader = SegmentFileReader::open(&partial_path, segment_domain()?)?;
    assert!(matches!(
        reader.read_segment(&AcceptDigest),
        Err(SegmentFileError::PartialSegment)
    ));
    fs::remove_file(partial_path)?;

    let trailing_path = temp_path("segment-trailing").map_err(wal_to_segment_error)?;
    let body = [5_u8; 4];
    let header = segment_header(&body)?;
    let footer = SegmentFooter::from_header(&header)?;
    {
        let mut file = OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&trailing_path)?;
        file.write_all(&header.encode())?;
        file.write_all(&body)?;
        file.write_all(&footer.encode())?;
        file.write_all(&[0_u8])?;
        file.flush()?;
    }
    let mut reader = SegmentFileReader::open(&trailing_path, segment_domain()?)?;
    assert!(matches!(
        reader.read_segment(&AcceptDigest),
        Err(SegmentFileError::FileLengthMismatch)
    ));
    fs::remove_file(trailing_path)?;
    Ok(())
}

#[test]
fn segment_reader_requires_content_digest_verifier() -> core::result::Result<(), SegmentFileError> {
    let path = temp_path("segment-digest-verifier").map_err(wal_to_segment_error)?;
    let body = [9_u8; 4];
    {
        let mut writer = SegmentFileWriter::create(&path, SegmentWriteOptions::new(false))?;
        writer.write_segment(&segment_header(&body)?, &body, &AcceptDigest)?;
    }
    let mut reader = SegmentFileReader::open(&path, segment_domain()?)?;

    assert!(matches!(
        reader.read_segment(&RejectDigest),
        Err(SegmentFileError::ContentDigestRejected(
            SkrifheimError::InvalidDigest
        ))
    ));
    fs::remove_file(path)?;
    Ok(())
}

#[cfg(unix)]
#[test]
fn segment_writer_creates_owner_only_files() -> core::result::Result<(), SegmentFileError> {
    let path = temp_path("segment-permissions").map_err(wal_to_segment_error)?;
    let body = [3_u8; 4];
    {
        let mut writer = SegmentFileWriter::create(&path, SegmentWriteOptions::new(false))?;
        writer.write_segment(&segment_header(&body)?, &body, &AcceptDigest)?;
    }
    let mode = fs::metadata(&path)?.permissions().mode() & 0o777;

    assert_eq!(mode, 0o600);
    fs::remove_file(path)?;
    Ok(())
}

#[cfg(unix)]
#[test]
fn segment_reader_and_writer_reject_symlink_paths() -> core::result::Result<(), SegmentFileError> {
    let target = temp_path("segment-symlink-target").map_err(wal_to_segment_error)?;
    let link = temp_path("segment-symlink-link").map_err(wal_to_segment_error)?;
    fs::write(&target, [])?;
    symlink(&target, &link)?;

    assert!(matches!(
        SegmentFileWriter::create(&link, SegmentWriteOptions::new(false)),
        Err(SegmentFileError::Io(_))
    ));
    assert!(matches!(
        SegmentFileReader::open(&link, segment_domain()?),
        Err(SegmentFileError::Io(_))
    ));

    fs::remove_file(link)?;
    fs::remove_file(target)?;
    Ok(())
}

#[test]
fn wal_writer_rejects_body_length_mismatch() -> Result<()> {
    let path = temp_path("length-mismatch")?;
    let mut writer = WalFileWriter::open_append(&path, WalAppendOptions::new(false))?;
    let result = writer.append_frame(&header(12, &[1, 2, 3, 4])?, &[1, 2, 3]);

    assert!(matches!(
        result,
        Err(WalFileError::InvalidFrame(SkrifheimError::InvalidWalFrame(
            _
        )))
    ));
    fs::remove_file(path)?;
    Ok(())
}

#[test]
fn wal_reader_rejects_body_crc_mismatch() -> Result<()> {
    let path = temp_path("crc-mismatch")?;
    let original_body = [1_u8, 2, 3, 4];
    let tampered_body = [1_u8, 2, 3, 5];
    let header = header(15, &original_body)?;
    {
        let mut file = OpenOptions::new().create(true).append(true).open(&path)?;
        file.write_all(&header.encode())?;
        file.write_all(&tampered_body)?;
        file.flush()?;
    }
    let mut reader = WalFileReader::open(&path, wal_domain()?)?;

    assert!(matches!(
        reader.next_frame(),
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

#[cfg(unix)]
#[test]
fn wal_writer_tightens_existing_file_permissions() -> Result<()> {
    let path = temp_path("existing-permissions")?;
    fs::write(&path, [])?;
    fs::set_permissions(&path, fs::Permissions::from_mode(0o644))?;
    {
        let _writer = WalFileWriter::open_append(&path, WalAppendOptions::new(false))?;
    }
    let mode = fs::metadata(&path)?.permissions().mode() & 0o777;

    assert_eq!(mode, 0o600);
    fs::remove_file(path)?;
    Ok(())
}

#[cfg(unix)]
#[test]
fn wal_writer_rejects_symlink_paths() -> Result<()> {
    let target = temp_path("symlink-target")?;
    let link = temp_path("symlink-link")?;
    fs::write(&target, [])?;
    symlink(&target, &link)?;

    assert!(matches!(
        WalFileWriter::open_append(&link, WalAppendOptions::new(false)),
        Err(WalFileError::Io(_))
    ));

    fs::remove_file(link)?;
    fs::remove_file(target)?;
    Ok(())
}

#[cfg(unix)]
#[test]
fn wal_reader_rejects_symlink_paths() -> Result<()> {
    let target = temp_path("read-symlink-target")?;
    let link = temp_path("read-symlink-link")?;
    fs::write(&target, [])?;
    symlink(&target, &link)?;

    assert!(matches!(
        WalFileReader::open(&link, wal_domain()?),
        Err(WalFileError::Io(_))
    ));

    fs::remove_file(link)?;
    fs::remove_file(target)?;
    Ok(())
}

#[test]
fn wal_reader_rejects_unexpected_domain() -> Result<()> {
    let path = temp_path("domain")?;
    let body = [21_u8; 4];
    {
        let mut writer = WalFileWriter::open_append(&path, WalAppendOptions::new(false))?;
        writer.append_frame(&header(13, &body)?, &body)?;
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
    let header = header(14, &[1_u8; 4])?;
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
