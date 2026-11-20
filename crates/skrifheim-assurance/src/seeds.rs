//! Frozen wire fixtures, assembled from documented offsets, not production encoders.
use alloc::{vec, vec::Vec};

pub fn wal() -> Vec<u8> {
    let mut b = vec![0; 120];
    b[..8].copy_from_slice(b"SKRIFWAL");
    for offset in [8, 10, 16, 64, 80, 88, 104, 112] {
        b[offset] = 1;
    }
    b
}

pub fn segment() -> Vec<u8> {
    let mut b = vec![0; 256];
    b[..8].copy_from_slice(b"SKRIFSEG");
    for offset in [
        8, 10, 16, 32, 48, 64, 80, 96, 120, 128, 144, 160, 168, 176, 192,
    ] {
        b[offset] = 1;
    }
    b
}

pub fn footer() -> Vec<u8> {
    let mut b = segment();
    b[..8].copy_from_slice(b"SKRIFFTR");
    b[8] = 2;
    b
}

/// Independent libsodium XChaCha20-Poly1305 fixture recorded in v0.18.3.
pub fn envelope() -> Vec<u8> {
    let mut b = b"SKRIFENC\x01\0\x01\0".to_vec();
    b.extend_from_slice(&[10; 24]);
    b.extend_from_slice(&28_u64.to_le_bytes());
    b.extend_from_slice(&[
        0xbd, 0x50, 0xdc, 0x9a, 0xb8, 0x5e, 0x97, 0xe4, 0x66, 0xd7, 0xf0, 0xf0, 0xaf, 0xbc, 0x9b,
        0x95, 0x1d, 0xb8, 0x3e, 0xbb, 0x58, 0x45, 0x22, 0x78, 0xbe, 0x21, 0xc8, 0x44,
    ]);
    b
}

pub fn for_target(target: &str) -> Result<Vec<Vec<u8>>, &'static str> {
    let mut seeds = vec![vec![], vec![0], vec![0xff; 16]];
    match target {
        "wal_header" => {
            seeds.push(wal());
            let mut b = wal();
            b[104..112].fill(0xff);
            seeds.push(b);
        }
        "wal_replay" => {
            // Control: capacity, EOF mode; then kind/tx/epoch/key/tenant tuples.
            seeds.push(vec![4, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 2, 1, 1, 1, 1]);
            seeds.push(vec![0, 0, 0, 1, 1, 1, 1, 2, 1, 1, 1, 1]);
            seeds.push(vec![4, 1, 0, 1, 1, 1, 1]);
        }
        "segment_metadata" => {
            seeds.push(segment());
            seeds.push(footer());
            let mut b = segment();
            b.extend_from_slice(&footer());
            seeds.push(b);
        }
        "segment_layout" => {
            let mut file = segment();
            file.push(42);
            file.extend_from_slice(&footer());
            seeds.push(file);
            for body in [0_u64, 1, 1 << 30, (1 << 30) + 1, u64::MAX] {
                let mut b = body.to_le_bytes().to_vec();
                b.extend_from_slice(&body.saturating_add(512).to_le_bytes());
                seeds.push(b);
            }
        }
        "aead_envelope" => {
            seeds.push(envelope());
            let mut b = envelope();
            b[36..44].fill(0xff);
            seeds.push(b);
            let mut b = envelope();
            b.push(0);
            seeds.push(b);
        }
        "transcript" => {
            seeds.push(vec![1, 0, 3, b'a', b'b', b'c', 1, 0, 1, b'x']);
            seeds.push(vec![0xff; 1024]);
        }
        _ => return Err("unknown assurance target"),
    }
    Ok(seeds)
}
