use alloc::vec::Vec;
use core::fmt;

use skrifheim_core::{Result, SkrifheimError, TxId};
use skrifheim_crypto::{CryptoEpoch, EncryptionDomain, KeyId};

use super::{WalFrameHeader, WalRecordKind};

pub const WAL_REPLAY_MAX_TRANSACTIONS: usize = 1_000_000;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WalReplayStop {
    CleanEof,
    TruncatedFrame,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WalRecoveryOutcome {
    Clean,
    RecoveredUncommittedTail,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WalRollbackReason {
    AbortRecord,
    UncommittedTail,
}

#[derive(Clone)]
pub struct WalRecoveredTransaction {
    tx_id: TxId,
    frame_count: u32,
    fact_batch_count: u32,
    encryption_key_id: KeyId,
    crypto_epoch: CryptoEpoch,
    encryption_domain: EncryptionDomain,
}

impl fmt::Debug for WalRecoveredTransaction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WalRecoveredTransaction")
            .field("tx_id", &"<redacted>")
            .field("frame_count", &self.frame_count)
            .field("fact_batch_count", &self.fact_batch_count)
            .field("encryption_key_id", &"<redacted>")
            .field("crypto_epoch", &"<redacted>")
            .field("encryption_domain", &"<redacted>")
            .finish()
    }
}

impl WalRecoveredTransaction {
    #[must_use]
    pub const fn tx_id(&self) -> TxId {
        self.tx_id
    }

    #[must_use]
    pub const fn frame_count(&self) -> u32 {
        self.frame_count
    }

    #[must_use]
    pub const fn fact_batch_count(&self) -> u32 {
        self.fact_batch_count
    }

    #[must_use]
    pub const fn encryption_key_id(&self) -> KeyId {
        self.encryption_key_id
    }

    #[must_use]
    pub const fn crypto_epoch(&self) -> CryptoEpoch {
        self.crypto_epoch
    }

    #[must_use]
    pub const fn encryption_domain(&self) -> EncryptionDomain {
        self.encryption_domain
    }
}

#[derive(Clone)]
pub struct WalRolledBackTransaction {
    tx_id: TxId,
    reason: WalRollbackReason,
    frame_count: u32,
    fact_batch_count: u32,
}

impl fmt::Debug for WalRolledBackTransaction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WalRolledBackTransaction")
            .field("tx_id", &"<redacted>")
            .field("reason", &self.reason)
            .field("frame_count", &self.frame_count)
            .field("fact_batch_count", &self.fact_batch_count)
            .finish()
    }
}

impl WalRolledBackTransaction {
    #[must_use]
    pub const fn tx_id(&self) -> TxId {
        self.tx_id
    }

    #[must_use]
    pub const fn reason(&self) -> WalRollbackReason {
        self.reason
    }

    #[must_use]
    pub const fn frame_count(&self) -> u32 {
        self.frame_count
    }

    #[must_use]
    pub const fn fact_batch_count(&self) -> u32 {
        self.fact_batch_count
    }
}

pub struct WalRecoveryReport {
    outcome: WalRecoveryOutcome,
    replayed_frame_count: u64,
    checkpoint_count: u64,
    committed_transactions: Vec<WalRecoveredTransaction>,
    rolled_back_transactions: Vec<WalRolledBackTransaction>,
}

impl fmt::Debug for WalRecoveryReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WalRecoveryReport")
            .field("outcome", &self.outcome)
            .field("replayed_frame_count", &self.replayed_frame_count)
            .field("checkpoint_count", &self.checkpoint_count)
            .field("committed_count", &self.committed_transactions.len())
            .field("rolled_back_count", &self.rolled_back_transactions.len())
            .finish()
    }
}

impl WalRecoveryReport {
    #[must_use]
    pub const fn outcome(&self) -> WalRecoveryOutcome {
        self.outcome
    }

    #[must_use]
    pub const fn replayed_frame_count(&self) -> u64 {
        self.replayed_frame_count
    }

    #[must_use]
    pub const fn checkpoint_count(&self) -> u64 {
        self.checkpoint_count
    }

    #[must_use]
    pub fn committed_transactions(&self) -> &[WalRecoveredTransaction] {
        &self.committed_transactions
    }

    #[must_use]
    pub fn rolled_back_transactions(&self) -> &[WalRolledBackTransaction] {
        &self.rolled_back_transactions
    }
}

pub struct WalReplay {
    active: Option<ActiveTransaction>,
    last_closed_tx: Option<TxId>,
    max_observed_epoch: Option<CryptoEpoch>,
    replayed_frame_count: u64,
    checkpoint_count: u64,
    transaction_limit: usize,
    committed_transactions: Vec<WalRecoveredTransaction>,
    rolled_back_transactions: Vec<WalRolledBackTransaction>,
}

impl Default for WalReplay {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for WalReplay {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WalReplay")
            .field("has_active_transaction", &self.active.is_some())
            .field("last_closed_tx", &"<redacted>")
            .field("replayed_frame_count", &self.replayed_frame_count)
            .field("checkpoint_count", &self.checkpoint_count)
            .field("committed_count", &self.committed_transactions.len())
            .field("rolled_back_count", &self.rolled_back_transactions.len())
            .finish()
    }
}

impl WalReplay {
    #[must_use]
    pub const fn new() -> Self {
        Self::new_with_transaction_limit(WAL_REPLAY_MAX_TRANSACTIONS)
    }

    #[must_use]
    pub const fn new_with_transaction_limit(transaction_limit: usize) -> Self {
        Self {
            active: None,
            last_closed_tx: None,
            max_observed_epoch: None,
            replayed_frame_count: 0,
            checkpoint_count: 0,
            transaction_limit,
            committed_transactions: Vec::new(),
            rolled_back_transactions: Vec::new(),
        }
    }

