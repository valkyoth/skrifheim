//! Development-only bounded harnesses shared by corpus replay and libFuzzer.
//! Never link this unpublished crate into a deployed application.
#![no_std]
#![forbid(unsafe_code)]

extern crate alloc;

mod crypto;
mod replay;
pub mod seeds;
mod storage;

pub const TARGETS: [&str; 6] = [
    "wal_header",
    "wal_replay",
    "segment_metadata",
    "segment_layout",
    "aead_envelope",
    "transcript",
];
pub const MAX_INPUT: usize = 65_596;

/// Unknown targets and oversize inputs are errors, never successful empty runs.
pub fn exercise(target: &str, data: &[u8]) -> Result<(), &'static str> {
    if data.len() > MAX_INPUT {
        return Err("input budget exceeded");
    }
    match target {
        "wal_header" => storage::wal_header(data),
        "wal_replay" => replay::exercise(data),
        "segment_metadata" => storage::segment_metadata(data),
        "segment_layout" => storage::segment_layout(data),
        "aead_envelope" => crypto::envelope(data),
        "transcript" => crypto::transcript(data),
        _ => return Err("unknown assurance target"),
    }
    Ok(())
}

/// A deterministic mutation corpus, not a coverage-guided fuzz campaign.
pub fn smoke(target: &str) -> Result<usize, &'static str> {
    let mut count = 0;
    for seed in seeds::for_target(target)? {
        exercise(target, &seed)?;
        count += 1;
        for i in 0..seed.len().min(512) {
            exercise(target, &seed[..i])?;
            let mut changed = seed.clone();
            changed[i] ^= 0xff;
            exercise(target, &changed)?;
            count += 2;
        }
    }
    assert!(count > 0);
    Ok(count)
}

#[cfg(test)]
mod tests;
