use super::*;

#[test]
fn validator_and_report_match_for_generated_histories_and_eof() -> Result<()> {
    use WalRecordKind::*;
    let headers = [
        header(TransactionBegin, 1)?,
        header(FactBatch, 1)?,
        header(TransactionCommit, 1)?,
        header(TransactionAbort, 1)?,
        header(Checkpoint, 1)?,
        header(TransactionBegin, 2)?,
        header(TransactionCommit, 2)?,
        header_with_epoch(TransactionBegin, 2, 4)?,
    ];
    for limit in [0, 1, 2] {
        for length in 0..=4 {
            for sequence in 0..8_usize.pow(length) {
                for stop in [WalReplayStop::CleanEof, WalReplayStop::TruncatedFrame] {
                    let mut report = WalReplay::new_with_transaction_limit(limit);
                    let mut validator = WalReplayValidator::new_with_transaction_limit(limit);
                    let mut sequence = sequence;
                    for _ in 0..length {
                        let header = &headers[sequence % 8];
                        assert_eq!(
                            report.process_header(header),
                            validator.process_header(header)
                        );
                        sequence /= 8;
                    }
                    assert_eq!(
                        report.finish(stop).map(|r| r.outcome()),
                        validator.finish(stop)
                    );
                }
            }
        }
    }
    Ok(())
}

#[test]
fn eof_counts_incomplete_tail_and_candidate_must_precede_finish() -> Result<()> {
    let mut validator = WalReplayValidator::new_with_transaction_limit(1);
    validator.process_header(&header(WalRecordKind::TransactionBegin, 1)?)?;
    validator.process_header(&header(WalRecordKind::TransactionCommit, 1)?)?;
    validator.process_header(&header(WalRecordKind::TransactionBegin, 2)?)?;
    assert!(validator.finish(WalReplayStop::CleanEof).is_err());

    let mut validator = WalReplayValidator::new_with_transaction_limit(1);
    validator.process_header(&header(WalRecordKind::TransactionBegin, 1)?)?;
    assert!(
        validator
            .process_header(&header(WalRecordKind::TransactionBegin, 2)?)
            .is_err()
    );
    assert_eq!(
        validator.finish(WalReplayStop::CleanEof)?,
        WalRecoveryOutcome::RecoveredUncommittedTail
    );
    Ok(())
}

#[test]
fn validator_retains_fixed_storage_at_the_default_million_transaction_limit() -> Result<()> {
    assert!(!core::mem::needs_drop::<WalReplayValidator>());
    assert!(core::mem::size_of::<WalReplayValidator>() <= 512);
    let mut validator = WalReplayValidator::new();
    for tx in 1..=WAL_REPLAY_MAX_TRANSACTIONS as u128 {
        validator.process_header(&header(WalRecordKind::TransactionBegin, tx)?)?;
        let close = if tx % 2 == 0 {
            WalRecordKind::TransactionAbort
        } else {
            WalRecordKind::TransactionCommit
        };
        validator.process_header(&header(close, tx)?)?;
    }
    let at_limit = WalReplayValidator {
        state: validator.state,
    };
    assert_eq!(
        at_limit.finish(WalReplayStop::CleanEof)?,
        WalRecoveryOutcome::Clean
    );
    let next_tx = WAL_REPLAY_MAX_TRANSACTIONS as u128 + 1;
    validator.process_header(&header(WalRecordKind::TransactionBegin, next_tx)?)?;
    assert!(
        validator
            .process_header(&header(WalRecordKind::TransactionCommit, next_tx)?)
            .is_err()
    );
    assert!(validator.finish(WalReplayStop::CleanEof).is_err());
    Ok(())
}
