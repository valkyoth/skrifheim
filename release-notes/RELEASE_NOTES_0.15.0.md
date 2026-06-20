# skrifheim 0.15.0 Release Notes

Status: implementation stop, pending pentest.

## Scope

`0.15.0` adds the first host-file WAL append/read helper crate while preserving
the `no_std` core storage model. It also starts the quantum-aware digest policy
work required before compact `WorldId` values can become durable storage
authority.

The WAL work is still pre-recovery. Frames can be appended and read back through
portable host file I/O, but the database still does not replay committed state
or recover after a crash.

## Changes

- Added `skrifheim-storage-host` outside `crates/` so host-only file I/O can use
  `std` without weakening the core crate `no_std` rule.
- Added `WalFileWriter` with append-only frame writes.
- Added `WalAppendOptions` with an explicit `sync_on_append` fsync boundary.
- Added `WalFileReader` for sequential frame reads.
- Added reader validation against the caller-provided expected WAL encryption
  domain.
- Added body-length mismatch rejection before writing a frame body.
- Added partial-header and partial-body detection for truncated WAL files.
- Added tests using temporary host files.
- Added `DigestStrength` profiles for `Sha3_256`, `Sha3_384`, `Sha3_512`,
  `Shake256_256`, and `Shake256_512`.
- Added `DigestPolicy` presets for high-security, long-horizon, and military
  profiles.
- Added full-width digest skeleton types: `WorldIdentityDigest`,
  `ContentDigest`, and `ManifestDigest`.
- Added release-gate checks that require the phrase "BLAKE3 remains
  scaffold-only" in release notes and
  require the full-width digest skeletons before durable storage authority.
- Bumped workspace and internal crate dependency versions to `0.15.0`.
- Added `scripts/release_0_15_gate.sh`.

## Verification

- `cargo test -p skrifheim-crypto`
- `cargo test -p skrifheim-storage-host`
- `scripts/checks.sh`
- `scripts/release_0_15_gate.sh`

## Non-Claims

This release does not implement real SHA-3/SHAKE hashing, admit a SHA-3/SHAKE
crate, compute content digests, replace existing scaffold BLAKE3 world ID
derivation, write durable manifests, replay WAL files, recover database state,
encrypt or decrypt WAL bodies, recompute WAL body CRCs, or enforce transaction
commit semantics. The new host WAL crate validates and moves already-encrypted
frame bytes; it does not make those bytes cryptographically trustworthy.

## Pentest Status

`0.15.0` is at an implementation stop and is ready for pentest.
