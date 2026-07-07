#!/usr/bin/env sh
set -eu

scripts/checks.sh

if ! grep -R "pub struct SovereigntyScope" crates/skrifheim-policy/src/result.rs >/dev/null; then
    echo "0.18.1 requires typed sovereignty scope" >&2
    exit 1
fi

if ! grep -R "MultiJurisdiction" crates/skrifheim-policy/src/result.rs >/dev/null; then
    echo "0.18.1 requires multi-jurisdiction sovereignty sentinel" >&2
    exit 1
fi

if ! grep -R "SOVEREIGNTY_SCOPE_INPUT_MAX_ITEMS" crates/skrifheim-policy/src/result.rs >/dev/null; then
    echo "0.18.1 requires bounded sovereignty scope input" >&2
    exit 1
fi

if ! grep -R "SovereigntyContainment::Indeterminate" crates/skrifheim-policy/src/result.rs crates/skrifheim-policy/src/result_tests.rs >/dev/null; then
    echo "0.18.1 requires explicit indeterminate containment for saturated sovereignty scope" >&2
    exit 1
fi

if ! grep -R "requires_restrictive_handling" crates/skrifheim-policy/src/result.rs >/dev/null; then
    echo "0.18.1 requires explicit restrictive-handling marker" >&2
    exit 1
fi

if ! grep -R "plan_preserves_saturated_sovereignty_scope" crates/skrifheim-query/src/tests.rs >/dev/null; then
    echo "0.18.1 requires query-plan saturation coverage" >&2
    exit 1
fi

if ! grep -R "sovereignty_overflow_still_rejects_invalid_tokens" crates/skrifheim-policy/src/result_tests.rs >/dev/null; then
    echo "0.18.1 requires invalid-token rejection coverage" >&2
    exit 1
fi

if ! grep -R "check_no_sensitive_derive .* SovereigntyScope" scripts/validate-security-policy.sh >/dev/null; then
    echo "0.18.1 requires security-policy derive checks for SovereigntyScope" >&2
    exit 1
fi

if ! grep -R "SovereigntyScope.*contains" scripts/validate-security-policy.sh >/dev/null; then
    echo "0.18.1 requires security-policy gate against bare sovereignty containment" >&2
    exit 1
fi

if ! grep -R "#\\[cfg(not(unix))\\]" host/skrifheim-storage-host/src/lib.rs >/dev/null; then
    echo "0.18.1 requires host storage to fail closed on unsupported non-Unix targets" >&2
    exit 1
fi

if ! grep -R "has_expiring_attestation" crates/skrifheim-audit/src/lib.rs >/dev/null; then
    echo "0.18.1 requires break-glass audit events to use expiring attestation evidence" >&2
    exit 1
fi

if ! grep -R "incoming.len() > max_items" crates/skrifheim-world/src/lib.rs >/dev/null; then
    echo "0.18.1 requires batch world fact input to be bounded before sorting" >&2
    exit 1
fi

if ! grep -q "RELEASE_NOTES_0.18.1.md" scripts/validate-release-metadata.sh; then
    echo "0.18.1 release notes must be part of release metadata validation" >&2
    exit 1
fi

if ! grep -q "security/pentest/v0.18.1.md" scripts/validate-release-metadata.sh; then
    echo "0.18.1 pentest report must be part of release metadata validation" >&2
    exit 1
fi

cargo deny check
cargo audit
cargo run --quiet -p skrifheim
scripts/validate-release-readiness.sh v0.18.1

if [ "${SKRIFHEIM_SKIP_PODMAN:-0}" = "1" ]; then
    echo "SKRIFHEIM_SKIP_PODMAN=1 set; skipping rootless Podman smoke"
else
    scripts/podman_smoke.sh
fi
