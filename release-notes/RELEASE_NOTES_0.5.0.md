# skrifheim 0.5.0 Release Notes

Status: implementation stop, pentest complete, pending GitHub verification.

## Scope

`0.5.0` makes world identity part of validated metadata. Worlds are now
deterministic, tenant-scoped branch overlays: root and child IDs derive from
validated world metadata instead of caller-provided identifiers.

## Changes

- Added `WorldMetadata` with ID, tenant ID, name, kind, parent pointer, and
  branch depth.
- Root worlds now derive deterministic IDs from tenant ID and validated root
  metadata.
- Forked worlds now inherit the parent tenant ID and derive deterministic IDs
  from tenant ID, parent ID, depth, name, and kind.
- Documented the world identity contract: root and fork creation are idempotent
  for the `(tenant_id, kind, depth, parent, name)` tuple. The future storage
  registry must enforce that tuple as the uniqueness key.
- Added `InvalidWorldIdentity` for invalid deterministic identity states.
- Kept added and hidden fact sets isolated per branch overlay.
- Added tests for repeated root identity, tenant separation, parent-sensitive
  child identity, kind separation, branch isolation, and direct-child diff
  validation.
- Bounded policy token sets and requested query-label lists to reduce
  algorithmic-complexity DoS risk.
- Made time ranges fail on inverted bounds at construction.
- Removed public confidence clamping so malformed external confidence values
  must fail validation instead of silently becoming maximum confidence.
- Replaced storage integrity zero sentinels with explicit presence fields.
- Added roadmap commitments for causal blast-radius invalidation, signed
  declassification proofs, capability-scoped AI derivation cones, and
  propagated confidence fused with mandatory access control.
- Bumped workspace and internal crate dependency versions to `0.5.0`.

## Verification

- `scripts/checks.sh`
- `scripts/release_0_5_gate.sh`
- `cargo deny check`
- `cargo audit`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `scripts/podman_smoke.sh`

## Non-Claims

This release is not a usable database engine. It does not provide durable
storage, networking, replication, cryptographic verification, production query
execution, merge/promotion preflight, or collision-resistant world identity.
The deterministic world ID derivation is tenant-scoped and stable, but remains
a scaffold, not a cryptographic content-addressing scheme.

## Pentest Status

Pentest findings and retest findings for the `0.5.0` implementation stop have
been resolved. Root `PENTEST.md` is absent. The permanent release handoff is
recorded in `security/pentest/v0.5.0.md`.
