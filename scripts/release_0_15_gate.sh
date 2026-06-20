#!/usr/bin/env sh
set -eu

scripts/checks.sh

if ! grep -R "WorldIdentityDigest" crates/skrifheim-crypto/src >/dev/null; then
    echo "0.15 requires full-width world identity digest skeletons" >&2
    exit 1
fi

if ! grep -R "ContentDigest" crates/skrifheim-crypto/src >/dev/null; then
    echo "0.15 requires content digest skeletons" >&2
    exit 1
fi

if ! grep -R "ManifestDigest" crates/skrifheim-crypto/src >/dev/null; then
    echo "0.15 requires manifest digest skeletons" >&2
    exit 1
fi

if ! grep -q "BLAKE3 remains" release-notes/RELEASE_NOTES_0.15.0.md \
    || ! grep -q "scaffold-only" release-notes/RELEASE_NOTES_0.15.0.md; then
    echo "0.15 release notes must preserve the BLAKE3 scaffold-only boundary" >&2
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
