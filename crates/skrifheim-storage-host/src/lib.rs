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
    DurabilityMode, WalAppendOptions, WalAppendOutcome, WalCommitStatus, WalFileError,
    WalFileFrame, WalFileReader, WalFileWriter, WalReceipt, WalTransactionOutcome,
};

#[cfg(test)]
mod tests;
