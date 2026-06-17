# skrifheim 0.14.0 Release Notes

Status: implementation stop, pending pentest.

## Scope

`0.14.0` defines the first append-only WAL frame header format before file I/O,
writers, readers, replay, or recovery exist. The release keeps WAL work at the
metadata and parser-validation layer: frames carry record kind, transaction,
key epoch, encryption key, encryption domain, encrypted body length, and body
checksum metadata.

## Changes

- Added `WalRecordKind` for transaction begin, fact batch, transaction commit,
  transaction abort, and checkpoint records.
- Added `WalFrameHeader` and `WalFrameHeaderInput`.
- Added fixed-size v1 WAL frame header encoding and parsing.
- Added WAL frame magic, version, header-size, and encrypted-body-size limits.
- Required WAL frame encrypted body length to be non-zero and bounded.
- Required WAL frame body CRC metadata to be present.
- Required WAL frame encryption domains to use
  `EncryptionDomainPurpose::Wal`.
- Required WAL frame tenant metadata to match the WAL encryption domain tenant.
- Added malformed parser tests for bad magic, unknown record kind, non-zero
  reserved bytes, and truncated headers.
- Added redacted diagnostics for WAL frame headers and inputs.
- Added `InvalidWalFrame` with a generic trust-boundary public message.
- Bumped workspace and internal crate dependency versions to `0.14.0`.
- Added `scripts/release_0_14_gate.sh`.

## Verification

- `cargo test -p skrifheim-storage`
- `scripts/checks.sh`
- `scripts/release_0_14_gate.sh`

## Non-Claims

This release does not write WAL files, read WAL files from disk, encrypt or
decrypt frame bodies, compute or verify body checksums, fsync data, detect
partial writes, replay transactions, or recover database state. It only defines
and validates the fixed frame metadata that later WAL writer, reader, and
replay milestones must use.

## Pentest Status

`0.14.0` is at an implementation stop and is ready for pentest. Root
`PENTEST.md` remains the temporary findings handoff file and must be removed
after findings are resolved.
