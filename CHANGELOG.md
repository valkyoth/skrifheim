# Changelog

## Unreleased

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
