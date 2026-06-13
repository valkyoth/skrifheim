# skrifheim 0.6.0 Release Notes

Status: implementation stop, pending pentest.

## Scope

`0.6.0` adds deterministic world diff, promotion preflight, rollback preflight,
and explicit conflict categories for the current world-overlay scaffold.

## Changes

- Added a dedicated `skrifheim-world` diff module.
- Kept world diff deterministic by collecting fact deltas through ordered sets.
- Added `PromotionPreflight` with `can_promote()`.
- Added `RollbackPreflight` with `can_rollback()`.
- Added conflict reporting for facts added and hidden in the same child overlay.
- Added conflict reporting for facts reintroduced after the parent world hid
  them.
- Added `World::diff_to_child`, `World::promotion_preflight`, and
  `World::rollback_preflight`.
- Added an explicit tenant consistency check to world diff/preflight boundaries.
- Documented that promotion and rollback preflight are parent-vs-child scaffold
  checks and do not yet validate full transitive ancestry.
- Added tests for clean promotion, non-child promotion rejection, conflict
  detection, and rollback inverse-delta reporting.
- Bounded variable-length `Named` and `HybridClassicalPq` signature payloads to
  reduce resource-exhaustion risk on untrusted signatures.
- Bounded fact evidence lists with the same ceiling used for causal fact links.
- Documented `WorldId` as a deterministic non-secret namespacing key, not an
  access-control capability.
- Added `SkrifheimError::public_message()` so external/API boundaries can avoid
  leaking internal storage-header validation detail.
- Bumped workspace and internal crate dependency versions to `0.6.0`.
- Re-checked the stable Rust channel on 2026-06-13. Rust stable remains
  `1.96.0`, dated 2026-05-28 in the official stable manifest.

## Verification

- `cargo check --workspace`
- `cargo test -p skrifheim-world`
- `scripts/checks.sh`
- `scripts/release_0_6_gate.sh`
- `cargo deny check`
- `cargo audit`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `scripts/podman_smoke.sh`

## Non-Claims

This release is not a usable database engine. It does not provide durable
storage, merge execution, promotion execution, rollback execution, transitive
ancestor validation, cryptographic verification, or production conflict
resolution. The preflight model reports deterministic parent-vs-child scaffold
conflicts before storage-backed world history exists.

## Pentest Status

Pentest is required before tagging. Root `PENTEST.md` is the temporary findings
handoff file and must be removed after findings are resolved.
