use std::{
    fs::{self, OpenOptions},
    io::Write,
};

#[cfg(unix)]
use std::os::unix::fs::{PermissionsExt, symlink};

use skrifheim_core::SkrifheimError;
use skrifheim_crypto::EncryptionDomain;

use crate::{WalAppendOptions, WalFileError, WalFileReader, WalFileWriter};

use super::helpers::*;

#[test]
fn wal_writer_and_reader_round_trip_encrypted_frames() -> WalResult<()> {
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
fn wal_writer_rejects_body_length_mismatch() -> WalResult<()> {
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
fn wal_reader_rejects_body_crc_mismatch() -> WalResult<()> {
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
fn wal_writer_creates_owner_only_files() -> WalResult<()> {
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
fn wal_writer_tightens_existing_file_permissions() -> WalResult<()> {
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
fn wal_writer_rejects_second_concurrent_writer() -> WalResult<()> {
    let path = temp_path("exclusive-writer")?;
    let writer = WalFileWriter::open_append(&path, WalAppendOptions::new(false))?;

    assert!(matches!(
        WalFileWriter::open_append(&path, WalAppendOptions::new(false)),
        Err(WalFileError::Io(_))
    ));

    drop(writer);
    fs::remove_file(path)?;
    Ok(())
}

#[cfg(unix)]
#[test]
fn wal_writer_requires_explicit_parent_for_new_files() -> WalResult<()> {
    let path = format!(
        "skrifheim-bare-wal-{}-{}.wal",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|error| WalFileError::Io(std::io::Error::other(error)))?
            .as_nanos()
    );
    let result = WalFileWriter::open_append(&path, WalAppendOptions::new(false));

    assert!(matches!(result, Err(WalFileError::Io(_))));
    assert!(!std::path::Path::new(&path).exists());
    Ok(())
}

#[cfg(unix)]
#[test]
fn wal_writer_rejects_symlink_paths() -> WalResult<()> {
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
fn wal_reader_rejects_symlink_paths() -> WalResult<()> {
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
fn wal_reader_rejects_unexpected_domain() -> WalResult<()> {
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
fn wal_reader_detects_partial_header_and_body() -> WalResult<()> {
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
