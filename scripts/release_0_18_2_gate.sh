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

if grep -R "pub fn reconstruct" crates/skrifheim-crypto/src/key.rs >/dev/null; then
    echo "0.18.2 must not expose public arbitrary key metadata reconstruction" >&2
    exit 1
fi

if ! cargo test -p skrifheim-storage --quiet \
    segment::tests::decoded_footer_rejects_header_kind_mismatch \
    -- --exact >/dev/null; then
    echo "0.18.2 requires segment footer kind binding coverage" >&2
    exit 1
fi

if ! cargo test -p skrifheim-storage --quiet \
    segment::tests::legacy_v1_footer_keeps_kind_unbound_explicit \
    -- --exact >/dev/null; then
    echo "0.18.2 requires explicit legacy v1 footer migration-only coverage" >&2
    exit 1
fi

if ! cargo test -p skrifheim-storage-host --quiet \
    tests::segment::segment_reader_rejects_v2_to_v1_kind_binding_downgrade \
    -- --exact >/dev/null; then
    echo "0.18.2 requires segment kind downgrade rejection coverage" >&2
    exit 1
fi

if ! cargo test -p skrifheim-storage-host --quiet \
    tests::segment::segment_reader_rejects_oversized_sparse_segment_before_allocation \
    -- --exact >/dev/null; then
    echo "0.18.2 requires sparse segment allocation-limit coverage" >&2
    exit 1
fi

if ! cargo test -p skrifheim-storage-host --quiet \
    tests::segment::segment_writer_rejects_oversized_body_before_publishing \
    -- --exact >/dev/null; then
    echo "0.18.2 requires segment writer/read host cap symmetry coverage" >&2
    exit 1
fi

if ! cargo test -p skrifheim-storage-host --quiet \
    tests::segment::segment_writer_and_reader_round_trip_max_in_memory_body \
    -- --exact >/dev/null; then
    echo "0.18.2 requires max host segment round-trip coverage" >&2
    exit 1
fi

if ! cargo test -p skrifheim-storage-host --quiet \
    tests::segment::segment_writer_does_not_publish_until_complete_write \
    -- --exact >/dev/null; then
    echo "0.18.2 requires staged segment publication coverage" >&2
    exit 1
fi

if ! grep -R "pub enum SegmentPublishOutcome" crates/skrifheim-storage-host/src/segment.rs >/dev/null; then
    echo "0.18.2 requires explicit segment publication outcomes" >&2
    exit 1
fi

if ! grep -R "PublishedDurabilityUnknown" crates/skrifheim-storage-host/src/segment.rs >/dev/null; then
    echo "0.18.2 requires explicit post-publication durability uncertainty" >&2
    exit 1
fi

if ! cargo test -p skrifheim-storage-host --quiet \
    tests::segment_cleanup::published_durability_error_does_not_disclose_storage_path \
    -- --exact >/dev/null; then
    echo "0.18.2 requires redacted post-publication durability diagnostics" >&2
    exit 1
fi

if ! cargo test -p skrifheim-storage-host --quiet \
    tests::segment_cleanup::cleanup_staged_segments_removes_owned_strict_candidates_only \
    -- --exact >/dev/null; then
    echo "0.18.2 requires strict staged segment cleanup coverage" >&2
    exit 1
fi

if ! cargo test -p skrifheim-storage-host --quiet \
    tests::segment_cleanup::cleanup_staged_segments_rejects_symlink_candidates \
    -- --exact >/dev/null; then
    echo "0.18.2 requires staged segment cleanup symlink rejection coverage" >&2
    exit 1
fi

if ! cargo test -p skrifheim-storage-host --quiet \
    tests::segment_cleanup::cleanup_staged_segments_preserves_published_staging_like_segment_name \
    -- --exact >/dev/null; then
    echo "0.18.2 requires staged cleanup to preserve published staging-like names" >&2
    exit 1
fi

if ! cargo test -p skrifheim-storage-host --quiet \
    tests::segment_cleanup::segment_writer_rejects_reserved_staging_namespace_target \
    -- --exact >/dev/null; then
    echo "0.18.2 requires writer rejection for reserved staging namespace targets" >&2
    exit 1
fi

if ! cargo test -p skrifheim-storage-host --quiet \
    tests::segment_cleanup::cleanup_staged_segments_rejects_symlink_directory \
    -- --exact >/dev/null; then
    echo "0.18.2 requires staged cleanup symlink directory rejection coverage" >&2
    exit 1
fi

if ! cargo test -p skrifheim-storage-host --quiet \
    tests::segment::segment_writer_requires_explicit_parent_for_new_files \
    -- --exact >/dev/null; then
    echo "0.18.2 requires segment bare-path error atomicity coverage" >&2
    exit 1
fi

if ! cargo test -p skrifheim-storage-host --quiet \
    tests::wal::wal_writer_rejects_second_concurrent_writer \
    -- --exact >/dev/null; then
    echo "0.18.2 requires exclusive WAL writer coverage" >&2
    exit 1
fi

if ! cargo test -p skrifheim-storage-host --quiet \
    tests::wal::wal_writer_requires_explicit_parent_for_new_files \
    -- --exact >/dev/null; then
    echo "0.18.2 requires WAL bare-path error atomicity coverage" >&2
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

if ! grep -q 'actions/checkout@9c091bb21b7c1c1d1991bb908d89e4e9dddfe3e0' .github/workflows/ci.yml; then
    echo "0.18.2 requires pinned actions/checkout v7.0.0 SHA" >&2
    exit 1
fi

if ! grep -q 'cargo install --locked --version 0.20.2 cargo-deny' .github/workflows/ci.yml; then
    echo "0.18.2 requires pinned cargo-deny CI install" >&2
    exit 1
fi

if ! grep -q 'cargo install --locked --version 0.22.2 cargo-audit' .github/workflows/ci.yml; then
    echo "0.18.2 requires pinned cargo-audit CI install" >&2
    exit 1
fi

if ! grep -q 'docker.io/library/rust@sha256:606f3248aa86ce49e0b98d9e0bbffde042adeb18982320f97bcc218615de1c99' Containerfile; then
    echo "0.18.2 requires pinned Rust 1.97 container digest" >&2
    exit 1
fi

if ! grep -q 'cargo build --release --locked -p skrifheim' Containerfile; then
    echo "0.18.2 requires locked container builds" >&2
    exit 1
fi

if ! grep -q 'gcr.io/distroless/cc-debian12@sha256:66aa873a4a14fb164aa01296058efd8253744606d72715e45acface073359faa' Containerfile; then
    echo "0.18.2 requires pinned distroless runtime digest" >&2
    exit 1
fi

if ! grep -q 'docker.io/library/rust@sha256:ec9c91e77119ce498cd1e87d96d77e0f75b2cee21655a29bc2bf75a51a2b20a4' containers/Containerfile.alpine; then
    echo "0.18.2 requires pinned Alpine Rust 1.97 container digest" >&2
    exit 1
fi

if ! grep -q 'cargo test --workspace --locked' containers/Containerfile.alpine; then
    echo "0.18.2 requires locked Alpine container test builds" >&2
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
