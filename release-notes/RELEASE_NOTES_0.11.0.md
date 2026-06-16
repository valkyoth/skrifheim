# skrifheim 0.11.0 Release Notes

Status: implementation complete, pending pentest.

## Scope

`0.11.0` makes index and projection encryption policy explicit before durable
indexes or projection builders exist. The release adds policy primitives that
future secondary indexes, graph/search/vector indexes, columnar projections,
caches, and compaction temporary files must use when choosing encryption
domains.

## Changes

- Added `ProjectionSurface` for secondary indexes, graph indexes, search
  indexes, vector indexes, columnar projections, cache files, and compaction
  temporary files.
- Added `ProjectionEncryptionPolicy`, which requires every index/projection
  surface to use an `EncryptionDomainPurpose::Projection` domain.
- Added explicit encrypted-at-rest and no-plaintext-temporary-file policy hooks
  for projection and compaction surfaces.
- Added domain compatibility checks so projection builders can reject
  cross-compartment or otherwise incompatible encryption-domain mixing.
- Added tests for encrypted secondary-index, graph, search, vector, and
  columnar projection surfaces.
- Added tests that reject non-projection domains and cross-compartment
  projection mixing.
- Added tests that compaction temporary projection files require encryption and
  disallow plaintext temporary files.
- Length-separated scaffold world ID hash fields to remove field-boundary
  ambiguity while keeping collision-resistant world ID derivation planned before
  storage-key authority.
- Replaced derived debug output on encryption domains and projection encryption
  policies with redacted diagnostics.
- Removed derived equality from encryption domains and projection encryption
  policies; structural comparison is now explicit and documented as
  non-constant-time.
- Projection encryption policies now require a classified projection domain.
- Policy-token shape checks in policy evaluation now fail closed in release
  builds instead of relying on `debug_assert!`.
- `Timestamp` now has a private field with explicit `new()` and `get()`
  methods; zero remains a documented valid timestamp.
- Added release-gate checks for sensitive encryption-domain/projection-policy
  derives, direct domain debug exposure, public timestamp tuple fields, and
  `debug_assert!` use in policy decisions.
- Aligned projection policy compatibility checks with merge semantics so
  matching domains are not treated as compatible when projection surfaces
  differ.
- Redacted direct `ProjectionSurface` debug output so the public `surface()`
  accessor cannot bypass `ProjectionEncryptionPolicy` diagnostic redaction.
- Strengthened release-gate redaction checks for encryption-domain and
  projection-policy debug implementations.
- Bumped workspace and internal crate dependency versions to `0.11.0`.
- Added `scripts/release_0_11_gate.sh`.

## Verification

- `cargo test -p skrifheim-crypto`
- `scripts/checks.sh`
- `scripts/release_0_11_gate.sh`

## Non-Claims

This release does not build indexes, persist projections, encrypt files on
disk, implement compaction, or execute projection workers. It records the
policy contract that those future systems must satisfy.

## Pentest Status

The initial `0.11.0` pentest has not yet been run. Root `PENTEST.md` is the
temporary findings handoff file and must be removed after findings are
resolved.
