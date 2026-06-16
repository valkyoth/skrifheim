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

- None.

## Specific Crate Rules

- Do not use `zeroize`; use `sanitization` only if memory cleanup is needed and after dependency admission. This is preferred because it is our own crate.
- Do not use the `base64` crate. If base64 is unavoidable, use `base64-ng` only after dependency admission. This is preferred because it is our own crate.

## Constant-Time Primitive Rule

Source-level branchless code is not enough evidence for production
constant-time behavior. Rust does not provide a language-level guarantee that
ordinary codegen preserves constant-time properties.

The current scaffold may use local, reviewed, no-dependency helpers for
bounded token comparison. Before any production claim for timing-sensitive
policy, key, signature, authentication, or secret comparison paths, `skrifheim`
must either:

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
