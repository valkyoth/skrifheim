use alloc::vec::Vec;
use skrifheim::storage::*;

/// Small reference automaton: bounded synthetic identities, no storage internals.
pub fn exercise(data: &[u8]) {
    let Some(control) = data.get(..2) else {
        return;
    };
    let limit = usize::from(control[0] % 17);
    let stop = if control[1] & 1 == 0 {
        WalReplayStop::CleanEof
    } else {
        WalReplayStop::TruncatedFrame
    };
    let mut replay = WalReplay::new_with_transaction_limit(limit);
    let mut validator = WalReplayValidator::new_with_transaction_limit(limit);
    let mut active: Option<([u8; 4], u32)> = None;
    let mut last = 0;
    let mut epoch = 0;
    let mut closed = 0;
    let mut frames = 0;
    let mut checkpoints = 0;
    let mut commits = Vec::new();
    let mut rollbacks = Vec::new();
    for event in data[2..].as_chunks::<5>().0.iter().take(128) {
        let kind = event[0] % 5;
        let identity = [
            event[1].max(1),
            event[2].max(1),
            event[3].max(1),
            event[4].max(1),
        ];
        let mut wire = crate::seeds::wal();
        wire[10] = kind + 1;
        wire[64] = identity[0];
        wire[80] = identity[1];
        wire[88] = identity[2];
        wire[16] = identity[3];
        let parsed = WalFrameHeader::parse(&wire);
        assert!(parsed.is_ok());
        let Ok(header) = parsed else {
            return;
        };
        let matching = active.is_some_and(|(id, _)| id == identity);
        let allowed = match kind {
            0 => active.is_none() && identity[0] > last && identity[1] >= epoch,
            1 => matching,
            2 | 3 => matching && closed < limit,
            _ => active.is_none(),
        };
        assert_eq!(replay.process_header(&header).is_ok(), allowed);
        assert_eq!(validator.process_header(&header).is_ok(), allowed);
        // Continue after rejected transitions to check unchanged state.
        if !allowed {
            continue;
        }
        frames += 1;
        match kind {
            0 => {
                active = Some((identity, 0));
                epoch = identity[1];
            }
            1 => {
                if let Some((_, batches)) = &mut active {
                    *batches += 1;
                }
            }
            2 | 3 => {
                if let Some((id, batches)) = active.take() {
                    if kind == 2 {
                        commits.push((id[0], batches));
                    } else {
                        rollbacks.push((id[0], batches, WalRollbackReason::AbortRecord));
                    }
                }
                closed += 1;
                last = identity[0];
            }
            _ => checkpoints += 1,
        }
    }
    let allowed = stop == WalReplayStop::CleanEof && (active.is_none() || closed < limit);
    let result = replay.finish(stop);
    let validation = validator.finish(stop);
    assert_eq!(result.is_ok(), allowed);
    assert_eq!(validation.is_ok(), allowed);
    if let Ok(report) = result {
        assert_eq!(validation, Ok(report.outcome()));
        assert_eq!(report.replayed_frame_count(), frames);
        assert_eq!(report.checkpoint_count(), checkpoints);
        assert_eq!(
            report.outcome(),
            if active.is_some() {
                WalRecoveryOutcome::RecoveredUncommittedTail
            } else {
                WalRecoveryOutcome::Clean
            }
        );
        if let Some((id, batches)) = active {
            rollbacks.push((id[0], batches, WalRollbackReason::UncommittedTail));
        }
        assert_eq!(report.committed_transactions().len(), commits.len());
        assert_eq!(report.rolled_back_transactions().len(), rollbacks.len());
        for (actual, (tx, batches)) in report.committed_transactions().iter().zip(commits) {
            assert_eq!(actual.tx_id().get(), u128::from(tx));
            assert_eq!(actual.fact_batch_count(), batches);
            assert_eq!(actual.frame_count(), batches + 2);
        }
        for (actual, (tx, batches, reason)) in
            report.rolled_back_transactions().iter().zip(rollbacks)
        {
            assert_eq!(actual.tx_id().get(), u128::from(tx));
            assert_eq!(actual.fact_batch_count(), batches);
            assert_eq!(actual.frame_count(), batches + 1);
            assert_eq!(actual.reason(), reason);
        }
    }
}
