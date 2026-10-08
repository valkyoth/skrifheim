use super::{ReadState, Result, WalFileError, map_partial_read, read_exact_or_clean_eof};
use skrifheim_crypto::EncryptionDomain;
use skrifheim_storage::{WAL_FRAME_HEADER_BYTES, WalFrameHeader, wal_body_crc64_update};
use std::{
    fs::File,
    io::{Read, Seek, SeekFrom},
};

/// Validate every byte under the exclusive writer lock, with bounded scratch.
/// CRC is structural only; WAL-v1 cannot prove authenticity or ordering.
pub(super) fn validate_tail(file: &mut File, expected_domain: EncryptionDomain) -> Result<u64> {
    file.seek(SeekFrom::Start(0))?;
    let mut scratch = [0; 8192];
    loop {
        let mut bytes = [0; WAL_FRAME_HEADER_BYTES];
        if matches!(
            read_exact_or_clean_eof(file, &mut bytes)?,
            ReadState::CleanEof
        ) {
            return Ok(file.stream_position()?);
        }
        let header = WalFrameHeader::parse_for_domain(&bytes, expected_domain)?;
        let mut remaining = header.encrypted_body_len();
        let mut crc = 0;
        while remaining != 0 {
            let size = remaining.min(scratch.len() as u64) as usize;
            file.read_exact(&mut scratch[..size])
                .map_err(map_partial_read)?;
            crc = wal_body_crc64_update(crc, &scratch[..size]);
            remaining -= size as u64;
        }
        if crc != header.body_crc64() {
            return Err(WalFileError::InvalidFrame(
                skrifheim_core::SkrifheimError::InvalidWalFrame("WAL tail CRC mismatch".into()),
            ));
        }
    }
}
