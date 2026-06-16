# skrifheim 0.12.0 Release Notes

Status: release-prep complete; local release gate, submitted pentest handoffs,
and GitHub verification passed; pending signed tag creation.

## Scope

`0.12.0` creates the first memory-secrecy boundary before real key material,
KMS/HSM integration, WAL encryption, or durable encrypted storage exists. The
release admits the project-owned `sanitization` crate for clear-on-drop secret
storage and exposes a narrow `SecretBytes` wrapper for future crypto-control
plane APIs.

## Changes

- Added `SecretBytes` in `skrifheim-crypto`.
- Bounded secret material with `SECRET_VALUE_MAX_BYTES`.
- Rejected empty secret material.
- Rejected owned vectors whose allocation capacity exceeds the secret bound.
- Added closure-only secret access through `with_secret` and
  `try_with_secret`.
- Added explicit `clear_secret` and `into_cleared` paths.
- Added redacted `Debug` output for `SecretBytes` without exact byte lengths.
- Added `InvalidSecretMaterial` with static internal and public error text.
- Admitted `sanitization` `1.1.0` with `default-features = false` and only the
  `alloc` feature.
- Documented the external dependency exception in
  `docs/engineering-policy.md`.
- Added `docs/memory-secrecy.md`.
- Updated encryption architecture and security controls to record the
  scaffolded memory-secrecy boundary.
- Added release-gate checks that block `Debug`, `Clone`, `PartialEq`, and `Eq`
  derives on `SecretBytes`.
- Added release-gate checks that block raw secret byte accessors on
  `SecretBytes`.
- Redacted `FactBuilder` classification diagnostics.
- Redacted exact `PolicyTokenSlot` token lengths.
- Made `SecretBytes::from_slice` validate allocation capacity consistently with
  `from_vec`.
- Documented that `SecretBytes::from_slice` copies from and does not clear the
  borrowed source slice.
- Replaced scaffold world-id polynomial hashing with admitted BLAKE3
  domain-separated derivation.
- Removed spaces and slash characters from valid world names.
- Made `SegmentHeader` fields private and added a validating constructor with
  read-only accessors.
- Added release-gate checks for builder/token debug regressions and public
  segment-header fields.
- Documented that `WorldId` uses 16 bytes of BLAKE3 output because the ID is a
  `u128`; the forced low bit only satisfies the non-zero ID type and is not a
  secrecy boundary.
- Bumped workspace and internal crate dependency versions to `0.12.0`.
- Added `scripts/release_0_12_gate.sh`.

## Verification

- `cargo info sanitization`
- `cargo test -p skrifheim-crypto`
- `scripts/checks.sh`
- `scripts/release_0_12_gate.sh`

## Non-Claims

This release does not generate keys, encrypt WAL or segment bytes, lock memory,
disable process dumps, prevent swap, scrub CPU caches, integrate with a KMS or
HSM, or provide hardware-backed secret handles. It only establishes the
in-process ownership and documentation boundary that future secret-bearing
crypto APIs must use.

## Pentest Status

The first two `0.12.0` pentest passes have been resolved locally and the
submitted pentest handoff is green. Root `PENTEST.md` remains the temporary
findings handoff file and must be removed after findings are resolved.
