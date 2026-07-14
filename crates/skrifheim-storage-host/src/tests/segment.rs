use std::{
    fs::{self, OpenOptions},
    io::{Seek, SeekFrom, Write},
};

#[cfg(unix)]
use std::os::unix::fs::{PermissionsExt, symlink};

use skrifheim_core::SkrifheimError;
use skrifheim_crypto::{CompartmentKeyId, EncryptionDomain, SegmentKeyId};
use skrifheim_storage::{SEGMENT_FOOTER_BYTES, SEGMENT_HEADER_BYTES, SegmentFooter};

use crate::{
    MAX_IN_MEMORY_SEGMENT_BYTES, SegmentFileError, SegmentFileReader, SegmentFileWriter,
    SegmentWriteOptions,
};

use super::helpers::*;

#[test]
fn segment_writer_and_reader_round_trip_encrypted_segment() -> SegmentResult<()> {
    let path = temp_path("segment-round-trip").map_err(wal_to_segment_error)?;
    let body = [41_u8; 16];
    let header = segment_header(&body)?;
    {
        let writer = SegmentFileWriter::create(&path, SegmentWriteOptions::new(false))?;
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
fn segment_writer_rejects_body_length_mismatch() -> SegmentResult<()> {
    let path = temp_path("segment-length-mismatch").map_err(wal_to_segment_error)?;
    let writer = SegmentFileWriter::create(&path, SegmentWriteOptions::new(false))?;
    let result = writer.write_segment(&segment_header(&[1, 2, 3, 4])?, &[1, 2, 3], &AcceptDigest);

    assert!(matches!(result, Err(SegmentFileError::BodyLengthMismatch)));
    fs::remove_file(path)?;
    Ok(())
}

#[test]
fn segment_writer_requires_content_digest_verifier() -> SegmentResult<()> {
    let path = temp_path("segment-write-digest-verifier").map_err(wal_to_segment_error)?;
    let body = [6_u8; 4];
    let writer = SegmentFileWriter::create(&path, SegmentWriteOptions::new(false))?;
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
fn segment_reader_rejects_body_crc_mismatch() -> SegmentResult<()> {
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
fn segment_reader_rejects_footer_header_mismatch() -> SegmentResult<()> {
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
fn segment_reader_rejects_unexpected_domain() -> SegmentResult<()> {
    let path = temp_path("segment-domain").map_err(wal_to_segment_error)?;
    let body = [21_u8; 4];
    {
        let writer = SegmentFileWriter::create(&path, SegmentWriteOptions::new(false))?;
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
fn segment_reader_rejects_partial_segment_and_trailing_bytes() -> SegmentResult<()> {
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
fn segment_reader_requires_content_digest_verifier() -> SegmentResult<()> {
    let path = temp_path("segment-digest-verifier").map_err(wal_to_segment_error)?;
    let body = [9_u8; 4];
    {
        let writer = SegmentFileWriter::create(&path, SegmentWriteOptions::new(false))?;
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

#[test]
fn segment_reader_rejects_oversized_sparse_segment_before_allocation() -> SegmentResult<()> {
    let path = temp_path("segment-sparse-oversized").map_err(wal_to_segment_error)?;
    let mut input = segment_header_input(&[1_u8; 4])?;
    input.body_len = MAX_IN_MEMORY_SEGMENT_BYTES + 1;
    let header = skrifheim_storage::SegmentHeader::new(input)?;
    let expected_len =
        (SEGMENT_HEADER_BYTES as u64) + header.body_len() + (SEGMENT_FOOTER_BYTES as u64);
    {
        let mut file = OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&path)?;
        file.write_all(&header.encode())?;
        file.seek(SeekFrom::Start(expected_len - 1))?;
        file.write_all(&[0])?;
        file.flush()?;
    }
    let mut reader = SegmentFileReader::open(&path, segment_domain()?)?;

    assert!(matches!(
        reader.read_segment(&AcceptDigest),
        Err(SegmentFileError::ResourceLimitExceeded)
    ));
    fs::remove_file(path)?;
    Ok(())
}

#[cfg(unix)]
#[test]
fn segment_writer_creates_owner_only_files() -> SegmentResult<()> {
    let path = temp_path("segment-permissions").map_err(wal_to_segment_error)?;
    let body = [3_u8; 4];
    {
        let writer = SegmentFileWriter::create(&path, SegmentWriteOptions::new(false))?;
        writer.write_segment(&segment_header(&body)?, &body, &AcceptDigest)?;
    }
    let mode = fs::metadata(&path)?.permissions().mode() & 0o777;

    assert_eq!(mode, 0o600);
    fs::remove_file(path)?;
    Ok(())
}

#[cfg(unix)]
#[test]
fn segment_writer_requires_explicit_parent_for_new_files() -> SegmentResult<()> {
    let path = format!(
        "skrifheim-bare-segment-{}-{}.seg",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|error| SegmentFileError::Io(std::io::Error::other(error)))?
            .as_nanos()
    );
    let result = SegmentFileWriter::create(&path, SegmentWriteOptions::new(false));

    assert!(matches!(result, Err(SegmentFileError::Io(_))));
    let _ = fs::remove_file(path);
    Ok(())
}

#[cfg(unix)]
#[test]
fn segment_reader_and_writer_reject_symlink_paths() -> SegmentResult<()> {
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
