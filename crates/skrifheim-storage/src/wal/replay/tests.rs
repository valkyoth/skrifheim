use alloc::format;

use skrifheim_core::{Result, SkrifheimError, TenantId, TxId, WorldId};
use skrifheim_crypto::{CryptoEpoch, EncryptionDomain, KeyId, RegionKeyId};

use super::*;
use crate::{BodyChecksum, WalFrameHeaderInput};

fn id<T>(id: Option<T>) -> Result<T> {
    id.ok_or(SkrifheimError::InvalidIdentifier)
}

fn tenant() -> Result<TenantId> {
    id(TenantId::from_u128(1))
}

fn domain() -> Result<EncryptionDomain> {
    Ok(EncryptionDomain::wal(
        tenant()?,
        Some(id(RegionKeyId::from_u128(2))?),
        Some(id(WorldId::from_u128(3))?),
    ))
}

fn header(kind: WalRecordKind, tx: u128) -> Result<WalFrameHeader> {
    header_with_epoch(kind, tx, 5)
}

fn header_with_epoch(kind: WalRecordKind, tx: u128, epoch: u64) -> Result<WalFrameHeader> {
    WalFrameHeader::new(WalFrameHeaderInput {
        record_kind: kind,
        tenant_id: tenant()?,
        tx_id: id(TxId::from_u128(tx))?,
        encryption_key_id: id(KeyId::from_u128(4))?,
        crypto_epoch: CryptoEpoch::new(epoch),
        encryption_domain: domain()?,
        encrypted_body_len: 1,
        body_crc64: BodyChecksum::Present(9),
    })
}

#[test]
fn replay_reports_committed_transactions() -> Result<()> {
    let mut replay = WalReplay::new();
    replay.process_header(&header(WalRecordKind::TransactionBegin, 10)?)?;
    replay.process_header(&header(WalRecordKind::FactBatch, 10)?)?;
    replay.process_header(&header(WalRecordKind::FactBatch, 10)?)?;
    replay.process_header(&header(WalRecordKind::TransactionCommit, 10)?)?;
    replay.process_header(&header(WalRecordKind::Checkpoint, 10)?)?;

    let report = replay.finish(WalReplayStop::CleanEof)?;

    assert_eq!(report.outcome(), WalRecoveryOutcome::Clean);
    assert_eq!(report.replayed_frame_count(), 5);
    assert_eq!(report.checkpoint_count(), 1);
    assert_eq!(report.committed_transactions().len(), 1);
    assert_eq!(report.committed_transactions()[0].tx_id().get(), 10);
    assert_eq!(report.committed_transactions()[0].frame_count(), 4);
    assert_eq!(report.committed_transactions()[0].fact_batch_count(), 2);
    assert!(report.rolled_back_transactions().is_empty());
    Ok(())
}

#[test]
fn replay_rolls_back_abort_records() -> Result<()> {
    let mut replay = WalReplay::new();
    replay.process_header(&header(WalRecordKind::TransactionBegin, 11)?)?;
    replay.process_header(&header(WalRecordKind::FactBatch, 11)?)?;
    replay.process_header(&header(WalRecordKind::TransactionAbort, 11)?)?;

    let report = replay.finish(WalReplayStop::CleanEof)?;

    assert_eq!(report.outcome(), WalRecoveryOutcome::Clean);
    assert_eq!(report.rolled_back_transactions().len(), 1);
    assert_eq!(
        report.rolled_back_transactions()[0].reason(),
        WalRollbackReason::AbortRecord
    );
    assert!(report.committed_transactions().is_empty());
    Ok(())
}

#[test]
fn replay_recovers_uncommitted_tail_on_clean_eof() -> Result<()> {
    let mut replay = WalReplay::new();
    replay.process_header(&header(WalRecordKind::TransactionBegin, 12)?)?;
    replay.process_header(&header(WalRecordKind::FactBatch, 12)?)?;

    let report = replay.finish(WalReplayStop::CleanEof)?;

    assert_eq!(
        report.outcome(),
        WalRecoveryOutcome::RecoveredUncommittedTail
    );
    assert_eq!(report.rolled_back_transactions().len(), 1);
    assert_eq!(
        report.rolled_back_transactions()[0].reason(),
        WalRollbackReason::UncommittedTail
    );
    Ok(())
}

#[test]
fn replay_rejects_truncated_frame_stop() -> Result<()> {
    let replay = WalReplay::new();

    assert!(matches!(
        replay.finish(WalReplayStop::TruncatedFrame),
        Err(SkrifheimError::InvalidWalFrame(_))
    ));
    Ok(())
}

