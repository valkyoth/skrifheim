# skrifheim 0.17.0 Release Notes

Status: release-prep stop, local release gate passed, GitHub verification pending.

## Scope

`0.17.0` defines the immutable segment format scaffold. It adds
policy-scoped segment headers and footers that bind transaction range, policy
identifier, encryption key, crypto epoch, encryption domain, body length,
CRC64 presence, and algorithm-agile content digest metadata.

This release does not yet write or read segment files. The segment reader and
writer begin in `0.18.0`; this release establishes the validation model they
must use.

## Changes

- Moved segment metadata out of the storage crate root into a dedicated
  `skrifheim-storage` segment module.
- Added `SegmentFooter` and `SegmentFooterInput` with private fields,
  validating constructors, redacted Debug output, and accessors.
- Added `SegmentFooter::from_header` for constructing matching footer metadata
  from a validated header.
- Added `SegmentFooter::validate_against_header` to reject mismatched tenant,
  transaction range, policy, encryption key, crypto epoch, encryption domain,
  body length, body CRC, or content digest metadata.
- Added crypto epoch and segment encryption-domain fields to
  `SegmentHeader` and `SegmentHeaderInput`.
- Required segment crypto epochs to be non-zero.
- Required segment encryption domains to use segment purpose and match the
  segment tenant.
- Required segment encryption domains to carry classification, compartment,
  and segment identifiers.
- Kept the explicit CRC64 presence model and continued rejecting missing CRCs
  and CRC value `0` as missing-integrity sentinels.
- Continued rejecting all-zero content digests.
- Added constant-width structural matching for header/footer encryption-domain
  and content-digest comparisons.
- Extended the security-policy validation script so segment headers, footers,
  and inputs cannot derive sensitive Debug, PartialEq, or Eq behavior and keep
  their internal fields private.
- Bumped workspace and internal crate dependency versions to `0.17.0`.
- Added `scripts/release_0_17_gate.sh`.
- Resolved the first `0.17.0` pentest pass by failing closed on unknown Unix
  `O_NOFOLLOW` WAL file targets, using constant-time encryption-domain
  comparison in `WalFrameHeader::validate_for_domain`, checking host WAL body
  length conversion before allocation, renaming fact validation to explicit
  structural validation, adding storage-validation guards to world promotion
  and rollback preflights, switching WAL CRC64 to a compile-time table-based
  ECMA-182 implementation, rejecting WAL crypto epoch regressions across
  transaction boundaries, rejecting unsynchronized audit timestamps, and
  tracking production timing evidence as a planned release gate.

## Verification

- `cargo test -p skrifheim-storage`
- `scripts/checks.sh`
- `scripts/release_0_17_gate.sh`

## Non-Claims

This release does not add a segment writer, segment reader, segment encoding
layout, durable manifest, checkpoint root, compaction, encrypted body
authentication, production digest implementation, or recovery from immutable
segments. Header and footer validation is structural metadata validation only;
future readers must recompute and verify body CRCs and content digests over
segment bytes before accepting stored facts.

## Pentest Status

The first `0.17.0` pentest pass has been resolved locally. Root `PENTEST.md`
has been removed after findings were resolved.

Retest reported no new issues. Two residuals remain intentionally tracked:

- `DigestValue::structurally_equal_ct` remains scaffold-only until the planned
  production timing evidence gate adds dudect-style or equivalent statistical
  evidence.
- Promotion and rollback preflight ancestry validation is fail-closed and
  partial by design: external callers cannot mark a preflight storage-validated
  yet, so `require_storage_validated()` always rejects until storage-backed
  ancestry traversal is implemented.

The local release gate passed after this retest update. This release is waiting
for GitHub verification before signed tag creation.
