# skrifheim 0.18.2 Release Notes

Status: first pentest pass and retest resolved locally.

## Scope

`0.18.2` separates key-material epoch identity from lifecycle event ordering.
The crypto epoch remains the authority marker used by WAL frames, immutable
segments, signatures, manifests, and recovery checks. Administrative lifecycle
changes that do not introduce new key material now advance a separate lifecycle
event sequence instead of forcing a crypto epoch bump.

This keeps compromise and quarantine operationally precise: declaring a key
compromised is a security event on the existing key material unless a rotation
or replacement actually introduces new material.

## Changes

- Bumped workspace and internal crate dependency versions to `0.18.2`.
- Updated the pinned stable Rust toolchain from `1.96.1` to `1.97.0` after
  checking the official Rust release announcement.
- Updated the admitted `sanitization` dependency from `1.2.2` to `1.2.4` after
  checking crates.io metadata.
- Added `KeyLifecycleEventSequence`.
- Added lifecycle-event ordering to `KeyMetadata`.
- Added lifecycle-event ordering to crypto-erasure metadata.
- Allowed compromise, quarantine, destruction, and crypto-erasure transitions
  to preserve the existing `CryptoEpoch` when no new key material is introduced.
- Kept created-to-active activation and active-to-rotating rotation transitions
  on strictly advancing `CryptoEpoch` values.
- Preserved fail-closed key hierarchy validation for compromised,
  quarantined, destroyed, and crypto-erased parent keys.
- Updated encryption architecture and security controls to describe the split
  between crypto-material epochs and lifecycle event ordering.
- Resolved the first pentest pass by removing public arbitrary key lifecycle
  reconstruction, computing crypto-erasure sequence once, binding segment kind
  into the footer, adding an in-memory segment read cap, enforcing exclusive
  WAL writers, rejecting bare new-file paths that cannot durably fsync an
  explicit parent directory, and making rejected WAL close records
  non-destructive.
- Resolved the retest by enforcing the same host in-memory body cap on segment
  writes and reads, staging immutable segment writes in the target directory
  before no-overwrite publication, validating explicit parent paths before file
  creation, fsyncing WAL parent directories for every successful writer open,
  parsing `v0.18.0`/`v0.18.1` footers as explicit legacy kind-unbound metadata,
  and pinning CI/container supply-chain inputs to reviewed versions or digests
  for both the runtime and Alpine smoke container definitions.

## Verification

- `cargo test -p skrifheim-crypto key_lifecycle`
- `cargo test -p skrifheim-storage segment`
- `cargo test -p skrifheim-storage replay`
- `cargo test -p skrifheim-storage-host`
- `cargo test -p skrifheim-storage-host segment`
- `cargo test -p skrifheim-storage-host wal`
- `scripts/checks.sh`
- `scripts/release_0_18_2_gate.sh` after pentest evidence is committed

## Non-Claims

This release does not add real key storage, HSM/KMS integration, threshold key
guardian workflows, storage-backed key-subtree revocation, persisted key
history, production AEAD, production digest authority, or recovery manifest
execution. Lifecycle event sequencing is a scaffolded ordering marker for key
metadata; durable audit/event storage remains planned work.

## Pentest Status

The first `0.18.2` pentest pass and retest have been resolved locally. Root
`PENTEST.md` has been removed after findings were resolved.
