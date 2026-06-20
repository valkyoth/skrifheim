# skrifheim Engineering Policy

Status: hard rule

`skrifheim` is a military-security-oriented database project. Convenience is not enough reason to add `std`, unsafe code, or third-party dependencies to the trusted core.

## Hard Rules

Crates under `crates/` are core database crates. Their library targets must:

- use `#![no_std]`,
- use `#![forbid(unsafe_code)]`,
- avoid `std` imports entirely,
- prefer `alloc` only where owned dynamic data is necessary,
- prefer `skrifheim`-owned primitives for security-critical behavior,
- avoid external dependencies unless the reason is documented before use.

Host-only code may use `std`:

- `crates/skrifheim/src/main.rs`,
- `tools/xtask`,
- shell scripts,
- future fuzz, release, and test-only tools.

Host-only code still follows the dependency review rule.

## Build Our Own By Default

`skrifheim` should own its security-critical primitives:

- fact identity and validation,
- world DAG and merge semantics,
- classification and compartment checks,
- policy planner decisions,
- storage frame formats,
- WAL and recovery state machine,
- query language parser and planner,
- manifest and audit-proof formats,
- crypto-agile envelope metadata,
- CMS release/dependency primitives.

External crates can be safer for some standards or host-only tooling, but they must not quietly import a different authority model, runtime model, parser behavior, allocator assumption, or unsafe trusted boundary.

## External Dependency Admission

Before adding any external crate:

1. Discuss why local implementation is not the better option.
2. Check the latest crate version.
3. Review license compatibility with EUPL-1.2.
4. Review maintenance, unsafe usage, transitive dependencies, and advisories.
5. Add focused tests for the behavior we rely on.
6. Record the exception here before merging.

Exception format:

```text
Crate:
Used by:
Scope:
Reason:
Why not local:
Unsafe review:
Transitive dependency review:
License:
Review deadline:
Removal condition:
```

Current external dependency exceptions:

- Crate: `sanitization` `1.1.0`
  Used by: `skrifheim-crypto`
  Scope: `SecretBytes` clear-on-drop heap secret storage for memory-secrecy
  scaffolding.
  Reason: secret cleanup requires a compiler-resistant volatile wipe boundary
  that safe local Rust cannot provide by itself.
  Why not local: implementing the wipe backend locally would require adding an
  unsafe boundary to `skrifheim`; `sanitization` is a separate reviewed
  no-std-first crate owned by the same project family and intended for this
  purpose.
  Unsafe review: `skrifheim` uses only the safe API with
  `default-features = false` and `alloc`; the selected feature set uses the
  crate's documented volatile wipe boundary and no platform memory-locking,
  derive, serde, zeroize, or subtle interop features.
  Transitive dependency review: selected features have no transitive runtime
  dependencies.
  License: `MIT OR Apache-2.0`, allowed by `deny.toml`.
  Review deadline: revisit before any release that stores real key material or
  before `v0.20.0`, whichever comes first.
  Removal condition: remove or replace if the crate adds mandatory transitive
  dependencies, changes license posture, loses no-std support, or if a narrower
  admitted local unsafe boundary is approved.
- Crate: `blake3` `1.8.5`
  Used by: `skrifheim-world`
  Scope: deterministic tenant-scoped world identity derivation for scaffold
  compact handles.
  Reason: world identity must use collision-resistant domain-separated
  derivation before it can safely scope fact sets, world diffs, projection
  metadata, or future storage keys.
  Boundary: this is a non-secret identifier derivation boundary only. BLAKE3
  must not be used as a signature algorithm, encryption algorithm, password
  hash, or authorization token. `skrifheim-crypto` rejects `AlgorithmId::Blake3`
  in signature-envelope contexts.
  Production direction: before `WorldId` or derived storage addresses become
  durable trust roots, add an admitted SHA-3/SHAKE digest boundary with
  configurable `Sha3_256`, `Sha3_384`, `Sha3_512`, `Shake256_256`, and
  `Shake256_512` profiles. Compact IDs remain handles; full-width digests carry
  storage authority.
  Why not local: implementing a cryptographic hash locally would be higher
  risk than admitting a reviewed hash crate. The previous local polynomial hash
  was suitable only as scaffold metadata and was not collision-resistant.
  Unsafe review: `skrifheim` uses the safe API with `default-features = false`.
  Unsafe, SIMD, and C backend details remain inside the dependency; no unsafe
  Rust is added to `skrifheim` core crates.
  Transitive dependency review: selected no-default feature graph is limited to
  `arrayref`, `arrayvec`, `cfg-if`, `constant_time_eq`, `cpufeatures`, and
  `cc` as the build dependency used by `blake3`; no `std`, `serde`, `zeroize`,
  mmap, rayon, or digest features are enabled by `skrifheim`.
  License: `CC0-1.0 OR Apache-2.0 OR Apache-2.0 WITH LLVM-exception`; accepted
  through the Apache-2.0 option and checked by `cargo deny`.
  Review deadline: revisit before world IDs become durable storage keys or
  before `v0.20.0`, whichever comes first.
  Removal condition: replace if the crate loses no-std support, requires an
  incompatible license, pulls mandatory broad transitive dependencies, or an
  admitted project-owned cryptographic hash boundary supersedes it.
