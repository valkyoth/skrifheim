# skrifheim 0.8.0 Release Notes

Status: implementation stop, pending pentest.

## Scope

`0.8.0` adds key lifecycle metadata on top of the `0.7.0` key hierarchy
scaffold. It models key state transitions, rotation preflight, compromise and
quarantine states, destruction, and crypto-erasure records.

## Changes

- Added `KeyLifecycleState` for created, active, rotating, retired,
  compromised, quarantined, destroyed, and crypto-erased keys.
- Added lifecycle transition validation through `KeyMetadata::transition`.
- Added `KeyRotationPreflight` for validating active key to candidate key
  rotation metadata.
- Added `KeyErasureMetadata` and `KeyErasureReason` for crypto-erasure records.
- Added compromise, quarantine, destruction, and erasure transition tests.
- Added invalid lifecycle transition and invalid rotation preflight tests.
- Bumped workspace and internal crate dependency versions to `0.8.0`.
- Added `scripts/release_0_8_gate.sh`.
- Re-checked the stable Rust channel on 2026-06-14. Rust stable remains
  `1.96.0`, dated 2026-05-28 in the official stable manifest.

## Verification

- `cargo test -p skrifheim-crypto`
- `scripts/checks.sh`
- `scripts/release_0_8_gate.sh`
- `cargo deny check`
- `cargo audit`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `scripts/podman_smoke.sh`

## Non-Claims

This release is not a usable database engine. It does not generate, store,
encrypt, decrypt, rotate, recover, or erase real key material. Lifecycle and
erasure records are metadata only. No KMS/HSM integration, threshold approval,
durable storage, or cryptographic verification is implemented.

Future key-material types must follow the existing `sanitization` dependency
admission rule for memory cleanup; `zeroize` is not admitted for this project.

## Pentest Status

Pentest is required before tagging. Root `PENTEST.md` is the temporary findings
handoff file and must be removed after findings are resolved.
