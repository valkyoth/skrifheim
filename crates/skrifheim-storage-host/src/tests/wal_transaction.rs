use super::helpers::*;
use crate::{WalAppendOptions, WalCommitStatus, WalFileWriter, WalTransactionOutcome};
use std::fs;

#[test]
fn retry_is_idempotent_across_reopen_and_conflicts_do_not_write() -> WalResult<()> {
    let path = temp_path("idempotent")?;
    let body = b"encrypted fact";
    let header = header(101, body)?;
    {
        let mut writer = WalFileWriter::open_append(&path, WalAppendOptions::default())?;
        assert_eq!(
            writer.transaction_status(header.encryption_domain(), header.tx_id())?,
            WalCommitStatus::Absent
        );
        assert_eq!(
            writer.append_transaction_once(&header, body)?,
            WalTransactionOutcome::Durable
        );
        assert_eq!(
            writer.transaction_status(header.encryption_domain(), header.tx_id())?,
            WalCommitStatus::Committed
        );
    }
    let before = fs::read(&path)?;
    {
        let mut writer = WalFileWriter::open_append(&path, WalAppendOptions::default())?;
        assert_eq!(
            writer.append_transaction_once(&header, body)?,
            WalTransactionOutcome::AlreadyDurable
        );
        assert_eq!(fs::read(&path)?, before);
        let conflict = super::helpers::header(101, b"different fact")?;
        assert!(
            writer
                .append_transaction_once(&conflict, b"different fact")
                .is_err()
        );
        assert_eq!(fs::read(&path)?, before);
    }
    fs::remove_file(path)?;
    Ok(())
}

#[test]
fn interrupted_transaction_is_never_automatically_retried() -> WalResult<()> {
    let path = temp_path("incomplete-tx")?;
    let body = b"encrypted fact";
    let header = header(102, body)?;
    {
        let mut writer = WalFileWriter::open_append(&path, WalAppendOptions::default())?;
        assert_eq!(
            writer.append_transaction_once(&header, body)?,
            WalTransactionOutcome::Durable
        );
    }
    let bytes = fs::read(&path)?;
    // Only whole-frame boundaries reopen: begin only and begin plus batch.
    for length in [121, 121 + 120 + body.len()] {
        fs::write(&path, &bytes[..length])?;
        {
            let mut writer = WalFileWriter::open_append(&path, WalAppendOptions::default())?;
            assert_eq!(
                writer.transaction_status(header.encryption_domain(), header.tx_id())?,
                WalCommitStatus::Incomplete
            );
            assert!(writer.append_transaction_once(&header, body).is_err());
        }
        assert_eq!(fs::read(&path)?, bytes[..length]);
    }
    fs::remove_file(path)?;
    Ok(())
}

#[test]
fn status_rejects_key_epoch_changes_and_noncanonical_markers() -> WalResult<()> {
    let path = temp_path("status-binding")?;
    let body = b"encrypted fact";
    let header = header(103, body)?;
    {
        let mut writer = WalFileWriter::open_append(&path, WalAppendOptions::default())?;
        assert_eq!(
            writer.append_transaction_once(&header, body)?,
            WalTransactionOutcome::Durable
        );
    }
    let original = fs::read(&path)?;
    for offset in [121 + 80, 121 + 88, 120] {
        let mut bytes = original.clone();
        bytes[offset] ^= 2;
        if offset == 120 {
            let crc = skrifheim_storage::wal_body_crc64(&bytes[120..121]);
            bytes[112..120].copy_from_slice(&crc.to_le_bytes());
        }
        fs::write(&path, &bytes)?;
        {
            let mut writer = WalFileWriter::open_append(&path, WalAppendOptions::default())?;
            assert!(
                writer
                    .transaction_status(header.encryption_domain(), header.tx_id())
                    .is_err()
            );
        }
        assert_eq!(fs::read(&path)?, bytes);
    }
    fs::remove_file(path)?;
    Ok(())
}
