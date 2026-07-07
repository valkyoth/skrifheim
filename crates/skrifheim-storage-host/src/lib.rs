#![forbid(unsafe_code)]

mod common;
mod segment;
mod wal;

pub use segment::{
    SegmentContentVerifier, SegmentFileError, SegmentFileReader, SegmentFileSegment,
    SegmentFileWriter, SegmentWriteOptions,
};
pub use wal::{WalAppendOptions, WalFileError, WalFileFrame, WalFileReader, WalFileWriter};

#[cfg(test)]
mod tests;