- Crate: `subtle` `2.6.1`
  Used by: `skrifheim-core`
  Scope: policy-token equality in authorization paths.
  Reason: compartment and releasability checks must not rely on hand-rolled
  source-level branchlessness when an admitted no-std constant-time primitive is
  available.
  Unsafe review: `skrifheim` uses the safe API with `default-features = false`.
  No unsafe Rust is added to `skrifheim` core crates.
  Transitive dependency review: selected no-default feature graph has no
  transitive runtime dependencies.
  License: `BSD-3-Clause`, allowed by `deny.toml`.
  Review deadline: revisit before any production constant-time claim or before
  `v0.20.0`, whichever comes first.
  Removal condition: remove or replace if the crate loses no-std support,
  changes license posture, adds mandatory transitive dependencies, or if a
  narrower verified local constant-time boundary is approved.

## Specific Crate Rules

- Do not use `zeroize`; use `sanitization` only if memory cleanup is needed and after dependency admission. This is preferred because it is our own crate.
- Do not use the `base64` crate. If base64 is unavoidable, use `base64-ng` only after dependency admission. This is preferred because it is our own crate.

## Constant-Time Primitive Rule

Source-level branchless code is not enough evidence for production
constant-time behavior. Rust does not provide a language-level guarantee that
ordinary codegen preserves constant-time properties.

The current scaffold may use admitted no-std constant-time helper crates or
local reviewed helpers for bounded token comparison. Before any production
claim for timing-sensitive policy, key, signature, authentication, or secret
comparison paths, `skrifheim` must either:

- admit a reviewed constant-time primitive crate such as `subtle` or an
  equivalent under the external dependency admission process, or
- provide equivalent compiler-barrier and codegen evidence in a reviewed local
  implementation.

No constant-time helper graduates from scaffold to production without tests,
documentation, dependency or local-implementation review, and release-gate
evidence.

Before `skrifheim` handles real classified policy labels, the release gate must
include statistical timing evidence, such as a dudect-style harness or
equivalent codegen review, for policy-token comparison and other
timing-sensitive authorization helpers.

Structural canonicalization helpers, such as policy-token union sorting, may
use ordinary ordering comparisons while the scaffold is not operating as a
remote timing oracle. They must be documented as non-constant-time, must not be
used for authorization decisions, and must be replaced or covered by timing
evidence before any production path can expose them to untrusted timing
measurement.

## Unsafe Boundary Rule

Unsafe Rust is not allowed in core crates.

If a future feature truly cannot be implemented without unsafe code, the unsafe must first be admitted in [Unsafe Policy](unsafe-policy.md), then isolated in a dedicated boundary crate with a name that makes the risk obvious. The safe `skrifheim` core should consume only a narrow reviewed wrapper. The default project posture remains no unsafe.

## External Error Boundary Rule

`SkrifheimError` implements `Display` for internal diagnostics and trusted
operator logs only. Code that returns an error message across a tenant,
classification, process, HTTP/API, plugin, or network boundary must use
`SkrifheimError::public_message()` or a stricter wrapper that cannot expose the
diagnostic reason string.

Release reviews must treat `format!("{error}")`, `error.to_string()`, and
generic error serialization on boundary paths as information-disclosure risks.

## Validator

`scripts/validate-engineering-policy.sh` enforces the current baseline:

- every library under `crates/` has `#![no_std]`,
- every library under `crates/` has `#![forbid(unsafe_code)]`,
- core crates do not import `std`,
- `zeroize` is rejected,
- the `base64` crate is rejected.
