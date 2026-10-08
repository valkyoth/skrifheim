#![forbid(unsafe_code)]

mod common;
mod segment;
mod wal;

pub use segment::{
    MAX_IN_MEMORY_SEGMENT_BYTES, SegmentContentVerifier, SegmentFileError, SegmentFileReader,
    SegmentFileSegment, SegmentFileWriter, SegmentPublishOutcome, SegmentWriteOptions,
    cleanup_staged_segments,
};
pub use wal::{
    DurabilityMode, WAL_FILE_MAX_BYTES, WAL_FILE_MAX_FRAMES, WalAppendOptions, WalAppendOutcome,
    WalCommitStatus, WalFileError, WalFileFrame, WalFileLimits, WalFileReader, WalFileWriter,
    WalReceipt, WalTransactionOutcome,
};

#[cfg(test)]
mod tests;
