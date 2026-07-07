# Changelog

## Unreleased

- Reserved pre-1.0 milestones for admitted production digest and AEAD storage
  encryption, early WAL/segment fuzzing, early storage/API performance and
  integration smoke, approval-role modeling, and backup/schema compatibility.
- Reserved `v0.33.1` for the threshold-signature or bounded quorum
  multi-signature proof model before approval-sensitive production workflows.
- Added pre-1.0 source-state backend milestones for compliance-forge
  workloads: proof-carrying bundles, resource-budgeted verification,
  operation/event/explanation/context records, and sealed private realms.
- Added pre-1.0 social-feed backend milestones for high-volume applications:
  social graph visibility, timeline projections, media authorization,
  realtime/notification rebuilds, moderation/safety labels, consent, ads
  transparency, privacy-rights workflows, and E2EE message metadata boundaries.

## 0.18.1

- Added `SovereigntyScope` in `skrifheim-policy` so query-result
  sovereignty metadata can remain exact up to the bounded token-set limit or
  saturate to a redacted multi-jurisdiction sentinel on overflow.
- Changed result classification joins to return the multi-jurisdiction
  sentinel instead of failing with `InvalidSecurityToken` when more than the
  exact bounded sovereignty set is present.
- Kept invalid sovereignty tokens fail-closed before saturation is accepted.
- Ensured saturated sovereignty scope is not a grantable clearance token and
  that non-allow query plans continue to expose only public sentinel metadata.
- Added policy and query tests for exact sovereignty propagation, overflow
  saturation, invalid-token rejection, redacted debug output, and non-allow
  metadata masking.
- Resolved the first `0.18.1` pentest pass by failing closed on unsupported
  non-Unix host-storage targets, replacing bare sovereignty containment with
  explicit present/absent/indeterminate containment, bounding sovereignty
  scope input before scanning, requiring expiring break-glass attestation
  evidence, rejecting oversized world fact batches before sorting, and
  documenting the `SecretBytes` no-retention closure contract.
- Resolved the `0.18.1` pentest retest by keeping overlapping sovereignty
  scopes exact when their true unique union still fits the bounded set, and by
  fixing the release-gate self-check for the bare-containment guard.
- Removed the remaining sovereignty overflow error-overload trade-off by
  adding typed policy-token union overflow reporting in `skrifheim-core`.
- Updated the pinned stable Rust toolchain from `1.96.0` to `1.96.1` after
  checking the official Rust release announcement.
- Bumped workspace and internal crate dependency versions to `0.18.1`.

## 0.18.0

- Added fixed-size immutable segment header and footer byte encoding/parsing in
  `skrifheim-storage`.
- Kept the mirrored 256-byte segment footer layout for stronger tail
  validation and documented the overhead decision.
- Split segment wire-format mechanics into a dedicated encoding module.
- Added parser tests for fixed byte round trips, malformed metadata rejection,
  expected-domain rejection, and footer/header mismatch rejection.
- Added `SegmentFileWriter` and `SegmentFileReader` in
  `skrifheim-storage-host` for immutable segment file persistence.
- Required body length checks, body CRC verification, exact file-length checks,
  footer/header binding, expected encryption-domain validation, owner-only Unix
  permissions, and Unix symlink rejection for host segment files.
- Added a mandatory `SegmentContentVerifier` boundary on both segment writes
  and reads until the production SHA-3/SHAKE digest engine is admitted.
- Added host-file tests for segment round trips and corruption rejection.
- Resolved the first `0.18.0` pentest pass by clarifying current unkeyed
  WAL/segment integrity limits, blocking production in-repo segment verifier
  implementations until the admitted digest engine lands, syncing parent
  directories after Unix file creation, and adding batch world fact-list merge
  APIs for future high-volume ingestion.
- Bumped workspace and internal crate dependency versions to `0.18.0`.

## 0.17.0

- Moved immutable segment metadata from the storage crate root into a dedicated
  `skrifheim-storage` segment module.
- Added immutable segment footer metadata with validating construction and
  header/footer consistency checks.
- Added crypto epoch and segment encryption-domain metadata to segment headers
  and footers.
- Rejected zero segment crypto epochs, non-segment encryption domains,
  tenant/domain mismatches, missing classification/compartment/segment domain
  fields, missing CRCs, zero CRC sentinels, and all-zero content digests.
- Added constant-width structural comparison for header/footer digest and
  encryption-domain matching.
- Added redacted Debug output and security-policy gate coverage for segment
  headers, footers, and their inputs.
- Resolved the first `0.17.0` pentest pass by hardening WAL symlink handling,
  WAL domain comparison, WAL body allocation, structural fact validation
  wording, world preflight execution guards, WAL CRC64 performance, WAL crypto
  epoch ordering, audit timestamp validity, and timing-evidence planning.
