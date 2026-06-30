#!/usr/bin/env sh
set -eu

scripts/checks.sh

if ! grep -R "pub struct SegmentHeader" crates/skrifheim-storage/src/segment.rs >/dev/null; then
    echo "0.17 requires immutable segment header metadata" >&2
    exit 1
fi

if ! grep -R "pub struct SegmentFooter" crates/skrifheim-storage/src/segment.rs >/dev/null; then
    echo "0.17 requires immutable segment footer metadata" >&2
    exit 1
fi

if ! grep -R "SEGMENT_FOOTER_MAGIC" crates/skrifheim-storage/src/segment.rs >/dev/null; then
    echo "0.17 requires explicit segment footer magic" >&2
    exit 1
fi

if ! grep -R "crypto_epoch" crates/skrifheim-storage/src/segment.rs >/dev/null; then
    echo "0.17 requires segment key epoch metadata" >&2
    exit 1
fi

if ! grep -R "EncryptionDomainPurpose::Segment" crates/skrifheim-storage/src/segment crates/skrifheim-storage/src/segment.rs >/dev/null; then
    echo "0.17 requires segment encryption-domain validation" >&2
    exit 1
fi

if ! grep -R "validate_against_header" crates/skrifheim-storage/src/segment.rs >/dev/null; then
    echo "0.17 requires footer/header consistency validation" >&2
    exit 1
fi

if ! grep -R "structurally_equal_ct" crates/skrifheim-storage/src/segment crates/skrifheim-storage/src/segment.rs >/dev/null; then
    echo "0.17 requires fixed-width segment metadata comparison helpers" >&2
    exit 1
fi

if ! grep -R "BodyChecksum::Present(0)" crates/skrifheim-storage/src/segment crates/skrifheim-storage/src/segment.rs >/dev/null; then
    echo "0.17 requires zero CRC sentinel rejection" >&2
    exit 1
fi

if grep -R -E "const O_NOFOLLOW_FLAG: i32 = 0;" host/skrifheim-storage-host/src >/dev/null; then
    echo "0.17 requires unknown Unix WAL targets to fail closed instead of disabling O_NOFOLLOW" >&2
    exit 1
fi

if ! grep -R "structurally_equal_ct(&expected_domain)" crates/skrifheim-storage/src/wal.rs >/dev/null; then
    echo "0.17 requires constant-time WAL expected-domain comparison" >&2
    exit 1
fi

if ! grep -R "usize::try_from(header.encrypted_body_len())" host/skrifheim-storage-host/src >/dev/null; then
    echo "0.17 requires checked WAL body allocation length conversion" >&2
    exit 1
fi

if ! grep -R "validate_structure" crates/skrifheim-fact/src >/dev/null; then
    echo "0.17 requires structural fact validation to be explicitly named" >&2
    exit 1
fi

if ! grep -R "require_storage_validated" crates/skrifheim-world/src/diff.rs >/dev/null; then
    echo "0.17 requires world preflight storage-validation guards" >&2
    exit 1
fi

if ! grep -R "WAL_BODY_CRC64_TABLE" crates/skrifheim-storage/src/wal.rs >/dev/null; then
    echo "0.17 requires table-based WAL CRC64 implementation" >&2
    exit 1
fi

if ! grep -R "require_non_regressing_epoch" crates/skrifheim-storage/src/wal/replay.rs >/dev/null; then
    echo "0.17 requires WAL replay crypto epoch regression checks" >&2
    exit 1
fi

if ! grep -R "AUDIT_MIN_VALID_TIMESTAMP" crates/skrifheim-audit/src >/dev/null; then
    echo "0.17 requires audit clock initialization guard" >&2
    exit 1
fi

if ! grep -q "RELEASE_NOTES_0.17.0.md" scripts/validate-release-metadata.sh; then
    echo "0.17 release notes must be part of release metadata validation" >&2
    exit 1
fi

cargo deny check
cargo audit
cargo run --quiet -p skrifheim
scripts/validate-release-readiness.sh v0.17.0

if [ "${SKRIFHEIM_SKIP_PODMAN:-0}" = "1" ]; then
    echo "SKRIFHEIM_SKIP_PODMAN=1 set; skipping rootless Podman smoke"
else
    scripts/podman_smoke.sh
fi