    pub fn process_header(&mut self, header: &WalFrameHeader) -> Result<()> {
        header.validate()?;
        self.replayed_frame_count = self
            .replayed_frame_count
            .checked_add(1)
            .ok_or_else(|| invalid_replay("WAL replay frame count overflow"))?;
        match header.record_kind() {
            WalRecordKind::TransactionBegin => self.begin_transaction(header),
            WalRecordKind::FactBatch => self.record_fact_batch(header),
            WalRecordKind::TransactionCommit => self.commit_transaction(header),
            WalRecordKind::TransactionAbort => self.abort_transaction(header),
            WalRecordKind::Checkpoint => self.record_checkpoint(),
        }
    }

    pub fn finish(mut self, stop: WalReplayStop) -> Result<WalRecoveryReport> {
        if stop == WalReplayStop::TruncatedFrame {
            return Err(invalid_replay(
                "WAL replay stopped inside a truncated frame",
            ));
        }
        let mut outcome = WalRecoveryOutcome::Clean;
        if let Some(active) = self.active.take() {
            self.require_transaction_capacity()?;
            outcome = WalRecoveryOutcome::RecoveredUncommittedTail;
            self.rolled_back_transactions
                .push(active.into_rollback(WalRollbackReason::UncommittedTail));
        }
        Ok(WalRecoveryReport {
            outcome,
            replayed_frame_count: self.replayed_frame_count,
            checkpoint_count: self.checkpoint_count,
            committed_transactions: self.committed_transactions,
            rolled_back_transactions: self.rolled_back_transactions,
        })
    }

    fn begin_transaction(&mut self, header: &WalFrameHeader) -> Result<()> {
        if self.active.is_some() {
            return Err(invalid_replay("WAL transaction began before prior close"));
        }
        self.require_advancing_tx(header.tx_id())?;
        self.require_non_regressing_epoch(header.crypto_epoch())?;
        self.max_observed_epoch = Some(header.crypto_epoch());
        self.active = Some(ActiveTransaction::new(header));
        Ok(())
    }

    fn record_fact_batch(&mut self, header: &WalFrameHeader) -> Result<()> {
        let active = self
            .active
            .as_mut()
            .ok_or_else(|| invalid_replay("WAL fact batch is outside transaction"))?;
        active.require_matching_header(header)?;
        active.record_fact_batch()
    }

    fn commit_transaction(&mut self, header: &WalFrameHeader) -> Result<()> {
        let active = self
            .active
            .take()
            .ok_or_else(|| invalid_replay("WAL commit is outside transaction"))?;
        active.require_matching_header(header)?;
        self.require_transaction_capacity()?;
        self.last_closed_tx = Some(active.tx_id);
        self.committed_transactions.push(active.into_commit()?);
        Ok(())
    }

    fn abort_transaction(&mut self, header: &WalFrameHeader) -> Result<()> {
        let active = self
            .active
            .take()
            .ok_or_else(|| invalid_replay("WAL abort is outside transaction"))?;
        active.require_matching_header(header)?;
        self.require_transaction_capacity()?;
        self.last_closed_tx = Some(active.tx_id);
        self.rolled_back_transactions
            .push(active.into_rollback(WalRollbackReason::AbortRecord));
        Ok(())
    }

    fn record_checkpoint(&mut self) -> Result<()> {
        if self.active.is_some() {
            return Err(invalid_replay("WAL checkpoint is inside transaction"));
        }
        self.checkpoint_count = self
            .checkpoint_count
            .checked_add(1)
            .ok_or_else(|| invalid_replay("WAL checkpoint count overflow"))?;
        Ok(())
    }

    fn require_advancing_tx(&self, tx_id: TxId) -> Result<()> {
        if let Some(last_closed) = self.last_closed_tx
            && tx_id.get() <= last_closed.get()
        {
            return Err(invalid_replay("WAL transaction identifier did not advance"));
        }
        Ok(())
    }

    fn require_non_regressing_epoch(&self, crypto_epoch: CryptoEpoch) -> Result<()> {
        if let Some(max_observed_epoch) = self.max_observed_epoch
            && crypto_epoch.get() < max_observed_epoch.get()
        {
            return Err(invalid_replay("WAL crypto epoch regressed"));
        }
        Ok(())
    }

    fn require_transaction_capacity(&self) -> Result<()> {
        let closed_transactions = self
            .committed_transactions
            .len()
            .checked_add(self.rolled_back_transactions.len())
            .ok_or_else(|| invalid_replay("WAL replay transaction count overflow"))?;
        if closed_transactions >= self.transaction_limit {
            return Err(invalid_replay("WAL replay transaction limit exceeded"));
        }
        Ok(())
    }
}

struct ActiveTransaction {
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

    fn into_commit(mut self) -> Result<WalRecoveredTransaction> {
        self.frame_count = self
            .frame_count
            .checked_add(1)
            .ok_or_else(|| invalid_replay("WAL transaction frame count overflow"))?;
        Ok(WalRecoveredTransaction {
            tx_id: self.tx_id,
            frame_count: self.frame_count,
            fact_batch_count: self.fact_batch_count,
            encryption_key_id: self.encryption_key_id,
            crypto_epoch: self.crypto_epoch,
            encryption_domain: self.encryption_domain,
        })
    }

    fn into_rollback(self, reason: WalRollbackReason) -> WalRolledBackTransaction {
        WalRolledBackTransaction {
            tx_id: self.tx_id,
            reason,
            frame_count: self.frame_count,
            fact_batch_count: self.fact_batch_count,
        }
    }
}

fn invalid_replay(reason: &'static str) -> SkrifheimError {
    SkrifheimError::InvalidWalFrame(reason.into())
}

#[cfg(test)]
mod tests;