- Prepared `0.17.0` for signed tag creation after pentest retest and the local
  release gate passed; GitHub verification remains the final pre-tag gate.
- Added permanent pentest-report release readiness validation for
  `security/pentest/v0.17.0.md`.
- Bumped workspace and internal crate dependency versions to `0.17.0`.
- Added `0.17.0` release metadata and gate script.

## 0.16.0

- Updated CI to `actions/checkout@v7`.
- Updated the admitted `sanitization` dependency from `1.1.0` to `1.2.2`.
- Added header-driven WAL replay state machine primitives in
  `skrifheim-storage`.
- Added recovery reports for clean replay, committed transactions, aborted
  transactions, and clean-EOF uncommitted tails.
- Added rejection paths for truncated-frame stops, transaction shape errors,
  non-advancing transaction identifiers, transaction identifier mismatch, and
  key/crypto-epoch/domain mismatch inside a transaction.
- Added crash-matrix style WAL replay tests and redacted replay diagnostics.
- Resolved the first `0.16.0` pentest pass by adding CRC64-ECMA encrypted-body
  verification on WAL write/read paths, rejecting Unix symlink WAL paths,
  rejecting zero segment integrity sentinels and WAL crypto epoch zero,
  bounding replay transaction summaries, redacting fact structural identifiers,
  moving segment integrity metadata to `ContentDigest`, and using fixed-width
  domain comparison in WAL replay.
- Resolved the second `0.16.0` pentest pass by applying Unix symlink and
  regular-file checks to the WAL read path and adding a portable Unix fallback
  for unlisted `O_NOFOLLOW` targets.
- Prepared `0.16.0` for signed tag creation after pentest passed; GitHub
  verification remains the final pre-tag gate.
- Bumped workspace and internal crate dependency versions to `0.16.0`.
- Added `0.16.0` release metadata and gate script.

## 0.15.0

- Updated the implementation and version plans so BLAKE3 remains scaffold-only
  for compact non-secret world handles, while production storage authority
  moves to configurable SHA-3/SHAKE full-width digests before durable storage
  keys are trusted.
- Added `DigestStrength`, `DigestPolicy`, `WorldIdentityDigest`,
  `ContentDigest`, and `ManifestDigest` skeletons in `skrifheim-crypto`.
- Added `skrifheim-storage-host` for host-file WAL append/read helpers outside
  the `no_std` core crates.
- Added WAL host-file tests for appending encrypted frame bodies, sequential
  reads, expected-domain rejection, body-length mismatch rejection, and partial
  header/body detection.
- Resolved the first `0.15.0` pentest pass by admitting `subtle` for
  policy-token equality, redacting `FactBuilder` actor and policy identifiers,
  creating Unix WAL files with owner-only permissions, removing the WAL encoder
  missing-CRC state at the type level, and reaffirming cryptographic
  verification as a documented non-claim.
- Resolved the second `0.15.0` pentest pass by adding explicit
  `subtle`-backed digest equality helpers, documenting structural digest
  comparison as non-authentication-only, tightening permissions on pre-existing
  Unix WAL files, removing redundant file flushes before `sync_all`, making
  policy-token length comparison fixed-width, and returning raw WAL CRC values
  now that missing WAL CRCs are structurally impossible.
- Resolved the third `0.15.0` pentest pass by making digest constant-time
  comparison expansion use a fixed 64-byte loop with documented scaffold timing
  limits, and by narrowing policy-token length comparison to a guarded `u8`
  representation.
- Prepared `0.15.0` for signed tag creation after pentest and GitHub
  verification passed.
- Bumped workspace and internal crate dependency versions to `0.15.0`.
- Added `0.15.0` release metadata and gate script.

## 0.14.0

- Added fixed-size WAL frame header metadata and parser validation in
  `skrifheim-storage`.
- Added WAL record kinds for transaction begin, fact batch, transaction commit,
  transaction abort, and checkpoint records.
- Added encrypted-body length, body CRC, encryption key, crypto epoch, and WAL
  encryption-domain metadata to WAL frame headers.
- Added malformed parser tests for bad magic, unknown record kind, non-zero
  reserved bytes, and truncated headers.
- Added redacted diagnostics and release-gate checks for WAL frame metadata.
- Added `InvalidWalFrame` with generic trust-boundary public messaging.
- Resolved the first `0.14.0` pentest pass by hardening policy-token slot
  copying, requiring strictly advancing key lifecycle epochs, preventing
  rotating keys from returning to active, rejecting zero WAL CRC sentinels,
  adding expected-domain WAL validation, adding clock-aware audit-event
  construction, and correcting empty-signature error classification.
