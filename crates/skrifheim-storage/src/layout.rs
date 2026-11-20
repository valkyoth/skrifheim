use crate::{SEGMENT_BODY_MAX_BYTES, SEGMENT_FOOTER_BYTES, SEGMENT_HEADER_BYTES};

/// Structural file extent only, not checksum, authenticity or freshness evidence.
/// Checks a declared body without allocating it or narrowing its length to usize.
pub const fn segment_file_length_matches(body_len: u64, file_len: u64) -> bool {
    if body_len == 0 || body_len > SEGMENT_BODY_MAX_BYTES {
        return false;
    }
    match body_len.checked_add((SEGMENT_HEADER_BYTES + SEGMENT_FOOTER_BYTES) as u64) {
        Some(expected) => expected == file_len,
        None => false,
    }
}
