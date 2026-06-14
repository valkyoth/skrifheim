# skrifheim 0.8.0 Release Notes

Status: fifth pentest pass resolved, pending retest.

## Scope

`0.8.0` adds key lifecycle metadata on top of the `0.7.0` key hierarchy
scaffold. It models key state transitions, rotation preflight, compromise and
quarantine states, destruction, and crypto-erasure records.

## Changes

- Added `KeyLifecycleState` for created, active, rotating, retired,
  compromised, quarantined, destroyed, and crypto-erased keys.
- Added lifecycle transition validation through `KeyMetadata::transition`.
- Added `KeyRotationPreflight` for validating active key to candidate key
  rotation metadata.
- Added `KeyErasureMetadata` and `KeyErasureReason` for crypto-erasure records.
- Added compromise, quarantine, destruction, and erasure transition tests.
- Added invalid lifecycle transition and invalid rotation preflight tests.
- Made `PolicyTokenSet::slot` return `Option<PolicyTokenSlot>` so public slot
  access cannot panic on out-of-range indexes.
- Gated the `SKRIFHEIM-TEST-SIG` fixture algorithm out of non-test builds.
- Added minimum signature length enforcement for approved hybrid signature
  component pairs.
- Documented that `SkrifheimError::Display` is diagnostic only and boundary
  responses must use `public_message()`.
- Recorded the future traversal rule that large fixed-slot policy structures
  should not be cloned through recursive graph walks.
- Added redacted `Debug` implementations for sensitive labels, policy tokens,
  fact values, facts, and fact builders.
- Added a release-gate check that blocks direct `Debug` derives on those
  sensitive data carriers.
- Required keys to pass through `Destroyed` before `crypto_erase()` can move
  metadata into `CryptoErased`.
- Documented future raw key material memory-hygiene requirements through the
  project-approved `sanitization` dependency path.
- Added compromise declaration paths from created, retired, and quarantined key
  states for incident triage.
- Added redacted `Debug` implementations for `SignatureEnvelope` and
  `SignatureSet` and release-gate coverage for those crypto types.
- Removed the fixed `WorldId(1)` fallback from deterministic world identity
  derivation; invariant violation now returns `InvalidWorldIdentity`.
- Added `SEGMENT_BODY_MAX_BYTES` and validation coverage to bound segment body
  lengths before future segment readers allocate.
- Documented that storage segment acceptance must bind `encryption_key_id` to
  safe key lifecycle states once storage and crypto metadata are wired together.
- Removed derived `PartialEq`/`Eq` from security labels, policy-token sets,
  policy-token slots, signature envelopes, and signature sets.
- Added explicit `structurally_equal` helpers for non-authoritative structural
  comparison and documented that they are not for policy or crypto decisions.
- Extended the release-gate security script to reject direct sensitive
  `PartialEq`/`Eq` derives.
- Made key hierarchy validation reject compromised, quarantined, destroyed, or
  crypto-erased parent keys.
- Added `WORLD_FACT_LIST_MAX_ITEMS`, sorted binary-search insertion, and tests
  for bounded world fact tracking.
- Reviewed the six low/informational notes: world ID hashing remains planned
  for replacement before storage-key authority, query-label memory remains
  explicitly budgeted and needs outer request limits later, arithmetic checks
  remain acceptable, private token-slot copying is documented at its invariant,
  lifecycle audit history must preserve repeated state movement, and supply
  chain remains dependency-free.
- Enforced exact lengths for currently approved fixed-size hybrid and named
  signature envelopes, removing trailing-byte slack.
- Added `compartment_id` to segment/data key scopes and required parent
  compartment keys to match that compartment.
- Changed policy required-token evaluation to iterate fixed token slots
  directly instead of handling an impossible out-of-range slot branch.
- Documented that policy-token needle canonicalization is not constant-time;
  only fixed-slot comparison is constant-shape.
- Documented tenant-level world count and aggregate world fact-reference quotas
  as future storage/orchestration requirements.
- Reconfirmed the key-material cleanup plan uses the project-approved
  `sanitization` path, not `zeroize`.
- Bumped workspace and internal crate dependency versions to `0.8.0`.
- Added `scripts/release_0_8_gate.sh`.
- Re-checked the stable Rust channel on 2026-06-14. Rust stable remains
  `1.96.0`, dated 2026-05-28 in the official stable manifest.

## Verification

- `cargo test -p skrifheim-crypto`
- `scripts/checks.sh`
- `scripts/release_0_8_gate.sh`
- `cargo deny check`
- `cargo audit`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `scripts/podman_smoke.sh`

## Non-Claims

This release is not a usable database engine. It does not generate, store,
encrypt, decrypt, rotate, recover, or erase real key material. Lifecycle and
erasure records are metadata only. No KMS/HSM integration, threshold approval,
durable storage, or cryptographic verification is implemented.

Future key-material types must follow the existing `sanitization` dependency
admission rule for memory cleanup; `zeroize` is not admitted for this project.

## Pentest Status

The first, second, third, fourth, and fifth pentest passes have been resolved.
Root `PENTEST.md` is the temporary findings handoff file and must be removed
after findings are resolved.
