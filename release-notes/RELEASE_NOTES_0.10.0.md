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
- Extended the equality release gate to cover query-result inputs and query
  requests, including manual `PartialEq`/`Eq` implementations.
- Hardened manual equality blockers to catch path-qualified trait
  implementations such as `core::cmp::PartialEq`.
- Hardened sensitive derive blockers to catch multi-line and path-qualified
  derive attributes.
- Hardened release-gate impl-method scans with brace-depth tracking so later
  methods in sensitive impl blocks cannot bypass API exposure checks.
- Hardened the raw query-result input accessor gate so `pub const fn`
  variants cannot bypass the release check.
- Updated query request memory accounting to budget full result inputs instead
  of label metadata alone.
- Collapsed query requests to a single result-input vector so sensitive label
  state is not duplicated inside the request model.
- Normalized query request result-input vector capacity after validation so
  spare caller capacity is not retained.
- Closed planner decision and policy proof construction so non-allow proofs
  cannot be externally forged with sensitive result metadata.
- Enforced non-allow policy-proof masking inside the proof constructor so
  internal callers cannot attach sensitive result metadata to rejections or
  redactions.
- Replaced derived debug output on query result inputs, result classifications,
  planner decisions, policy proofs, query requests, and query plans with
  redacted diagnostic output.
- Redacted query intent from query request and query plan debug output while
  keeping explicit intent accessors for programmatic planning.
- Redacted planner decision state from planner decision, policy proof, and
  query plan debug output while keeping explicit decision accessors.
- Added release-gate checks that block direct query debug exposure of world
  identifiers, raw inputs, input counts, and proofs.
- Removed the public raw result-input accessor from query requests; callers get
  aggregate counts while planning keeps label/result metadata internal.
- Removed public raw metadata accessors from query-result inputs; policy
  evaluation keeps crate-local access for planning.
- Replaced derived debug output on subject, device, workload, and aggregate
  authority contexts with redacted diagnostics.
- Made the label-only aggregate read helper fail closed for empty or oversized
  label sets, matching the 0.10.0 result-input path.
- Masked invalid aggregate label-set proof counts to a public zero sentinel
  instead of preserving exact oversized request lengths.
- Redacted exact query/proof input counts from debug output while keeping
  explicit count accessors for programmatic policy use.
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
