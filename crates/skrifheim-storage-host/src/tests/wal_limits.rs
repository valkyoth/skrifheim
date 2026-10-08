use super::helpers::*;
use crate::{
    WAL_FILE_MAX_BYTES, WAL_FILE_MAX_FRAMES, WalAppendOptions, WalCommitStatus, WalFileError,
    WalFileLimits, WalFileReader, WalFileWriter, WalTransactionOutcome,
};
use skrifheim_storage::WAL_FRAME_HEADER_BYTES;
use std::fs;

fn options(bytes: u64, frames: u64) -> WalResult<WalAppendOptions> {
    Ok(WalAppendOptions::default().with_limits(WalFileLimits::new(bytes, frames)?))
}

#[test]
fn budgets_cannot_disable_or_exceed_hard_scan_limits() {
    for (bytes, frames) in [
        (0, 1),
        (1, 0),
        (WAL_FILE_MAX_BYTES + 1, 1),
        (1, WAL_FILE_MAX_FRAMES + 1),
        (u64::MAX, u64::MAX),
    ] {
        assert!(WalFileLimits::new(bytes, frames).is_err());
    }
    assert!(WalFileLimits::new(WAL_FILE_MAX_BYTES, WAL_FILE_MAX_FRAMES).is_ok());
}

#[test]
fn transaction_capacity_is_reserved_before_begin_and_retries_still_work_at_limit() -> WalResult<()>
{
    let body = [1];
    let batch = header(10, &body)?;
    let transaction_bytes = 3 * (WAL_FRAME_HEADER_BYTES as u64 + 1);
    for limits in [
        options(transaction_bytes, 100)?,
        options(WAL_FILE_MAX_BYTES, 3)?,
    ] {
        let path = temp_path("transaction-budget")?;
        let mut writer = WalFileWriter::open_append(&path, wal_domain()?, limits)?;
        assert_eq!(
            writer.append_transaction_once(&batch, &body)?,
            WalTransactionOutcome::Durable
        );
        let before = fs::read(&path)?;
        assert!(matches!(
            writer.append_transaction_once(&header(11, &body)?, &body),
            Err(WalFileError::RotationRequired)
        ));
        assert_eq!(fs::read(&path)?, before);
        assert!(!writer.is_poisoned());
        assert_eq!(
            writer.transaction_status(wal_domain()?, batch.tx_id())?,
            WalCommitStatus::Committed
        );
        assert_eq!(
            writer.append_transaction_once(&batch, &body)?,
            WalTransactionOutcome::AlreadyDurable
        );
        drop(writer);
        let mut writer = WalFileWriter::open_append(&path, wal_domain()?, limits)?;
        assert!(matches!(
            writer.append_transaction_once(&header(11, &body)?, &body),
            Err(WalFileError::RotationRequired)
        ));
        assert_eq!(fs::read(&path)?, before);
        drop(writer);
        fs::remove_file(path)?;
    }
    // Not enough space for all three frames: do not leave a begin-only tail.
    for limits in [
        options(transaction_bytes - 1, 100)?,
        options(WAL_FILE_MAX_BYTES, 2)?,
    ] {
        let path = temp_path("whole-transaction-reservation")?;
        let mut writer = WalFileWriter::open_append(&path, wal_domain()?, limits)?;
        assert!(matches!(
            writer.append_transaction_once(&batch, &body),
            Err(WalFileError::RotationRequired)
        ));
        assert_eq!(fs::metadata(&path)?.len(), 0);
        assert!(!writer.is_poisoned());
        drop(writer);
        fs::remove_file(path)?;
    }
    Ok(())
}

#[test]
fn raw_append_and_reopen_obey_scan_limits_without_deleting_evidence() -> WalResult<()> {
    let path = temp_path("raw-budget")?;
    let body = [1];
    let batch = header(10, &body)?;
    let bytes = WAL_FRAME_HEADER_BYTES as u64 + 1;
    let mut writer = WalFileWriter::open_append(&path, wal_domain()?, options(bytes, 1)?)?;
    let _outcome = writer.append_frame(&batch, &body)?;
    let before = fs::read(&path)?;
    assert!(matches!(
        writer.append_frame(&batch, &body),
        Err(WalFileError::RotationRequired)
    ));
    drop(writer);
    assert!(matches!(
        WalFileWriter::open_append(&path, wal_domain()?, options(bytes - 1, 1)?),
        Err(WalFileError::RotationRequired)
    ));
    let mut doubled = before.clone();
    doubled.extend_from_slice(&before);
    fs::write(&path, &doubled)?;
    assert!(matches!(
        WalFileWriter::open_append(&path, wal_domain()?, options(2 * bytes, 1)?),
        Err(WalFileError::RotationRequired)
    ));
    assert_eq!(fs::read(&path)?, doubled);
    // The streaming reader is available for explicit recovery, not blocked by a writer budget.
    let mut reader = WalFileReader::open(&path, wal_domain()?)?;
    assert!(reader.next_frame()?.is_some());
    assert!(reader.next_frame()?.is_some());
    assert!(reader.next_frame()?.is_none());
    drop(reader);
    fs::remove_file(path)?;
    Ok(())
}

#[test]
fn oversized_sparse_wal_is_refused_before_parsing_or_scanning() -> WalResult<()> {
    let path = temp_path("oversized-scan")?;
    let file = fs::File::create(&path)?;
    file.set_len(WAL_FILE_MAX_BYTES + 1)?;
    drop(file);
    assert!(matches!(
        WalFileWriter::open_append(&path, wal_domain()?, WalAppendOptions::default()),
        Err(WalFileError::RotationRequired)
    ));
    assert_eq!(fs::metadata(&path)?.len(), WAL_FILE_MAX_BYTES + 1);
    fs::remove_file(path)?;
    Ok(())
}
