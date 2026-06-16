# skrifheim 0.12.0 Release Notes

Status: implementation stop, pending pentest.

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

The initial `0.12.0` pentest has not yet been run. Root `PENTEST.md` is the
temporary findings handoff file and must be removed after findings are
resolved.
