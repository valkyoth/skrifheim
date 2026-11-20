#!/usr/bin/env sh
set -eu
case "${1:-}" in
    ''|--release) ;;
    *) echo "usage: sh scripts/release_0_18_4_gate.sh [--release]" >&2; exit 2 ;;
esac
scripts/checks.sh
cargo test --release --locked --workspace --all-features
cargo deny --locked check
cargo audit
cargo deny --locked --manifest-path crates/skrifheim-fuzz/Cargo.toml --config crates/skrifheim-fuzz/deny.toml check
cargo audit --file crates/skrifheim-fuzz/Cargo.lock
python3 scripts/assurance.py fuzz all
if [ "${SKRIFHEIM_SKIP_PODMAN:-0}" != 1 ]; then
    scripts/podman_smoke.sh
    podman build -t skrifheim:alpine-tests -f containers/Containerfile.alpine .
fi
if [ "${1:-}" = --release ]; then
    scripts/validate-release-readiness.sh v0.18.4
else
    echo "v0.18.4 is ready for pentest."
fi