- Resolved the second `0.14.0` pentest pass by making `AuditEvent::new`
  require a trusted clock reference, bounding audit-event backdating, preserving
  `AuditEvent::new_at` as the clock-aware compatibility path, and removing the
  misleading unreachable WAL checksum encoding arm.
- Reconfirmed documented scaffold gaps for production constant-time evidence,
  real cryptographic operations, `SecretBytes::from_slice` source ownership,
  and 128-bit `WorldId` storage uniqueness enforcement.
- Prepared `0.14.0` for signed tag creation after pentest passed; GitHub
  verification remains the final pre-tag gate.
- Bumped workspace and internal crate dependency versions to `0.14.0`.
- Added `0.14.0` release metadata and gate script.

## 0.13.0

- Added `skrifheim-audit` for typed identity and audit-event metadata.
- Added core IDs for user, service, node, replica, plugin, AI worker,
  backup-agent, admin-tool, attestation-evidence, and audit-event identities.
- Added audit identities, device/workload attestation references, audit events,
  break-glass event skeletons, audit-log protection metadata, and audit records.
- Added validation for required actor attribution, attestation time bounds,
  audit-log encryption domains, and event/protection tenant consistency.
- Added redacted debug output and security-gate checks for audit metadata.
- Resolved the first `0.13.0` pentest pass by redacting storage, world, fact,
  and audit diagnostics; requiring attested break-glass device/workload
  context; rejecting stale or future attestation evidence; and extending
  release-gate redaction checks.
- Recorded the second `0.13.0` pentest pass with no new findings; the remaining
  low-severity `MissingAuditActor` diagnostic specificity item stays covered by
  the engineering-policy trust-boundary error rule.
- Prepared `0.13.0` for signed tag creation after pentest and GitHub
  verification passed.
- Bumped workspace and internal crate dependency versions to `0.13.0`.
- Added `0.13.0` release metadata and gate script.

## 0.12.0

- Added `SecretBytes`, a bounded secret-material wrapper in `skrifheim-crypto`
  backed by admitted `sanitization` clear-on-drop storage.
- Added closure-only access, explicit clearing, redacted diagnostics, and
  oversize/capacity rejection for secret bytes.
- Admitted `sanitization` `1.1.0` with only the `alloc` feature under the
  external dependency policy.
- Added memory-secrecy documentation and security-control tracking.
- Added release-gate checks that prevent `SecretBytes` from deriving `Debug`,
  `Clone`, `PartialEq`, or `Eq`, and block raw secret byte accessors.
- Resolved the first `0.12.0` pentest pass by redacting builder/token debug
  metadata, enforcing secret capacity validation consistently, narrowing world
  names, making segment headers constructor-validated, documenting borrowed
  secret-slice behavior, and replacing scaffold world-id hashing with admitted
  BLAKE3 derivation.
- Resolved the second `0.12.0` pentest pass by documenting the 16-byte BLAKE3
  output truncation used for `WorldId`'s `u128` representation.
- Prepared `0.12.0` for signed tag creation after pentest and GitHub
  verification passed.
- Bumped workspace and internal crate dependency versions to `0.12.0`.
- Added `0.12.0` release metadata and gate script.

## 0.11.0

- Added projection encryption policy primitives for secondary indexes, graph
  indexes, search indexes, vector indexes, columnar projections, cache files,
  and compaction temporary files.
- Projection encryption policies now require
  `EncryptionDomainPurpose::Projection` domains and reject non-projection
  domains.
- Added domain-compatibility checks for projection builders so
  cross-compartment and otherwise incompatible projection/index mixing can be
  rejected before any durable projection work exists.
- Added explicit encrypted-at-rest and no-plaintext-temporary-file hooks for
  projection and compaction surfaces.
- Added tests for encrypted index/projection surfaces, cross-compartment
  rejection, invalid domain rejection, and encrypted compaction temporary files.
- Resolved the first `0.11.0` pentest pass by length-separating scaffold world
  ID hash fields, redacting encryption-domain and projection-policy diagnostics,
  removing derived equality from those policy types, enforcing classified
  projection domains, making policy-token shape checks fail closed in release
  builds, encapsulating timestamps, and documenting remaining production
  constant-time and cryptographic world-ID requirements.
- Resolved the second `0.11.0` pentest pass by aligning projection policy
  compatibility with merge semantics, redacting direct projection-surface debug
  output, and strengthening release-gate redaction checks.
- Resolved the third `0.11.0` pentest pass by making projection-surface
  structural comparison depend on an exhaustive internal tag match, forcing
  future projection-surface variants through an explicit compile-time update.
- Prepared `0.11.0` for signed tag creation after pentest and GitHub
  verification passed.
- Bumped workspace and internal crate dependency versions to `0.11.0`.
- Added `0.11.0` release metadata and gate script.

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
