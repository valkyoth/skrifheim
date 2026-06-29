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

if ! grep -q 'actions/checkout@v7' .github/workflows/ci.yml; then
    echo "0.16 requires actions/checkout v7 series" >&2
    exit 1
fi

if ! grep -q 'sanitization = { version = "1.2.2", default-features = false, features = \["alloc"\] }' crates/skrifheim-crypto/Cargo.toml; then
    echo "0.16 requires sanitization 1.2.2 with only alloc enabled" >&2
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
