# Changelog

## Unreleased

## 0.10.0

- Added query-result classification metadata for sovereignty propagation,
  PII-derived output marking, AI-processing eligibility, and confidence
  threshold policy hooks.
- Query plans now preserve full result-classification metadata for allowed
  plans while masking non-allow proofs to a public, non-PII, AI-eligible
  sentinel.
- Added explicit policy-layer result input count bounds and tests for combined
  sovereignty overflow.
- Hardened internal result-classification derivation with its own fail-closed
  input count guard.
- Added redaction-safe policy-token union support for sovereignty propagation.
- Removed public equality from planner proof/result surfaces and extended the
  security gate to keep sensitive result metadata out of equality comparisons.
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
- Updated release documentation to record the resolved 0.10.0 pentest handoff
  state and local release-gate status.
- Resolved the first `0.10.0` pentest pass by hardening policy-token union
  deduplication, preserving canonical union order, redacting signature and
  security-label diagnostics, masking non-allow proof input counts, preserving
  redact/reject distinction for trusted errors, and documenting remaining
  memory-secrecy and production constant-time admission requirements.
- Resolved the second `0.10.0` pentest pass by redacting fact classification
  diagnostics, redacting policy-token set counts, adding release-gate checks for
  both regressions, and documenting the scaffold-only non-constant-time union
  sort comparator.
- Prepared `0.10.0` for GitHub verification after the local release gate and
  submitted pentest handoffs passed.
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
- Bumped workspace and internal crate dependency versions to `0.10.0`.
- Added `0.10.0` release metadata and gate script.
- Re-checked the local Rust toolchain on 2026-06-16. Rust and Cargo report
  `1.96.0`; the workspace has no external crate dependencies beyond local
  workspace crates.

## 0.9.0

- Added encryption-domain metadata for tenant, region, classification,
  compartment, world, WAL, segment, projection, backup, export capsule, AI
  artifact, WASM/plugin secret, and audit-log boundaries.
- Added exact encryption-domain merge compatibility checks so incompatible
  blast-radius boundaries cannot be silently combined.
- Added tests for cross-region, cross-compartment, cross-world, cross-segment,
  and special-purpose domain incompatibility.
- Resolved the first `0.9.0` pentest pass by replacing world diff/preflight
  set allocation with sorted-slice scans and documenting the cost posture.
- Resolved the second `0.9.0` pentest pass by replacing the storage segment
  header's raw encryption key integer with a typed non-zero `KeyId`.
- Resolved the third `0.9.0` pentest pass by replacing an unreachable
  policy-token bounds branch with explicit invariant debug assertions.
- Resolved the fourth `0.9.0` pentest pass by making fact-builder bulk
  provenance setters merge with existing links and documenting `Fact::validate`
  as the authoritative construction gate.
- Resolved the fifth `0.9.0` pentest pass by rejecting duplicate signature
  signer entries, enforcing the variable signature byte ceiling, and making
  crypto epochs opaque behind constructor/accessor methods.
- Bumped workspace and internal crate dependency versions to `0.9.0`.
- Added `0.9.0` release metadata and gate script.
- Re-checked the stable Rust channel on 2026-06-15. Rust stable remains
  `1.96.0`; `rustup` has an available tooling update from `1.28.2` to
  `1.29.0`.

## 0.8.0

- Added key lifecycle states for creation, activation, rotation, retirement, compromise, quarantine, destruction, and crypto-erasure.
- Added key lifecycle transition validation and invalid-transition tests.
- Added rotation preflight metadata for active-to-candidate key rotation.
- Added crypto-erasure metadata records and tests for compromise/quarantine/destruction handling.
- Resolved the first `0.8.0` pentest pass by making policy-token slot access
  non-panicking, gating the test signature algorithm out of non-test builds,
  enforcing hybrid signature minimum lengths, and documenting external error
  and traversal guardrails.
- Resolved the second `0.8.0` pentest pass by redacting sensitive Debug output,
  requiring key destruction before crypto-erasure, and tracking raw key material
  memory hygiene requirements.
- Resolved the third `0.8.0` pentest pass by expanding compromise declaration
  paths, redacting signature Debug output, removing the fixed world-ID fallback,
  bounding segment body length, and documenting storage/key lifecycle binding.
- Resolved the fourth `0.8.0` pentest pass by removing derived equality from
  sensitive labels, policy tokens, and signature payloads; rejecting unsafe key
  parents; bounding and sorting world fact tracking; and documenting the
  reviewed low/informational notes.
- Resolved the fifth `0.8.0` pentest pass by enforcing exact fixed-size
  hybrid/named signatures, binding segment/data keys to compartment keys, and
  documenting token canonicalization timing and tenant-level world quotas.
- Resolved the sixth `0.8.0` pentest pass by rejecting empty query label sets,
  rejecting zero-width time ranges, and documenting that segment header
  validation is structural until body integrity verification is implemented.
- Bumped workspace and internal crate dependency versions to `0.8.0`.
- Added `0.8.0` release metadata and gate script.

## 0.7.0

- Added key hierarchy metadata for root trust, deployment, region, tenant, compartment, segment, and data key scopes.
- Added parent/child key hierarchy validation, including tenant deployment/region binding and data-key edge tests.
- Hardened policy-token comparison with local compiler barriers and fixed-slot bounded token-set scans.
- Hardened policy label evaluation to process bounded compartment and releasability slots.
- Added maximum signature count validation for `SignatureSet`.
- Added signature key ID length and character validation.
- Added fact object payload size validation for text and byte values.
- Removed per-label query decisions from the public query plan surface.
- Added constructor-enforced query-label limits and explicit query-label memory budgeting.
- Added closed allow-list validation for named and hybrid signature algorithms.
- Enabled release-profile overflow checks.
- Changed segment CRC metadata so an explicit CRC64 value of zero is representable.
- Switched fact-builder evidence and causal-link deduplication to sort/dedup behavior with fail-fast link bounds in builder methods.
- Added `0.7.0` release metadata and gate script.

## 0.6.0

- Added deterministic world diff preflight types for promotion and rollback checks.
- Added world conflict categories for facts added and hidden in the same overlay and facts reintroduced after parent hiding.
- Added `0.6.0` release metadata and gate script.

## 0.5.0

- Added deterministic world metadata identities and branch-depth tracking.
- Added branch isolation tests for world overlays.
- Resolved 0.5.0 pentest findings for bounded policy/query inputs and stricter invariants.
- Resolved 0.5.0 retest findings by making deterministic world identity tenant-scoped and documenting idempotent root/fork semantics.
- Added roadmap commitments for causal blast-radius invalidation, signed declassification proofs, capability-scoped AI derivation cones, and propagated confidence fused with mandatory access control.
- Added `0.5.0` release metadata and gate script.

## 0.4.0

- Added aggregate policy proof skeletons and output classification calculation.
- Resolved 0.4.0 pentest findings for non-allow proof disclosure and panic-free crypto validation.
- Added `0.4.0` release metadata and gate script.

## 0.3.0

- Added subject/device/workload authority context for policy checks.
- Added `0.3.0` release metadata and gate script.

## 0.2.0

- Added validated fact construction through `FactBuilder`.
- Added `0.2.0` release metadata and gate script.

## 0.1.0

- Initialized the `skrifheim` Rust workspace.
- Added focused crates for core types, facts, worlds, policy, crypto envelopes, storage metadata, query planning, and the main crate.
- Added security, modularity, toolchain, implementation, version, and CMS target documentation.
- Added local validation scripts and a rootless Podman smoke path.
