# Changelog

## Unreleased

- Added roadmap commitments for causal blast-radius invalidation, signed declassification proofs, capability-scoped AI derivation cones, and propagated confidence fused with mandatory access control.

## 0.5.0

- Added deterministic world metadata identities and branch-depth tracking.
- Added branch isolation tests for world overlays.
- Resolved 0.5.0 pentest findings for bounded policy/query inputs and stricter invariants.
- Resolved 0.5.0 retest findings by making deterministic world identity tenant-scoped and documenting idempotent root/fork semantics.
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
