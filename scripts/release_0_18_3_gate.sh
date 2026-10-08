#!/usr/bin/env sh
set -eu

case "${1:-}" in
    ''|--release) ;;
    *) echo "usage: sh scripts/release_0_18_3_gate.sh [--release]" >&2; exit 2 ;;
esac

scripts/checks.sh
cargo deny check
cargo audit

if [ "${SKRIFHEIM_SKIP_PODMAN:-0}" != 1 ]; then
    scripts/podman_smoke.sh
    podman build -t skrifheim:alpine-tests -f containers/Containerfile.alpine .
fi

if [ "${1:-}" = --release ]; then
    scripts/validate-release-readiness.sh v0.18.3
else
    echo "v0.18.3 local implementation checks passed; maintainer pentest is required before release."
fi
