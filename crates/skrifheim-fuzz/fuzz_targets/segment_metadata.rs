#![no_main]
#![forbid(unsafe_code)]

libfuzzer_sys::fuzz_target!(|data: &[u8]| {
    assert!(skrifheim_assurance::exercise("segment_metadata", data).is_ok());
});
