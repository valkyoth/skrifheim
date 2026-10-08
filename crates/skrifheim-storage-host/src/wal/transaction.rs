//! Single-batch WAL-v1 idempotency scaffold. Raw frame append remains a low-level
//! recovery API. These records have CRC protection only, not authenticated order.
use super::{
    DurabilityMode, ReadState, Result, WalAppendOutcome, WalFileError, WalFileWriter,
    map_partial_read, read_exact_or_clean_eof,
};
use skrifheim_core::TxId;
use skrifheim_crypto::EncryptionDomain;
use skrifheim_storage::{
    BodyChecksum, WAL_FRAME_HEADER_BYTES, WalFrameHeader, WalFrameHeaderInput, WalRecordKind,
    wal_body_crc64,
};
use std::io::{Read, Seek, SeekFrom};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WalCommitStatus {
    Absent,
    Incomplete,
    Committed,
}

#[must_use = "check whether the transaction was appended or already present"]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WalTransactionOutcome {
    Durable,
    AlreadyDurable,
}

impl WalFileWriter {
    /// The caller retains the same (domain, TxId) as its retry/idempotency key.
    /// This checks the locked local WAL only; absence is not rollback protection.
    pub fn transaction_status(
        &mut self,
        domain: EncryptionDomain,
        tx_id: TxId,
    ) -> Result<WalCommitStatus> {
        self.scan_transaction(domain, tx_id, None)
    }

    /// Appends exactly one begin/batch/commit transaction, synchronizing before
    /// success. A duplicate must match all batch bytes and header metadata.
    /// Incomplete or conflicting attempts require explicit recovery, never retry.
    pub fn append_transaction_once(
        &mut self,
        batch: &WalFrameHeader,
        body: &[u8],
    ) -> Result<WalTransactionOutcome> {
        if self.state.is_poisoned() {
            return Err(WalFileError::Poisoned);
        }
        batch.validate()?;
        super::validate_body_len(batch, body)?;
        super::verify_body_crc(batch, body)?;
        if batch.record_kind() != WalRecordKind::FactBatch
            || self.options.durability() != DurabilityMode::SyncAll
        {
            return Err(invalid_retry());
        }
        match self.scan_transaction(
            batch.encryption_domain(),
            batch.tx_id(),
            Some((batch, body)),
        )? {
            WalCommitStatus::Committed => {
                if self.file.sync_all().is_err() {
                    self.state.poison();
                    return Err(WalFileError::Poisoned);
                }
                return Ok(WalTransactionOutcome::AlreadyDurable);
            }
            WalCommitStatus::Incomplete => return Err(invalid_retry()),
            WalCommitStatus::Absent => {}
        }
        for (header, body) in [
            (
                marker(batch, WalRecordKind::TransactionBegin)?,
                &b"\x01"[..],
            ),
            (batch.clone(), body),
            (
                marker(batch, WalRecordKind::TransactionCommit)?,
                &b"\x01"[..],
            ),
        ] {
            if !matches!(
                self.append_frame(&header, body)?,
                WalAppendOutcome::Durable(_)
            ) {
                self.state.poison();
                return Err(WalFileError::Poisoned);
            }
        }
        Ok(WalTransactionOutcome::Durable)
    }

    fn scan_transaction(
        &mut self,
        domain: EncryptionDomain,
        tx_id: TxId,
        expected: Option<(&WalFrameHeader, &[u8])>,
    ) -> Result<WalCommitStatus> {
        if self.state.is_poisoned() {
            return Err(WalFileError::Poisoned);
        }
        // Recheck the complete file before interpreting a status. No truncation
        // or partial scan may turn an ambiguous append into permission to retry.
        if let Err(error) = super::scan::validate_tail(&mut self.file) {
            self.state.poison();
            return Err(error);
        }
        self.file.seek(SeekFrom::Start(0))?;
        let mut state = 0;
        let mut key_context = None;
        loop {
            let mut encoded = [0; WAL_FRAME_HEADER_BYTES];
            if matches!(
                read_exact_or_clean_eof(&mut self.file, &mut encoded)?,
                ReadState::CleanEof
            ) {
                break;
            }
            let header = WalFrameHeader::parse(&encoded)?;
            let len = header.encrypted_body_len();
            if header.tx_id() != tx_id || header.tenant_id() != domain.tenant_id() {
                self.file.seek(SeekFrom::Current(len as i64))?;
                continue;
            }
            if !header.encryption_domain().structurally_equal_ct(&domain) {
                return Err(invalid_retry());
            }
            let current_key = (header.encryption_key_id(), header.crypto_epoch());
            if key_context.is_some_and(|expected| expected != current_key) {
                return Err(invalid_retry());
            }
            key_context = Some(current_key);
            state = match (state, header.record_kind()) {
                (0, WalRecordKind::TransactionBegin) => 1,
                (1, WalRecordKind::FactBatch) => 2,
                (2, WalRecordKind::TransactionCommit) => 3,
                _ => return Err(invalid_retry()),
            };
            if let Some((batch, body)) = expected {
                let (expected_header, expected_body) = match header.record_kind() {
                    WalRecordKind::FactBatch => (batch.clone(), body),
                    kind => (marker(batch, kind)?, &b"\x01"[..]),
                };
                if encoded != expected_header.encode() {
                    return Err(invalid_retry());
                }
                let mut scratch = [0; 8192];
                for part in expected_body.chunks(scratch.len()) {
                    self.file
                        .read_exact(&mut scratch[..part.len()])
                        .map_err(map_partial_read)?;
                    if part != &scratch[..part.len()] {
                        return Err(invalid_retry());
                    }
                }
            } else if matches!(
                header.record_kind(),
                WalRecordKind::TransactionBegin | WalRecordKind::TransactionCommit
            ) {
                if len != 1 {
                    return Err(invalid_retry());
                }
                let mut marker = [0];
                self.file
                    .read_exact(&mut marker)
                    .map_err(map_partial_read)?;
                if marker != [1] {
                    return Err(invalid_retry());
                }
            } else {
                self.file.seek(SeekFrom::Current(len as i64))?;
            }
        }
        Ok(match state {
            0 => WalCommitStatus::Absent,
            3 => WalCommitStatus::Committed,
            _ => WalCommitStatus::Incomplete,
        })
    }
}

fn marker(batch: &WalFrameHeader, record_kind: WalRecordKind) -> Result<WalFrameHeader> {
    Ok(WalFrameHeader::new(WalFrameHeaderInput {
        record_kind,
        tenant_id: batch.tenant_id(),
        tx_id: batch.tx_id(),
        encryption_key_id: batch.encryption_key_id(),
        crypto_epoch: batch.crypto_epoch(),
        encryption_domain: batch.encryption_domain(),
        encrypted_body_len: 1,
        body_crc64: BodyChecksum::Present(wal_body_crc64(b"\x01")),
    })?)
}
fn invalid_retry() -> WalFileError {
    WalFileError::InvalidFrame(skrifheim_core::SkrifheimError::InvalidWalFrame(
        "WAL retry requires recovery or conflicts with previous attempt".into(),
    ))
}
