# skrifheim 0.18.1 Release Notes

Status: pentest findings resolved locally, retest pending.

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
- Replaced bare boolean sovereignty containment with explicit
  present/absent/indeterminate containment so saturated scopes cannot be
  silently treated as absence.
- Added an upfront sovereignty-scope input bound before token scanning.
- Redacted `SovereigntyScope` debug output and added release-gate checks so it
  cannot derive sensitive `Debug`, `PartialEq`, or `Eq`.
- Ensured non-allow query plans still expose only public sentinel result
  metadata.
- Failed closed on unsupported non-Unix `skrifheim-storage-host` targets until
  hardened platform-specific file controls exist.
- Required break-glass audit events to use expiring device and workload
  attestation evidence.
- Rejected oversized world fact batches before sorting.
- Documented that `SecretBytes::with_secret` and `try_with_secret` closures
  must not retain secret-derived bytes.
- Resolved retest feedback by keeping overlapping exact sovereignty scopes
  exact when their true unique union still fits the bounded set.
- Fixed the `0.18.1` release gate self-check for the
  `SovereigntyScope::contains` ban.

## Verification

- `cargo test -p skrifheim-policy -p skrifheim-query -p skrifheim-audit -p skrifheim-world -p skrifheim-crypto`
- `scripts/checks.sh`
- `scripts/release_0_18_1_gate.sh`

## Non-Claims

This release does not add legal/compliance passport enforcement, placement
planning, export authorization, backup policy enforcement, AI-processing
policy enforcement, production query execution, Windows host-file storage
hardening, one-time break-glass state tracking, or key-subtree revocation
execution. The saturated sovereignty scope is a scaffolded policy signal that
future legal/compliance and operation planners must treat as
approval-required or deny.

## Pentest Status

The first `0.18.1` pentest pass and retest feedback have been resolved
locally. Root `PENTEST.md` has been removed after findings were resolved.

Residual planned boundary: break-glass single-use evidence binding requires
persisted one-time-use state in the `v0.26.1` access model, and key-subtree
revocation requires the storage-backed key registry/control plane before real
key-use paths land.
