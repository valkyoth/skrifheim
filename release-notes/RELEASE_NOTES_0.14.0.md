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
- Hardened policy-token slot construction so an invariant violation cannot
  panic through out-of-bounds indexing.
- Required key lifecycle transitions to advance crypto epochs strictly.
- Removed the `Rotating` to `Active` key lifecycle transition so a rotating key
  cannot re-enter active state and create a second-active-key ambiguity.
- Changed empty signature bytes to return `InvalidSignatureLength` instead of
  `EmptySignatureSet`.
- Rejected zero WAL frame body CRC values and zero CRC values decoded from WAL
  frame bytes.
- Added `parse_for_domain` and `validate_for_domain` so future WAL readers can
  bind parsed region/world metadata to the expected WAL segment location.
- Added `AuditEvent::new_at` for trusted callers that can provide a clock
  reference and reject future-dated audit events.
- Made `AuditEvent::new` require a trusted clock reference too, so the shortest
  audit-event constructor is also the clock-aware constructor.
- Added an audit-event maximum lookback window to reject excessively backdated
  events.
- Removed the misleading unreachable `BodyChecksum::Missing` encoding arm from
  WAL frame header encoding.
- Reconfirmed BLAKE3 is admitted only for deterministic, non-secret world ID
  derivation and remains rejected in signature-envelope contexts.
- Confirmed production constant-time evidence, real cryptographic operations,
  `SecretBytes::from_slice` source cleanup, and `WorldId` storage uniqueness
  enforcement remain documented scaffold or future-service-boundary items.
- Bumped workspace and internal crate dependency versions to `0.14.0`.
- Added `scripts/release_0_14_gate.sh`.

## Verification

- `cargo test -p skrifheim-storage`
- `scripts/checks.sh`
- `scripts/release_0_14_gate.sh`

## Non-Claims

This release does not write WAL files, read WAL files from disk, encrypt or
decrypt frame bodies, compute or verify body checksums, fsync data, detect
partial writes, replay transactions, or recover database state. It also does
not provide production constant-time evidence for policy-token comparison or
real signature verification, key derivation, encryption, decryption, HSM/KMS
binding, segment content-hash verification, or WAL body-CRC recomputation. It
only defines and validates the fixed frame metadata that later WAL writer,
reader, and replay milestones must use.

## Pentest Status

The first and second `0.14.0` pentest passes have been resolved locally. Root
`PENTEST.md` remains the temporary findings handoff file and must be removed
after findings are resolved.
