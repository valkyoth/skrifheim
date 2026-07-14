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

if ! cargo test -p skrifheim-crypto --quiet \
    tests::key_lifecycle::metadata_only_lifecycle_changes_preserve_crypto_epoch_and_advance_event_sequence \
    -- --exact >/dev/null; then
    echo "0.18.2 requires metadata-only lifecycle transitions to preserve crypto epoch" >&2
    exit 1
fi

if ! cargo test -p skrifheim-crypto --quiet \
    tests::key_lifecycle::key_lifecycle_rejects_invalid_transitions_and_epoch_shape \
    -- --exact >/dev/null; then
    echo "0.18.2 requires rotation epoch advancement rejection coverage" >&2
    exit 1
fi

if ! cargo test -p skrifheim-crypto --quiet \
    tests::key_lifecycle::reconstructed_key_metadata_requires_valid_lifecycle_sequence_and_erasure \
    -- --exact >/dev/null; then
    echo "0.18.2 requires checked key metadata reconstruction coverage" >&2
    exit 1
fi

if ! cargo test -p skrifheim-storage --quiet \
    segment::tests::decoded_footer_rejects_header_kind_mismatch \
    -- --exact >/dev/null; then
    echo "0.18.2 requires segment footer kind binding coverage" >&2
    exit 1
fi

if ! cargo test -p skrifheim-storage-host --quiet \
    tests::segment::segment_reader_rejects_oversized_sparse_segment_before_allocation \
    -- --exact >/dev/null; then
    echo "0.18.2 requires sparse segment allocation-limit coverage" >&2
    exit 1
fi

if ! cargo test -p skrifheim-storage-host --quiet \
    tests::wal::wal_writer_rejects_second_concurrent_writer \
    -- --exact >/dev/null; then
    echo "0.18.2 requires exclusive WAL writer coverage" >&2
    exit 1
fi

if ! cargo test -p skrifheim-storage --quiet \
    wal::replay::tests::replay_rejected_close_keeps_active_transaction \
    -- --exact >/dev/null; then
    echo "0.18.2 requires non-destructive WAL replay rejection coverage" >&2
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
