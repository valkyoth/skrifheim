#![forbid(unsafe_code)]

mod common;
mod segment;
mod wal;

pub use segment::{
    MAX_IN_MEMORY_SEGMENT_BYTES, SegmentContentVerifier, SegmentFileError, SegmentFileReader,
    SegmentFileSegment, SegmentFileWriter, SegmentPublishOutcome, SegmentWriteOptions,
    cleanup_staged_segments,
};
pub use wal::{WalAppendOptions, WalFileError, WalFileFrame, WalFileReader, WalFileWriter};

#[cfg(test)]
mod tests;
