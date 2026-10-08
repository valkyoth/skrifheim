use super::helpers::*;
use crate::{WalAppendOptions, WalFileReader, WalFileWriter, WalTransactionOutcome};
use skrifheim_storage::{
    WalFrameHeader, WalFrameHeaderInput, WalRecordKind, WalReplay, WalReplayStop,
};
use std::fs;

fn frame(tx: u128, kind: WalRecordKind) -> WalResult<WalFrameHeader> {
    let batch = header(tx, &[1])?;
    Ok(WalFrameHeader::new(WalFrameHeaderInput {
        record_kind: kind,
        tenant_id: batch.tenant_id(),
        tx_id: batch.tx_id(),
        encryption_key_id: batch.encryption_key_id(),
        crypto_epoch: batch.crypto_epoch(),
        encryption_domain: batch.encryption_domain(),
        encrypted_body_len: batch.encrypted_body_len(),
        body_crc64: skrifheim_storage::BodyChecksum::Present(batch.body_crc64()),
    })?)
}

#[test]
fn incomplete_other_transaction_blocks_append_without_writing() -> WalResult<()> {
    let path = temp_path("incomplete-other")?;
    let mut writer = WalFileWriter::open_append(&path, wal_domain()?, WalAppendOptions::default())?;
    let _outcome = writer.append_frame(&frame(10, WalRecordKind::TransactionBegin)?, &[1])?;
    let before = fs::read(&path)?;
    assert!(
        writer
            .append_transaction_once(&header(11, &[1])?, &[1])
            .is_err()
    );
    assert_eq!(fs::read(&path)?, before);
    drop(writer);
    fs::remove_file(path)?;
    Ok(())
}

#[test]
fn nonadvancing_transaction_is_rejected_and_later_valid_commit_replays() -> WalResult<()> {
    let path = temp_path("nonadvancing")?;
    let mut writer = WalFileWriter::open_append(&path, wal_domain()?, WalAppendOptions::default())?;
    let _outcome = writer.append_transaction_once(&header(10, &[1])?, &[1])?;
    let before = fs::read(&path)?;
    assert!(
        writer
            .append_transaction_once(&header(9, &[1])?, &[1])
            .is_err()
    );
    assert_eq!(fs::read(&path)?, before);
    assert_eq!(
        writer.append_transaction_once(&header(11, &[1])?, &[1])?,
        WalTransactionOutcome::Durable
    );
    assert_eq!(
        writer.append_transaction_once(&header(10, &[1])?, &[1])?,
        WalTransactionOutcome::AlreadyDurable
    );
    drop(writer);
    let mut reader = WalFileReader::open(&path, wal_domain()?)?;
    let mut replay = WalReplay::new();
    while let Some(frame) = reader.next_frame()? {
        replay.process_header(frame.header())?;
    }
    let report = replay.finish(WalReplayStop::CleanEof)?;
    assert_eq!(report.committed_transactions().len(), 2);
    drop(reader);
    fs::remove_file(path)?;
    Ok(())
}

#[test]
fn globally_invalid_wal_cannot_report_commit_or_append() -> WalResult<()> {
    use WalRecordKind::{FactBatch, TransactionBegin as Begin, TransactionCommit as Commit};
    // Target 10 looks committed in isolation. Other frames invalidate replay.
    for records in [
        vec![(10, Begin), (11, Begin), (10, FactBatch), (10, Commit)],
        vec![(9, FactBatch), (10, Begin), (10, FactBatch), (10, Commit)],
        vec![(10, Begin), (10, FactBatch), (10, Commit), (9, Begin)],
    ] {
        let path = temp_path("global-order")?;
        let mut writer =
            WalFileWriter::open_append(&path, wal_domain()?, WalAppendOptions::default())?;
        for (tx, kind) in records {
            let _outcome = writer.append_frame(&frame(tx, kind)?, &[1])?;
        }
        let before = fs::read(&path)?;
        assert!(
            writer
                .transaction_status(wal_domain()?, header(10, &[1])?.tx_id())
                .is_err()
        );
        assert!(writer.is_poisoned());
        drop(writer);
        let mut writer =
            WalFileWriter::open_append(&path, wal_domain()?, WalAppendOptions::default())?;
        assert!(
            writer
                .append_transaction_once(&header(12, &[1])?, &[1])
                .is_err()
        );
        assert_eq!(fs::read(&path)?, before);
        drop(writer);
        fs::remove_file(path)?;
    }
    Ok(())
}

#[test]
fn regressing_crypto_epoch_is_rejected_before_writing() -> WalResult<()> {
    let path = temp_path("epoch-preflight")?;
    let mut writer = WalFileWriter::open_append(&path, wal_domain()?, WalAppendOptions::default())?;
    let _outcome = writer.append_transaction_once(&header(10, &[1])?, &[1])?;
    let batch = header(11, &[1])?;
    let stale = WalFrameHeader::new(WalFrameHeaderInput {
        record_kind: batch.record_kind(),
        tenant_id: batch.tenant_id(),
        tx_id: batch.tx_id(),
        encryption_key_id: batch.encryption_key_id(),
        crypto_epoch: skrifheim_crypto::CryptoEpoch::new(batch.crypto_epoch().get() - 1),
        encryption_domain: batch.encryption_domain(),
        encrypted_body_len: 1,
        body_crc64: skrifheim_storage::BodyChecksum::Present(batch.body_crc64()),
    })?;
    let before = fs::read(&path)?;
    assert!(writer.append_transaction_once(&stale, &[1]).is_err());
    assert_eq!(fs::read(&path)?, before);
    assert_eq!(
        writer.append_transaction_once(&batch, &[1])?,
        WalTransactionOutcome::Durable
    );
    drop(writer);
    fs::remove_file(path)?;
    Ok(())
}
