#!/usr/bin/env sh
set -eu

scripts/checks.sh

if ! grep -R "pub const SEGMENT_HEADER_BYTES: usize = 256" crates/skrifheim-storage/src/segment.rs >/dev/null; then
    echo "0.18 requires fixed-size segment headers" >&2
    exit 1
fi

if ! grep -R "pub const SEGMENT_FOOTER_BYTES: usize = 256" crates/skrifheim-storage/src/segment.rs >/dev/null; then
    echo "0.18 requires fixed-size segment footers" >&2
    exit 1
fi

if ! grep -R "pub(super) fn parse_header" crates/skrifheim-storage/src/segment/encoding.rs >/dev/null; then
    echo "0.18 requires dedicated segment header encoding/parser module" >&2
    exit 1
fi

if ! grep -R "pub(super) fn parse_footer" crates/skrifheim-storage/src/segment/encoding.rs >/dev/null; then
    echo "0.18 requires dedicated segment footer encoding/parser module" >&2
    exit 1
fi

if ! grep -R "parse_for_domain" crates/skrifheim-storage/src/segment.rs >/dev/null; then
    echo "0.18 requires segment expected-domain parsing" >&2
    exit 1
fi

if ! grep -R "SegmentFileWriter" crates/skrifheim-storage-host/src/lib.rs >/dev/null; then
    echo "0.18 requires host segment writer" >&2
    exit 1
fi

if ! grep -R "SegmentFileReader" crates/skrifheim-storage-host/src/lib.rs >/dev/null; then
    echo "0.18 requires host segment reader" >&2
    exit 1
fi

if ! grep -R "SegmentContentVerifier" crates/skrifheim-storage-host/src/lib.rs >/dev/null; then
    echo "0.18 requires explicit segment content verifier boundary" >&2
    exit 1
fi

if ! grep -R "verify_segment_file_len" crates/skrifheim-storage-host/src >/dev/null; then
    echo "0.18 requires exact segment file-length validation" >&2
    exit 1
fi

if ! grep -R "fsync_parent_dir" crates/skrifheim-storage-host/src >/dev/null; then
    echo "0.18 requires parent-directory fsync after new host file creation" >&2
    exit 1
fi

if ! grep -R "SegmentContentVerifier production impls are blocked" scripts/validate-security-policy.sh >/dev/null; then
    echo "0.18 requires production verifier impls to remain blocked until digest engine admission" >&2
    exit 1
fi

if ! grep -R "0.18.0.*full mirrored" docs/VERSION_PLAN.md docs/IMPLEMENTATION_PLAN.md >/dev/null; then
    echo "0.18 requires documented full mirrored footer decision" >&2
    exit 1
fi

if ! grep -q "RELEASE_NOTES_0.18.0.md" scripts/validate-release-metadata.sh; then
    echo "0.18 release notes must be part of release metadata validation" >&2
    exit 1
fi

if ! grep -q "security/pentest/v0.18.0.md" scripts/validate-release-metadata.sh; then
    echo "0.18 pentest report must be part of release metadata validation" >&2
    exit 1
fi

cargo deny check
cargo audit
cargo run --quiet -p skrifheim
scripts/validate-release-readiness.sh v0.18.0

if [ "${SKRIFHEIM_SKIP_PODMAN:-0}" = "1" ]; then
    echo "SKRIFHEIM_SKIP_PODMAN=1 set; skipping rootless Podman smoke"
else
    scripts/podman_smoke.sh
fi
