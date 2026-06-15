# skrifheim 0.9.0 Release Notes

Status: implementation stop complete, pending pentest.

## Scope

`0.9.0` adds encryption-domain metadata on top of the `0.8.0` key lifecycle
scaffold. It defines the blast-radius boundaries future encrypted storage,
projection, export, AI, plugin, backup, WAL, segment, and audit surfaces must
respect.

## Changes

- Added `EncryptionDomainPurpose` for tenant, region, classification,
  compartment, world, WAL, segment, projection, backup, export capsule, AI
  artifact, WASM/plugin secret, and audit-log domains.
- Added `EncryptionDomain` metadata with tenant, region, classification,
  compartment, world/branch, and segment boundary fields.
- Added purpose-specific constructors for the required domain shapes.
- Added exact merge compatibility checks so domains can only merge when every
  modeled boundary matches.
- Added tests that reject cross-region, cross-compartment, cross-world,
  cross-segment, and cross-purpose domain mixing.
- Bumped workspace and internal crate dependency versions to `0.9.0`.
- Added `scripts/release_0_9_gate.sh`.
- Re-checked the stable Rust channel on 2026-06-15. Rust stable remains
  `1.96.0`; `rustup` reports a tooling update from `1.28.2` to `1.29.0`.

## Verification

- `cargo test -p skrifheim-crypto`
- `scripts/checks.sh`
- `scripts/release_0_9_gate.sh`

## Non-Claims

This release is not a usable database engine. It does not encrypt, decrypt,
derive, store, rotate, recover, erase, or verify real key material. Encryption
domains are metadata only and do not yet perform durable storage isolation,
cryptographic key derivation, or runtime access enforcement.

## Pentest Status

No `0.9.0` pentest findings have been applied yet. Root `PENTEST.md` is the
temporary findings handoff file and must be removed after findings are
resolved.
