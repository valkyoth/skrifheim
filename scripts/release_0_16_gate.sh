#!/usr/bin/env sh
set -eu

scripts/checks.sh

if ! grep -R "pub struct WalReplay" crates/skrifheim-storage/src >/dev/null; then
    echo "0.16 requires WAL replay state machine primitives" >&2
    exit 1
fi

if ! grep -R "WalRecoveryReport" crates/skrifheim-storage/src >/dev/null; then
    echo "0.16 requires WAL recovery report primitives" >&2
    exit 1
fi

if ! grep -R "TruncatedFrame" crates/skrifheim-storage/src/wal/replay.rs >/dev/null; then
    echo "0.16 requires explicit truncated-frame replay handling" >&2
    exit 1
fi

if ! grep -R "WAL_REPLAY_MAX_TRANSACTIONS" crates/skrifheim-storage/src >/dev/null; then
    echo "0.16 requires bounded WAL replay transaction summaries" >&2
    exit 1
fi

if ! grep -R "wal_body_crc64" crates/skrifheim-storage/src crates/skrifheim-storage-host/src >/dev/null; then
    echo "0.16 requires WAL body CRC verification helpers" >&2
    exit 1
fi

if ! grep -R "O_NOFOLLOW" crates/skrifheim-storage-host/src >/dev/null; then
    echo "0.16 requires Unix WAL symlink rejection" >&2
    exit 1
fi

if ! grep -R "wal_reader_rejects_symlink_paths" crates/skrifheim-storage-host/src >/dev/null; then
    echo "0.16 requires WAL reader symlink rejection coverage" >&2
    exit 1
fi

if ! grep -R "ContentDigest" crates/skrifheim-storage/src >/dev/null; then
    echo "0.16 requires segment integrity metadata to use ContentDigest" >&2
    exit 1
fi

if ! grep -R "structurally_equal_ct" crates/skrifheim-storage/src/wal/replay.rs crates/skrifheim-crypto/src/domain.rs >/dev/null; then
    echo "0.16 requires fixed-width replay domain comparison" >&2
    exit 1
fi

if ! grep -q 'actions/checkout v7.0.1 reviewed' .github/workflows/ci.yml; then
    echo "0.16 requires reviewed actions/checkout v7 series evidence" >&2
    exit 1
fi

if ! grep -q 'actions/checkout@3d3c42e5aac5ba805825da76410c181273ba90b1' .github/workflows/ci.yml; then
    echo "0.16 requires pinned actions/checkout v7.0.1 SHA" >&2
    exit 1
fi

if ! grep -q 'sanitization = { version = "2.0.2", default-features = false, features = \["alloc"\] }' crates/skrifheim-crypto/Cargo.toml; then
    echo "0.16 requires sanitization 2.0.2 with only alloc enabled" >&2
    exit 1
fi

cargo deny check
cargo audit
cargo run --quiet -p skrifheim

if [ "${SKRIFHEIM_SKIP_PODMAN:-0}" = "1" ]; then
    echo "SKRIFHEIM_SKIP_PODMAN=1 set; skipping rootless Podman smoke"
else
    scripts/podman_smoke.sh
fi
