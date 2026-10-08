use alloc::vec::Vec;
use core::fmt;

use skrifheim_core::{Result, SkrifheimError, TxId};
use skrifheim_crypto::{CryptoEpoch, EncryptionDomain, KeyId};

use super::{WalFrameHeader, WalRecordKind};
mod state;
use state::{ReplayEvent, ReplayState};

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
    state: ReplayState,
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
            .field("has_active_transaction", &self.state.active.is_some())
            .field("last_closed_tx", &"<redacted>")
            .field("replayed_frame_count", &self.state.replayed_frame_count)
            .field("checkpoint_count", &self.state.checkpoint_count)
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
            state: ReplayState::new(transaction_limit),
            committed_transactions: Vec::new(),
            rolled_back_transactions: Vec::new(),
        }
    }

    pub fn process_header(&mut self, header: &WalFrameHeader) -> Result<()> {
        let mut next = self.state;
        match next.process_header(header)? {
            ReplayEvent::None => {}
            ReplayEvent::Committed(transaction) => {
                self.committed_transactions
                    .try_reserve(1)
                    .map_err(|_| invalid_replay("WAL report allocation failed"))?;
                self.committed_transactions.push(transaction);
            }
            ReplayEvent::RolledBack(transaction) => {
                self.rolled_back_transactions
                    .try_reserve(1)
                    .map_err(|_| invalid_replay("WAL report allocation failed"))?;
                self.rolled_back_transactions.push(transaction);
            }
        }
        self.state = next;
        Ok(())
    }

    pub fn finish(mut self, stop: WalReplayStop) -> Result<WalRecoveryReport> {
        let (state, outcome, tail) = self.state.finish(stop)?;
        if let Some(tail) = tail {
            self.rolled_back_transactions
                .try_reserve(1)
                .map_err(|_| invalid_replay("WAL report allocation failed"))?;
            self.rolled_back_transactions.push(tail);
        }
        Ok(WalRecoveryReport {
            outcome,
            replayed_frame_count: state.replayed_frame_count,
            checkpoint_count: state.checkpoint_count,
            committed_transactions: self.committed_transactions,
            rolled_back_transactions: self.rolled_back_transactions,
        })
    }
}

/// Fixed-memory validation using the same transition engine as report replay.
/// No per-transaction summary is retained. Call finish after any candidate
/// preflight to validate clean EOF and count an uncommitted tail against limits.
pub struct WalReplayValidator {
    state: ReplayState,
}

impl Default for WalReplayValidator {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for WalReplayValidator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("WalReplayValidator(<redacted>)")
    }
}

impl WalReplayValidator {
    pub const fn new() -> Self {
        Self::new_with_transaction_limit(WAL_REPLAY_MAX_TRANSACTIONS)
    }

    pub const fn new_with_transaction_limit(transaction_limit: usize) -> Self {
        Self {
            state: ReplayState::new(transaction_limit),
        }
    }

    pub fn process_header(&mut self, header: &WalFrameHeader) -> Result<()> {
        self.state.process_header(header).map(|_| ())
    }

    pub fn finish(self, stop: WalReplayStop) -> Result<WalRecoveryOutcome> {
        self.state.finish(stop).map(|(_, outcome, _)| outcome)
    }
}
fn invalid_replay(reason: &'static str) -> SkrifheimError {
    SkrifheimError::InvalidWalFrame(reason.into())
}

#[cfg(test)]
mod tests;
