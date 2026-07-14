#!/usr/bin/env sh
set -eu

scripts/checks.sh

if ! grep -R "pub struct KeyLifecycleEventSequence" crates/skrifheim-crypto/src/key.rs >/dev/null; then
    echo "0.18.2 requires lifecycle event sequence type" >&2
    exit 1
fi

if ! grep -R "lifecycle_event_sequence" crates/skrifheim-crypto/src/key.rs >/dev/null; then
    echo "0.18.2 requires lifecycle event ordering on key metadata" >&2
    exit 1
fi

if ! grep -R "requires_crypto_epoch_advancement" crates/skrifheim-crypto/src/key.rs >/dev/null; then
    echo "0.18.2 requires explicit crypto epoch advancement policy" >&2
    exit 1
fi

if ! grep -R "next_epoch != self.epoch" crates/skrifheim-crypto/src/key.rs >/dev/null; then
    echo "0.18.2 requires metadata-only lifecycle transitions to preserve crypto epoch" >&2
    exit 1
fi

if ! grep -R "metadata_only_lifecycle_changes_preserve_crypto_epoch_and_advance_event_sequence" crates/skrifheim-crypto/src/tests/key_lifecycle.rs >/dev/null; then
    echo "0.18.2 requires metadata-only lifecycle transition coverage" >&2
    exit 1
fi

if ! grep -R "active.transition(KeyLifecycleState::Rotating, CryptoEpoch::new(3))" crates/skrifheim-crypto/src/tests/key_lifecycle.rs >/dev/null; then
    echo "0.18.2 requires rotation epoch advancement rejection coverage" >&2
    exit 1
fi

if ! grep -q "RELEASE_NOTES_0.18.2.md" scripts/validate-release-metadata.sh; then
    echo "0.18.2 release notes must be part of release metadata validation" >&2
    exit 1
fi

if ! grep -q "security/pentest/v0.18.2.md" scripts/validate-release-metadata.sh; then
    echo "0.18.2 pentest report must be part of release metadata validation" >&2
    exit 1
fi

cargo deny check
cargo audit
cargo run --quiet -p skrifheim
scripts/validate-release-readiness.sh v0.18.2

if [ "${SKRIFHEIM_SKIP_PODMAN:-0}" = "1" ]; then
    echo "SKRIFHEIM_SKIP_PODMAN=1 set; skipping rootless Podman smoke"
else
    scripts/podman_smoke.sh
fi
