use super::{Result, WalFileError};
use skrifheim_core::TxId;
use skrifheim_storage::{WAL_FRAME_HEADER_BYTES, WalFrameHeader};
use std::{
    fs::File,
    io::{self, Write},
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DurabilityMode {
    /// The caller must not acknowledge a durable transaction from this result.
    Buffered,
    SyncAll,
}

/// Local WAL-v1 byte range, not a globally durable LSN or commit proof.
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct WalReceipt {
    start: u64,
    end: u64,
    tx_id: TxId,
}

impl WalReceipt {
    pub(super) fn encode_evidence(self, outcome: u8) -> [u8; 34] {
        let mut bytes = [0; 34];
        bytes[0] = 1;
        bytes[1] = outcome;
        bytes[2..10].copy_from_slice(&self.start.to_le_bytes());
        bytes[10..18].copy_from_slice(&self.end.to_le_bytes());
        bytes[18..34].copy_from_slice(&self.tx_id.get().to_le_bytes());
        bytes
    }
    pub const fn start(self) -> u64 {
        self.start
    }
    pub const fn end(self) -> u64 {
        self.end
    }
    pub const fn tx_id(self) -> TxId {
        self.tx_id
    }
}

impl std::fmt::Debug for WalReceipt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("WalReceipt(<redacted>)")
    }
}

#[must_use = "inspect durability before acknowledging the write"]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WalAppendOutcome {
    Buffered(WalReceipt),
    Durable(WalReceipt),
}

impl WalAppendOutcome {
    /// Versioned local diagnostic evidence; not a signed commit or recovery input.
    pub fn encode_evidence(self) -> [u8; 34] {
        match self {
            Self::Buffered(receipt) => receipt.encode_evidence(0),
            Self::Durable(receipt) => receipt.encode_evidence(1),
        }
    }
}

pub(super) trait WalSink: Write {
    fn sync_all(&mut self) -> io::Result<()>;
}
impl WalSink for File {
    fn sync_all(&mut self) -> io::Result<()> {
        File::sync_all(self)
    }
}

pub(super) struct AppendState {
    next_offset: u64,
    poisoned: bool,
}

impl AppendState {
    pub(super) fn poison(&mut self) {
        self.poisoned = true;
    }
    pub(super) fn new(next_offset: u64) -> Self {
        Self {
            next_offset,
            poisoned: false,
        }
    }
    pub(super) fn is_poisoned(&self) -> bool {
        self.poisoned
    }

    pub(super) fn append(
        &mut self,
        sink: &mut impl WalSink,
        mode: DurabilityMode,
        header: &WalFrameHeader,
        body: &[u8],
    ) -> Result<WalAppendOutcome> {
        if self.poisoned {
            return Err(WalFileError::Poisoned);
        }
        let end = self
            .next_offset
            .checked_add(WAL_FRAME_HEADER_BYTES as u64)
            .and_then(|n| n.checked_add(body.len() as u64))
            .ok_or_else(|| WalFileError::Io(io::Error::other("WAL offset exhausted")))?;
        let receipt = WalReceipt {
            start: self.next_offset,
            end,
            tx_id: header.tx_id(),
        };
        // Remain poisoned after every partial write, flush, sync error or unwind.
        self.poisoned = true;
        let written = sink
            .write_all(&header.encode())
            .and_then(|()| sink.write_all(body))
            .and_then(|()| sink.flush())
            .and_then(|()| match mode {
                DurabilityMode::Buffered => Ok(()),
                DurabilityMode::SyncAll => sink.sync_all(),
            });
        if written.is_err() {
            return Err(WalFileError::Ambiguous { receipt });
        }
        self.next_offset = end;
        self.poisoned = false;
        Ok(match mode {
            DurabilityMode::Buffered => WalAppendOutcome::Buffered(receipt),
            DurabilityMode::SyncAll => WalAppendOutcome::Durable(receipt),
        })
    }
}

#[cfg(test)]
mod tests;
