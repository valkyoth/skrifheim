# skrifheim 0.18.1 Release Notes

Status: implementation stop, pentest pending.

## Scope

`0.18.1` repairs the sovereignty overflow design gap from the earlier
query-result classification scaffold. Query-result sovereignty metadata now
uses a typed scope that stays exact while the bounded token set can represent
all jurisdictions, and saturates to a redacted multi-jurisdiction sentinel when
the result spans too many distinct sovereignty tokens.

The saturated state is intentionally not a grantable clearance token. It is a
most-restrictive marker for later export, placement, indexing, backup, AI
processing, and legal/compliance decisions.

## Changes

- Bumped workspace and internal crate dependency versions to `0.18.1`.
- Updated the pinned stable Rust toolchain from `1.96.0` to `1.96.1` after
  checking the official Rust release announcement.
- Added `SovereigntyScope` in `skrifheim-policy`.
- Kept exact sovereignty token propagation for up to
  `POLICY_TOKEN_SET_MAX_ITEMS` distinct sovereignty tokens.
- Added a saturated multi-jurisdiction sentinel for overflow instead of
  returning `InvalidSecurityToken`.
- Kept invalid sovereignty tokens fail-closed before saturation is accepted.
- Redacted `SovereigntyScope` debug output and added release-gate checks so it
  cannot derive sensitive `Debug`, `PartialEq`, or `Eq`.
- Ensured non-allow query plans still expose only public sentinel result
  metadata.

## Verification

- `cargo test -p skrifheim-policy -p skrifheim-query`
- `scripts/checks.sh`
- `scripts/release_0_18_1_gate.sh`

## Non-Claims

This release does not add legal/compliance passport enforcement, placement
planning, export authorization, backup policy enforcement, AI-processing
policy enforcement, or production query execution. The saturated sovereignty
scope is a scaffolded policy signal that future legal/compliance and operation
planners must treat as approval-required or deny.

## Pentest Status

This release is ready for the first `0.18.1` pentest pass after the
implementation stop commit. Root `PENTEST.md` must be used only as temporary
findings input and removed after findings are resolved.
