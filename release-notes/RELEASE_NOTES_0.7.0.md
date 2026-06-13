# skrifheim 0.7.0 Release Notes

Status: release stop, pentest complete.

## Scope

`0.7.0` adds key hierarchy metadata and closes scaffold hardening gaps in
policy-token comparison, policy-label evaluation, signature-set validation, and
fact-builder deduplication/allocation bounds.

## Changes

- Added key hierarchy metadata for root trust, deployment, region, tenant,
  compartment, segment, and data key scopes.
- Added parent/child hierarchy validation with no database-wide-key shortcut.
- Bound tenant key scope metadata to explicit deployment and region IDs.
- Added invalid hierarchy edge tests, including data-key parent validation.
- Bounded signature sets with `MAX_SIGNATURES_PER_SET`.
- Bounded signature envelope key IDs with explicit length and character
  validation.
- Added closed allow-list validation for `Named` and `HybridClassicalPq`
  signature algorithm identifiers before real verification dispatch exists.
- Hardened policy-token equality with local compiler barriers while keeping the
  no-external-dependency posture.
- Changed policy-token storage to a fixed-slot byte representation for
  authorization checks.
- Changed policy-token set membership scans to fixed bounded slot counts without
  `BTreeSet` traversal in the policy hot path.
- Added fail-closed guards for oversized token sets at policy lookup and label
  evaluation boundaries.
- Changed policy-label compartment and releasability evaluation to fixed
  bounded slot counts.
- Changed segment CRC metadata so an explicit CRC64 value of zero is
  representable separately from a missing CRC.
- Changed fact-builder evidence and causal-link deduplication to sort/dedup
  behavior.
- Changed fact-builder link mutation methods to fail before accepting more than
  `FACT_LINK_LIST_MAX_ITEMS` entries.
- Added `FACT_OBJECT_MAX_BYTES` validation for fact text and byte payloads.
- Removed per-label planner decisions from `QueryPlan`; the public query plan
  surface exposes aggregate proof/decision state only.
- Changed `QueryRequest` to constructor-based creation with a reduced
  `QUERY_REQUEST_LABEL_MAX_ITEMS` and explicit label memory budget.
- Made deterministic world-id derivation infallible in its type signature,
  removing a misleading unreachable error branch.
- Enabled release-profile overflow checks.
- Bumped workspace and internal crate dependency versions to `0.7.0`.
- Re-checked the stable Rust channel on 2026-06-13. Rust stable remains
  `1.96.0`, dated 2026-05-28 in the official stable manifest.

## Verification

- `cargo test -p skrifheim-core -p skrifheim-policy -p skrifheim-crypto -p skrifheim-fact`
- `scripts/checks.sh`
- `scripts/release_0_7_gate.sh`
- `cargo deny check`
- `cargo audit`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `scripts/podman_smoke.sh`

## Non-Claims

This release is not a usable database engine. It does not provide real key
generation, encryption, decryption, key rotation, KMS/HSM integration,
cryptographic verification, durable storage, or production-grade
constant-time evidence. Local compiler barriers reduce scaffold timing risk but
do not replace a future reviewed constant-time primitive admission or local
codegen evidence package. A statistical timing test or equivalent codegen
evidence remains required before production claims for classified labels.

Deterministic world identifiers remain non-secret, non-capability identifiers.
Replacing the current local deterministic hash with an admitted
collision-resistant hash remains planned before durable storage uses world
identity as an authoritative storage key. Storage-backed world creation must
also enforce collision checks for different tuples that derive the same
`WorldId`.

Future key-material types must follow the existing `sanitization` dependency
admission rule for memory cleanup; `zeroize` is not admitted for this project.

## Pentest Status

First, second, third, and fourth pentest passes resolved. Root `PENTEST.md` has
been removed after the findings were handled.
