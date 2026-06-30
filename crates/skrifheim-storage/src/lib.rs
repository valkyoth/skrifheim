#![no_std]
#![forbid(unsafe_code)]

extern crate alloc;

mod segment;
mod wal;

pub use segment::{
    BodyChecksum, SEGMENT_BODY_MAX_BYTES, SEGMENT_FOOTER_MAGIC, SEGMENT_FOOTER_VERSION_MAX,
    SEGMENT_MAGIC, SEGMENT_VERSION_MAX, SegmentFooter, SegmentFooterInput, SegmentHeader,
    SegmentHeaderInput, SegmentKind,
};
pub use wal::{
    WAL_BODY_CRC64_ECMA_POLY, WAL_FRAME_BODY_MAX_BYTES, WAL_FRAME_HEADER_BYTES, WAL_FRAME_MAGIC,
    WAL_FRAME_VERSION_MAX, WAL_REPLAY_MAX_TRANSACTIONS, WalFrameHeader, WalFrameHeaderInput,
    WalRecordKind, WalRecoveredTransaction, WalRecoveryOutcome, WalRecoveryReport, WalReplay,
    WalReplayStop, WalRollbackReason, WalRolledBackTransaction, wal_body_crc64,
};
