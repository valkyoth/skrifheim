# skrifheim 0.10.0 Release Notes

Status: implementation complete, pending pentest.

## Scope

`0.10.0` adds query-result classification metadata before query execution
exists. The release keeps the existing aggregate decision model, but proofs for
allowed query plans now carry the metadata future execution, indexing, export,
AI processing, and compliance layers must honor.

## Changes

- Added `QueryResultInput` for result-classification planning inputs.
- Added `ResultClassification` for output classification, sovereignty tokens,
  PII-derived output marking, AI-processing eligibility, and confidence
  threshold policy hooks.
- Added `ConfidenceThreshold`, bounded to `0..=1000`, for future propagated
  confidence checks.
- Added redaction-safe `PolicyTokenSet::union()` for sovereignty propagation
  without exposing token strings.
- Added `evaluate_read_result_set()` so policy evaluation can derive result
  metadata only after all labels are allowed.
- Added an explicit policy-layer result input count bound and overflow tests for
  combined sovereignty propagation.
- Hardened internal result-classification derivation with its own fail-closed
  input count guard.
- Removed public equality from planner proof/result surfaces and extended the
  security gate so sensitive result metadata cannot be compared through
  `PartialEq`/`Eq`.
- Updated query request memory accounting to budget full result inputs instead
  of label metadata alone.
- Collapsed query requests to a single result-input vector so sensitive label
  state is not duplicated inside the request model.
- Normalized query request result-input vector capacity after validation so
  spare caller capacity is not retained.
- Kept `evaluate_read_set()` and `output_classification()` compatibility
  accessors for existing label-only planning.
- Updated query planning to accept richer result inputs while keeping the
  label-only constructor path.
- Masked rejected and redacted query plans to public, no-PII, AI-eligible,
  no-sovereignty result metadata.
- Bumped workspace and internal crate dependency versions to `0.10.0`.
- Added `scripts/release_0_10_gate.sh`.
- Re-checked the local Rust toolchain on 2026-06-16. Rust and Cargo report
  `1.96.0`; no external crate dependencies are present beyond local workspace
  crates.

## Verification

- `cargo test -p skrifheim-policy`
- `cargo test -p skrifheim-query`
- `cargo test -p skrifheim-core`
- `scripts/checks.sh`
- `scripts/release_0_10_gate.sh`

## Non-Claims

This release is not a usable database engine. It does not execute queries,
persist results, enforce runtime export paths, perform AI processing, or compute
Bayesian confidence. The new result-classification model is policy metadata for
future query execution and compliance enforcement.

## Pentest Status

The initial `0.10.0` pentest has not yet been run. Root `PENTEST.md` is the
temporary findings handoff file and must be removed after findings are
resolved.
