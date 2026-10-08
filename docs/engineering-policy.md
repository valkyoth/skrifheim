# skrifheim Engineering Policy

Status: hard rule

`skrifheim` is a military-security-oriented database project. Convenience is not enough reason to add `std`, unsafe code, or third-party dependencies to the trusted core.

## Hard Rules

Crates under `crates/` are core database crates unless they are explicitly
listed as host-boundary crates in this document. Core library targets must:

- use `#![no_std]`,
- use `#![forbid(unsafe_code)]`,
- avoid `std` imports entirely,
- prefer `alloc` only where owned dynamic data is necessary,
- prefer `skrifheim`-owned primitives for security-critical behavior,
- avoid external dependencies unless the reason is documented before use.

Host-only code may use `std`:

- `crates/skrifheim-storage-host`,
- `crates/skrifheim/src/main.rs`,
- `tools/xtask`,
- shell scripts,
- future fuzz, release, and test-only tools.

`crates/skrifheim-entropy-host` is an explicit OS boundary even though its
library is `no_std`; it depends on OS entropy through getrandom. The optional
`crates/skrifheim-crypto-rustcrypto` software-provider boundary is also
`no_std` and forbids project unsafe. Pure protocol crates cannot depend on
either boundary. The facade composes them only through opt-in features.

Host-only code still follows the dependency review rule.

Application-family extension crates may exist under `crates/` or a future
extension workspace, but they are not allowed to become hidden core
dependencies. Core crates must not depend on `skrifheim-ext-*` crates, and
extension crates must compose existing authorization, legal/compliance,
encryption-domain, key-lifecycle, provenance, audit, and release-evidence
semantics instead of redefining them.

Before implementation starts for an extension milestone, each proposed
primitive must be classified as mandatory core, generic extension helper, or
product-owned schema. Mandatory-core classification requires written proof that
the primitive is required by the world database itself. Product-owned schema
must stay outside core crates. The review must name the expected denial,
quarantine, redaction, rebuild, audit, and legal/compliance tests.

## Portability Rules

`skrifheim` must not become a Linux-only or x86-only database by accident.

Core crates must stay OS-neutral and architecture-neutral:

- no direct filesystem, network, process, clock, thread, or terminal APIs,
- no `target_arch`, `target_feature`, `std::arch`, or `core::arch` fast paths
  in core crates unless a portable baseline path is already present and the
  optimization is admitted as optional,
- no native-endian, pointer-width, alignment, page-size, or filesystem-ordering
  assumptions in durable formats,
- durable bytes must use explicit encodings, explicit endianness, explicit
  length bounds, and checked integer conversions,
- platform-specific behavior belongs behind host-boundary modules or crates
  with fail-closed unsupported-platform behavior and tests.

Supported production OS families are Linux, Windows, macOS, and BSD. Android
and iOS are future targets where the sandbox and filesystem model can support
the same security guarantees. Supported CPU families must include x86_64 and
AArch64 from the first production support pass; RISC-V and other architectures
must remain possible by avoiding architecture-specific assumptions.

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
- generic release, projection, and dependency primitives used by optional
  application-family extension crates.

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

- Crates: `sha3` `0.12.0`, `shake` `0.1.0`, `chacha20poly1305` `0.11.0`
  Used by: optional `skrifheim-crypto-rustcrypto` provider boundary only.
  Scope: SHA-3/SHAKE digests and full-round ChaCha20-Poly1305 variants.
  Reason: v0.18.3 needs standards-based primitives with independent known-answer
  evidence; locally writing cryptographic algorithms is higher risk.
  Why not local: brynja is the intended future replacement after qualification.
  The maintainer approved this temporary provider and its internal cleanup
  exception on 2026-10-08. No RustCrypto type enters the core provider contracts.
  Unsafe review: project wrappers forbid unsafe; selected upstream code uses
  reviewed slice casts, sponge cursor invariants, CPU detection and optional
  architecture intrinsics behind safe APIs. These remain dependency TCB and
  do not establish production timing or residue guarantees.
  Transitive dependency review: `digest`, `crypto-common`, `hybrid-array`,
  `typenum`, `keccak`, `sponge-cursor`, `aead`, `cipher`, `inout`, `chacha20`,
  `poly1305` (direct `0.9.1` solely to enable private MAC-state cleanup),
  `universal-hash`, `block-buffer`, `ctutils`, `cmov`, CPU support and internal
  `zeroize` cleanup.
  Default features disabled; no reduced-round algorithms, std, RNG shortcuts,
  serialization, or third-party runtime selected.
  License: MIT OR Apache-2.0; resolved licenses/advisories checked at the gate.
  Review deadline: every provider change and before final storage encryption.
  Removal condition: replace with a reviewed brynja adapter passing the same
  algorithm, transcript, failure, portability and compatibility tests.
- Crate: `zeroize` `1.9.1` (transitive exception only)
  Used by: admitted RustCrypto internals to wipe otherwise inaccessible key
  and sponge states. Never imported or directly depended on by project code.
  Reason: sanitization cannot access private upstream state through safe APIs.
  Unsafe review: upstream volatile writes and compiler fences; no project unsafe.
  License: MIT OR Apache-2.0.
  Review deadline/removal: same as RustCrypto; remove when brynja replaces it.
  Our owned secret buffers continue using `sanitization` exclusively.
- Crate: `getrandom` `0.4.3`
  Used by: `skrifheim-entropy-host` only.
  Scope: supported OS random source with fail-closed errors; no custom,
  unsupported, JavaScript or deterministic fallback backend admitted.
  Reason/why not local: owning cross-platform entropy syscall selection and
  initialization logic would introduce avoidable unsafe/FFI and portability risk.
  Unsafe review: OS syscall/FFI inside getrandom and its platform dependencies;
  project wrapper forbids unsafe and exports no OS-specific types.
  Transitive dependency review: cfg-if, libc, r-efi where applicable;
  default features disabled. No userspace PRNG state.
  License: MIT OR Apache-2.0; target-specific r-efi uses MIT OR Apache-2.0 OR LGPL-2.1-or-later,
  accepted through the MIT option and checked by cargo deny.
  Review deadline: every entropy/provider change and before persistent key use.
  Removal condition: a reviewed brynja platform adapter meets the same failure,
  fork/restart and platform requirements.
- Crate: `sanitization` `2.1.0`
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
- Crate: `blake3` `1.8.7`
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
  `arrayvec`, `cfg-if`, `constant_time_eq`, `cpufeatures`, and
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
  Used by: `skrifheim-core`, `skrifheim-crypto`, `skrifheim-crypto-rustcrypto`
  Scope: policy-token equality in authorization paths and explicit digest
  equality helpers for future identity and manifest checks.
  Reason: compartment and releasability checks must not rely on hand-rolled
  source-level branchlessness when an admitted no-std constant-time primitive is
  available. Digest identity checks need an explicit timing-sensitive equality
  path before full-width digest values become storage authority.
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

- Do not directly use `zeroize`; use `sanitization` for project-owned buffers.
  The only exception is transitive private-state cleanup inside the admitted
  RustCrypto provider described above.
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

Before digest values, encryption-domain values, or manifest roots are accepted
as production trust-boundary decisions, the same timing-evidence requirement
applies to their constant-time equality helpers. `subtle`-backed helpers are
the scaffold baseline; they are not a production constant-time claim without a
dudect-style or equivalent evidence gate.

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
- direct `zeroize` use is rejected outside admitted provider feature flags,
- the `base64` crate is rejected.
