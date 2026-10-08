use super::*;

#[cfg(test)]
mod tests;

#[derive(Clone, Copy)]
pub(super) struct ReplayState {
    pub(super) active: Option<ActiveTransaction>,
    last_closed_tx: Option<TxId>,
    max_observed_epoch: Option<CryptoEpoch>,
    pub(super) replayed_frame_count: u64,
    pub(super) checkpoint_count: u64,
    transaction_limit: usize,
    closed_transactions: usize,
}

pub(super) enum ReplayEvent {
    None,
    Committed(WalRecoveredTransaction),
    RolledBack(WalRolledBackTransaction),
}

impl ReplayState {
    pub(super) const fn new(transaction_limit: usize) -> Self {
        Self {
            active: None,
            last_closed_tx: None,
            max_observed_epoch: None,
            replayed_frame_count: 0,
            checkpoint_count: 0,
            transaction_limit,
            closed_transactions: 0,
        }
    }

    pub(super) fn process_header(&mut self, header: &WalFrameHeader) -> Result<ReplayEvent> {
        // Failed validation must leave the shared transition state unchanged.
        let mut next = *self;
        let event = next.process_inner(header)?;
        *self = next;
        Ok(event)
    }

    fn process_inner(&mut self, header: &WalFrameHeader) -> Result<ReplayEvent> {
        header.validate()?;
        let next_frames = self
            .replayed_frame_count
            .checked_add(1)
            .ok_or_else(|| invalid_replay("WAL replay frame count overflow"))?;
        let event = match header.record_kind() {
            WalRecordKind::TransactionBegin => {
                if self.active.is_some() {
                    return Err(invalid_replay("WAL transaction began before prior close"));
                }
                if self
                    .last_closed_tx
                    .is_some_and(|last| header.tx_id().get() <= last.get())
                {
                    return Err(invalid_replay("WAL transaction identifier did not advance"));
                }
                if self
                    .max_observed_epoch
                    .is_some_and(|epoch| header.crypto_epoch().get() < epoch.get())
                {
                    return Err(invalid_replay("WAL crypto epoch regressed"));
                }
                self.max_observed_epoch = Some(header.crypto_epoch());
                self.active = Some(ActiveTransaction::new(header));
                ReplayEvent::None
            }
            WalRecordKind::FactBatch => {
                let active = self
                    .active
                    .as_mut()
                    .ok_or_else(|| invalid_replay("WAL fact batch is outside transaction"))?;
                active.require_matching_header(header)?;
                active.record_fact_batch()?;
                ReplayEvent::None
            }
            WalRecordKind::TransactionCommit | WalRecordKind::TransactionAbort => {
                let active = self
                    .active
                    .as_ref()
                    .ok_or_else(|| invalid_replay("WAL close is outside transaction"))?;
                active.require_matching_header(header)?;
                let event = if header.record_kind() == WalRecordKind::TransactionCommit {
                    ReplayEvent::Committed(active.to_commit()?)
                } else {
                    ReplayEvent::RolledBack(active.to_rollback(WalRollbackReason::AbortRecord))
                };
                self.last_closed_tx = Some(active.tx_id);
                self.count_close()?;
                self.active = None;
                event
            }
            WalRecordKind::Checkpoint => {
                if self.active.is_some() {
                    return Err(invalid_replay("WAL checkpoint is inside transaction"));
                }
                self.checkpoint_count = self
                    .checkpoint_count
                    .checked_add(1)
                    .ok_or_else(|| invalid_replay("WAL checkpoint count overflow"))?;
                ReplayEvent::None
            }
        };
        self.replayed_frame_count = next_frames;
        Ok(event)
    }

    pub(super) fn finish(
        mut self,
        stop: WalReplayStop,
    ) -> Result<(Self, WalRecoveryOutcome, Option<WalRolledBackTransaction>)> {
        if stop != WalReplayStop::CleanEof {
            return Err(invalid_replay(
                "WAL replay stopped inside a truncated frame",
            ));
        }
        let tail = if let Some(active) = self.active.take() {
            self.count_close()?;
            Some(active.into_rollback(WalRollbackReason::UncommittedTail))
        } else {
            None
        };
        let outcome = if tail.is_some() {
            WalRecoveryOutcome::RecoveredUncommittedTail
        } else {
            WalRecoveryOutcome::Clean
        };
        Ok((self, outcome, tail))
    }

    fn count_close(&mut self) -> Result<()> {
        let count = self
            .closed_transactions
            .checked_add(1)
            .ok_or_else(|| invalid_replay("WAL replay transaction count overflow"))?;
        if count > self.transaction_limit {
            return Err(invalid_replay("WAL replay transaction limit exceeded"));
        }
        self.closed_transactions = count;
        Ok(())
    }
}

#[derive(Clone, Copy)]
pub(super) struct ActiveTransaction {
    tx_id: TxId,
    frame_count: u32,
    fact_batch_count: u32,
    encryption_key_id: KeyId,
    crypto_epoch: CryptoEpoch,
    encryption_domain: EncryptionDomain,
}

impl ActiveTransaction {
    const fn new(header: &WalFrameHeader) -> Self {
        Self {
            tx_id: header.tx_id(),
            frame_count: 1,
            fact_batch_count: 0,
            encryption_key_id: header.encryption_key_id(),
            crypto_epoch: header.crypto_epoch(),
            encryption_domain: header.encryption_domain(),
        }
    }

    fn require_matching_header(&self, header: &WalFrameHeader) -> Result<()> {
        if header.tx_id() != self.tx_id {
            return Err(invalid_replay("WAL transaction identifier mismatch"));
        }
        if header.encryption_key_id() != self.encryption_key_id {
            return Err(invalid_replay("WAL transaction key mismatch"));
        }
        if header.crypto_epoch() != self.crypto_epoch {
            return Err(invalid_replay("WAL transaction crypto epoch mismatch"));
        }
        if !header
            .encryption_domain()
            .structurally_equal_ct(&self.encryption_domain)
        {
            return Err(invalid_replay("WAL transaction encryption domain mismatch"));
        }
        Ok(())
    }

    fn record_fact_batch(&mut self) -> Result<()> {
        self.frame_count = self
            .frame_count
            .checked_add(1)
            .ok_or_else(|| invalid_replay("WAL transaction frame count overflow"))?;
        self.fact_batch_count = self
            .fact_batch_count
            .checked_add(1)
            .ok_or_else(|| invalid_replay("WAL fact batch count overflow"))?;
        Ok(())
    }

    fn to_commit(self) -> Result<WalRecoveredTransaction> {
        let frame_count = self
            .frame_count
            .checked_add(1)
            .ok_or_else(|| invalid_replay("WAL transaction frame count overflow"))?;
        Ok(WalRecoveredTransaction {
            tx_id: self.tx_id,
            frame_count,
            fact_batch_count: self.fact_batch_count,
            encryption_key_id: self.encryption_key_id,
            crypto_epoch: self.crypto_epoch,
            encryption_domain: self.encryption_domain,
        })
    }

    fn to_rollback(self, reason: WalRollbackReason) -> WalRolledBackTransaction {
        WalRolledBackTransaction {
            tx_id: self.tx_id,
            reason,
            frame_count: self.frame_count,
            fact_batch_count: self.fact_batch_count,
        }
    }

    fn into_rollback(self, reason: WalRollbackReason) -> WalRolledBackTransaction {
        self.to_rollback(reason)
    }
}