#[test]
fn replay_rejects_transaction_shape_errors() -> Result<()> {
    let mut replay = WalReplay::new();
    assert!(matches!(
        replay.process_header(&header(WalRecordKind::FactBatch, 13)?),
        Err(SkrifheimError::InvalidWalFrame(_))
    ));

    let mut replay = WalReplay::new();
    replay.process_header(&header(WalRecordKind::TransactionBegin, 14)?)?;
    assert!(matches!(
        replay.process_header(&header(WalRecordKind::TransactionBegin, 15)?),
        Err(SkrifheimError::InvalidWalFrame(_))
    ));

    let mut replay = WalReplay::new();
    replay.process_header(&header(WalRecordKind::TransactionBegin, 16)?)?;
    assert!(matches!(
        replay.process_header(&header(WalRecordKind::Checkpoint, 16)?),
        Err(SkrifheimError::InvalidWalFrame(_))
    ));
    Ok(())
}

#[test]
fn replay_rejects_key_epoch_or_domain_mismatch() -> Result<()> {
    let mut replay = WalReplay::new();
    replay.process_header(&header_with_epoch(WalRecordKind::TransactionBegin, 17, 5)?)?;
    assert!(matches!(
        replay.process_header(&header_with_epoch(WalRecordKind::FactBatch, 17, 6)?),
        Err(SkrifheimError::InvalidWalFrame(_))
    ));

    let mut replay = WalReplay::new();
    replay.process_header(&header(WalRecordKind::TransactionBegin, 18)?)?;
    assert!(matches!(
        replay.process_header(&header(WalRecordKind::TransactionCommit, 19)?),
        Err(SkrifheimError::InvalidWalFrame(_))
    ));
    Ok(())
}

#[test]
fn replay_rejected_close_keeps_active_transaction() -> Result<()> {
    let mut replay = WalReplay::new();
    replay.process_header(&header(WalRecordKind::TransactionBegin, 26)?)?;
    replay.process_header(&header(WalRecordKind::FactBatch, 26)?)?;

    assert!(matches!(
        replay.process_header(&header(WalRecordKind::TransactionCommit, 27)?),
        Err(SkrifheimError::InvalidWalFrame(_))
    ));
    replay.process_header(&header(WalRecordKind::TransactionCommit, 26)?)?;
    let report = replay.finish(WalReplayStop::CleanEof)?;

    assert_eq!(report.replayed_frame_count(), 3);
    assert_eq!(report.committed_transactions().len(), 1);
    assert_eq!(report.committed_transactions()[0].tx_id().get(), 26);
    assert!(report.rolled_back_transactions().is_empty());
    Ok(())
}

#[test]
fn replay_rejected_frame_does_not_advance_frame_count() -> Result<()> {
    let mut replay = WalReplay::new();

    assert!(matches!(
        replay.process_header(&header(WalRecordKind::FactBatch, 28)?),
        Err(SkrifheimError::InvalidWalFrame(_))
    ));
    let report = replay.finish(WalReplayStop::CleanEof)?;

    assert_eq!(report.replayed_frame_count(), 0);
    Ok(())
}

#[test]
fn replay_rejects_non_advancing_transaction_ids() -> Result<()> {
    let mut replay = WalReplay::new();
    replay.process_header(&header(WalRecordKind::TransactionBegin, 20)?)?;
    replay.process_header(&header(WalRecordKind::TransactionCommit, 20)?)?;
    assert!(matches!(
        replay.process_header(&header(WalRecordKind::TransactionBegin, 20)?),
        Err(SkrifheimError::InvalidWalFrame(_))
    ));
    Ok(())
}

#[test]
fn replay_rejects_crypto_epoch_regression_between_transactions() -> Result<()> {
    let mut replay = WalReplay::new();
    replay.process_header(&header_with_epoch(WalRecordKind::TransactionBegin, 24, 5)?)?;
    replay.process_header(&header_with_epoch(WalRecordKind::TransactionCommit, 24, 5)?)?;

    assert!(matches!(
        replay.process_header(&header_with_epoch(WalRecordKind::TransactionBegin, 25, 4)?),
        Err(SkrifheimError::InvalidWalFrame(_))
    ));
    Ok(())
}

#[test]
fn replay_rejects_transaction_summary_limit() -> Result<()> {
    let mut replay = WalReplay::new_with_transaction_limit(1);
    replay.process_header(&header(WalRecordKind::TransactionBegin, 22)?)?;
    replay.process_header(&header(WalRecordKind::TransactionCommit, 22)?)?;
    replay.process_header(&header(WalRecordKind::TransactionBegin, 23)?)?;

    assert!(matches!(
        replay.process_header(&header(WalRecordKind::TransactionCommit, 23)?),
        Err(SkrifheimError::InvalidWalFrame(_))
    ));
    Ok(())
}

#[test]
fn replay_debug_redacts_transaction_metadata() -> Result<()> {
    let mut replay = WalReplay::new();
    replay.process_header(&header(WalRecordKind::TransactionBegin, 21)?)?;
    let debug = format!("{replay:?}");

    assert!(debug.contains("has_active_transaction"));
    assert!(debug.contains("last_closed_tx: \"<redacted>\""));
    assert!(!debug.contains("TxId"));
    assert!(!debug.contains("21"));
    Ok(())
}
