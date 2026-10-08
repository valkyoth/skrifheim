use super::super::tests::header;
use super::*;

#[test]
fn rejected_overflows_leave_transition_state_unchanged() -> Result<()> {
    let begin = header(WalRecordKind::TransactionBegin, 1)?;
    let commit = header(WalRecordKind::TransactionCommit, 1)?;
    let batch = header(WalRecordKind::FactBatch, 1)?;
    let checkpoint = header(WalRecordKind::Checkpoint, 1)?;
    let mut state = ReplayState::new(usize::MAX);
    state.replayed_frame_count = u64::MAX;
    assert!(state.process_header(&begin).is_err());
    assert!(state.active.is_none());
    assert_eq!(state.replayed_frame_count, u64::MAX);

    state.replayed_frame_count = 0;
    state.checkpoint_count = u64::MAX;
    assert!(state.process_header(&checkpoint).is_err());
    assert_eq!(state.replayed_frame_count, 0);
    assert_eq!(state.checkpoint_count, u64::MAX);

    state.process_header(&begin)?;
    state.closed_transactions = usize::MAX;
    assert!(state.process_header(&commit).is_err());
    assert!(state.active.is_some());
    assert_eq!(state.last_closed_tx, None);
    assert_eq!(state.closed_transactions, usize::MAX);
    assert!(state.finish(WalReplayStop::CleanEof).is_err());

    state.closed_transactions = 0;
    let active = state
        .active
        .as_mut()
        .ok_or_else(|| invalid_replay("missing test transaction"))?;
    active.fact_batch_count = u32::MAX;
    assert!(state.process_header(&batch).is_err());
    let active = state
        .active
        .as_ref()
        .ok_or_else(|| invalid_replay("missing test transaction"))?;
    assert_eq!(active.frame_count, 1);
    assert_eq!(active.fact_batch_count, u32::MAX);
    assert_eq!(state.replayed_frame_count, 1);
    Ok(())
}
