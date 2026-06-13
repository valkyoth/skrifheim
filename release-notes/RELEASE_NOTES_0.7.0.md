# skrifheim 0.7.0 Release Notes

Status: implementation stop, pending pentest.

## Scope

`0.7.0` adds key hierarchy metadata and closes scaffold hardening gaps in
policy-token comparison, policy-label evaluation, signature-set validation, and
fact-builder deduplication.

## Changes

- Added key hierarchy metadata for root trust, deployment, region, tenant,
  compartment, segment, and data key scopes.
- Added parent/child hierarchy validation with no database-wide-key shortcut.
- Added invalid hierarchy edge tests.
- Bounded signature sets with `MAX_SIGNATURES_PER_SET`.
- Hardened policy-token equality with local compiler barriers while keeping the
  no-external-dependency posture.
- Changed policy-token set membership scans to fixed bounded slot counts.
- Changed policy-label compartment and releasability evaluation to fixed
  bounded slot counts.
- Changed fact-builder evidence and causal-link deduplication to sort/dedup
  behavior.
- Bumped workspace and internal crate dependency versions to `0.7.0`.
- Re-checked the stable Rust channel on 2026-06-13. Rust stable remains
  `1.96.0`, dated 2026-05-28 in the official stable manifest.

## Verification

- `cargo test -p skrifheim-core -p skrifheim-policy -p skrifheim-crypto -p skrifheim-fact`
- `scripts/checks.sh`
- `scripts/release_0_7_gate.sh`
- `cargo deny check`
- `cargo audit`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `scripts/podman_smoke.sh`

## Non-Claims

This release is not a usable database engine. It does not provide real key
generation, encryption, decryption, key rotation, KMS/HSM integration,
cryptographic verification, durable storage, or production-grade
constant-time evidence. Local compiler barriers reduce scaffold timing risk but
do not replace a future reviewed constant-time primitive admission or local
codegen evidence package.

## Pentest Status

Pentest is required before tagging. Root `PENTEST.md` is the temporary findings
handoff file and must be removed after findings are resolved.
