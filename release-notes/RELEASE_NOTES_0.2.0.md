# skrifheim 0.2.0 Release Notes

Status: implementation stop, pending pentest.

## Scope

`0.2.0` adds the first validated fact construction API. Canonical facts can now
be assembled through `FactBuilder`, which validates required fields and core
fact invariants before returning a `Fact`.

## Changes

- Added `Fact::builder()` and `FactBuilder`.
- Added required-field validation for fact identity, world, subject, predicate,
  object, valid time, commit transaction, asserting actor, policy, label, and
  signatures.
- Builder output validates evidence, valid-time ranges, signature sets, and
  self-referential causal links.
- Added clamped-confidence builder support.
- Added negative tests for missing required fields, missing evidence, invalid
  valid time, empty signatures, and self-referential causality.
- Bumped workspace and internal crate dependency versions to `0.2.0`.

## Pentest Remediation

- `Fact` fields are private with read-only accessors; validated facts cannot be
  reclassified, stripped of evidence, or stripped of signatures by external
  field mutation.
- `QueryPlan` decisions are private with read-only accessors; callers cannot
  clear rejected/redacted decisions after planning.
- Signature algorithm names are validated before signature envelopes are
  accepted in signing contexts.
- `SignatureSet` uses a validated constructor and read-only envelope access.
- Policy-denial errors use a non-leaking reason type instead of arbitrary
  strings.
- Policy compartment and releasability loops evaluate all labels before
  deciding reject or redact.
- Fact evidence and causal-link inputs are deduplicated.
- `World` structural fields are private; root/fork construction and fact
  mutation go through invariant-preserving methods.

## Verification

- `scripts/checks.sh`
- `scripts/release_0_2_gate.sh`
- `cargo deny check`
- `cargo audit`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `scripts/podman_smoke.sh`

## Non-Claims

This release is not a usable database engine. It does not provide durable
storage, networking, replication, cryptographic verification, or production
query execution.

## Pentest Status

Pentest is required before tagging. Root `PENTEST.md` is the temporary findings
handoff file and must be removed after findings are resolved.
