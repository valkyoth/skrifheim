# skrifheim 0.4.0 Release Notes

Status: implementation stop, pending pentest.

## Scope

`0.4.0` adds the first deterministic policy-decision proof skeleton. Query
planning now records aggregate policy proof metadata and output classification
for requested labels.

## Changes

- Added `DecisionKind` for allow, redact, and reject outcomes.
- Added `PolicyProof` with decision, input-label count, and output
  classification.
- Added output classification calculation across requested labels.
- Added aggregate `evaluate_read_set` policy evaluation.
- Query plans now expose aggregate proof and output classification.
- Kept denial reasons constant-shape through `AccessDeniedReason`.
- Non-allow proofs use a non-disclosing output classification sentinel.
- Removed panic-based unreachable signature validation paths.
- Added tests for dangerous joins, redaction, rejection, and proof metadata.
- Bumped workspace and internal crate dependency versions to `0.4.0`.

## Verification

- `scripts/checks.sh`
- `scripts/release_0_4_gate.sh`
- `cargo deny check`
- `cargo audit`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `scripts/podman_smoke.sh`

## Non-Claims

This release is not a usable database engine. It does not provide durable
storage, networking, replication, cryptographic verification, production query
execution, or formal policy proofs.

## Pentest Status

Pentest is required before tagging. Root `PENTEST.md` is the temporary findings
handoff file and must be removed after findings are resolved.
