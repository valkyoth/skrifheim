# skrifheim 0.18.2 Release Notes

Status: implementation stop, pentest pending.

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

## Verification

- `cargo test -p skrifheim-crypto key_lifecycle`
- `scripts/checks.sh`
- `scripts/release_0_18_2_gate.sh` after pentest evidence is committed

## Non-Claims

This release does not add real key storage, HSM/KMS integration, threshold key
guardian workflows, storage-backed key-subtree revocation, persisted key
history, production AEAD, production digest authority, or recovery manifest
execution. Lifecycle event sequencing is a scaffolded ordering marker for key
metadata; durable audit/event storage remains planned work.

## Pentest Status

`v0.18.2 implementation stop reached. Run pentest for this exact commit.`
