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
- Admitted `subtle` `2.6.1` with default features disabled for policy-token
  equality in authorization paths.
- Redacted `FactBuilder` actor attribution and policy identifiers in Debug
  output.
- Created Unix WAL files with owner-only `0o600` mode.
- Removed the impossible missing-CRC state from `WalFrameHeader` storage so
  encoding cannot silently emit a zeroed header for `BodyChecksum::Missing`.
- Reaffirmed that signature envelopes and audit/fact signature sets are still
  format-validated only, not cryptographically verified.
- Added explicit `subtle`-backed digest equality helpers and documented
  structural digest comparison as non-authentication-only.
- Tightened permissions on pre-existing Unix WAL files when opened for append.
- Removed redundant host-file flushes before `sync_all`.
- Made policy-token length comparison use a fixed-width integer before
  constant-time comparison.
- Simplified the WAL body CRC getter to return the guaranteed-present raw CRC
  value.
- Made digest constant-time comparison expansion use a fixed 64-byte loop and
  documented that production still requires statistical timing evidence.
- Narrowed policy-token length comparison to a guarded `u8` representation.
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
commit semantics. It does not provide dudect-style statistical timing evidence
or codegen proof for authorization paths, even though policy-token equality now
uses an admitted constant-time primitive. It also does not cryptographically
verify fact or audit signatures. The new host WAL crate validates and moves
already-encrypted frame bytes; it does not make those bytes cryptographically
trustworthy.

## Pentest Status

The first, second, and third `0.15.0` pentest passes have been resolved locally.
Root `PENTEST.md` remains the temporary findings handoff file and must be
removed after findings are resolved.
