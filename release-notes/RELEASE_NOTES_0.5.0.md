# skrifheim 0.5.0 Release Notes

Status: implementation stop, pending pentest.

## Scope

`0.5.0` makes world identity part of validated metadata. Worlds are now
deterministic branch overlays: root and child IDs derive from validated world
metadata instead of caller-provided identifiers.

## Changes

- Added `WorldMetadata` with ID, name, kind, parent pointer, and branch depth.
- Root worlds now derive deterministic IDs from validated root metadata.
- Forked worlds now derive deterministic IDs from parent ID, depth, name, and
  kind.
- Added `InvalidWorldIdentity` for invalid deterministic identity states.
- Kept added and hidden fact sets isolated per branch overlay.
- Added tests for repeated root identity, parent-sensitive child identity, kind
  separation, branch isolation, and direct-child diff validation.
- Bounded policy token sets and requested query-label lists to reduce
  algorithmic-complexity DoS risk.
- Made time ranges fail on inverted bounds at construction.
- Removed public confidence clamping so malformed external confidence values
  must fail validation instead of silently becoming maximum confidence.
- Replaced storage integrity zero sentinels with explicit presence fields.
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
The deterministic world ID derivation is a stable scaffold, not a cryptographic
content-addressing scheme.

## Pentest Status

Pentest is required before tagging. Root `PENTEST.md` is the temporary findings
handoff file and must be removed after findings are resolved.
