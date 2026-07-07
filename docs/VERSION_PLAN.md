# skrifheim Version Plan

Status: planning document

Tags use:

```text
v0.N.0      milestone release
v0.N.P      patch/fix release
v1.0.0      first serious production-ready database
```

## Release Principles

Every release must have:

- definition of done,
- a clean stop point before any tag work,
- local verification command,
- rootless Podman status,
- security review notes,
- a pentest handoff and resolution pass,
- known limitations,
- release notes,
- no hidden dependency on one developer machine.

## Application Support Rule

Application plans may inform database requirements, but they do not define
`skrifheim` semantics. New application-facing features must be expressed as
generic, tenant-scoped database primitives with:

- fail-closed policy and authorization behavior,
- legal/compliance passport compatibility,
- encryption-domain and key-lifecycle boundaries,
- bounded resource and parser behavior,
- audit and provenance records,
- redacted diagnostics and public error shapes,
- deterministic tests for denial, quarantine, redaction, and rebuild paths.

Product-specific schema, UI workflow, ranking choices, media processing tools,
source object formats, and moderation policy text belong in the consuming
application. `skrifheim` provides the secure, auditable, compliance-aware
foundation those applications adapt to.

## Clean Stop And Pentest Rule

Each version has a deliberate clean stop. When implementation criteria are done,
the work stops before tagging and the maintainer is told:

```text
vX.Y.Z implementation stop reached. Run pentest for this exact commit.
```

No tag is created at that point.

Pentest flow:

1. Implementation reaches the version stop point.
2. Local gates pass: `scripts/checks.sh`, `cargo deny check`, and `cargo audit`.
3. The maintainer runs pentest and writes findings to root `PENTEST.md`.
4. The findings are reviewed and fixed.
5. `PENTEST.md` is removed after the findings are handled.
6. Local gates are run again.
7. A permanent report is written at `security/pentest/<tag>.md` only after the
   release-prep commit is complete and the result is `Status: PASS`.
8. The permanent report commit changes only that report and records
   `Reviewed-Commit:` as its first parent, so the tag candidate contains both
   the reviewed code and committed pentest digest.
9. Tagging and pushing tags happen only when explicitly requested.

Root `PENTEST.md` is temporary scratch input. It must not be committed, and the
release metadata validator fails while it exists.

## v0.1.0 - Repository Foundation

Goal: initialize the serious Rust workspace and policy baseline.

Deliverables:

- Rust stable `1.96.1` pinned.
- Focused workspace crates.
- `scripts/checks.sh`.
- CI, dependency policy, security policy, release notes.
- Implementation, version, modularity, threat-model, toolchain, and CMS target docs.

Verification:

- `scripts/checks.sh`
- `scripts/release_0_1_gate.sh`
- `cargo test --workspace`

## v0.2.0 - Core Fact Builder

Goal: make canonical facts constructible through validated APIs.

Deliverables:

- fact builder,
- required evidence checks,
- valid-time checks,
- signature-set checks,
- confidence validation,
- negative tests for invalid facts.

## v0.3.0 - Security Labels And Authority Context

Goal: make classification, compartments, releasability, and authority context explicit.

Deliverables:

- security label model,
- subject/device/workload context types,
- clearance dominance checks,
- compartment and releasability tests,
- no read-above-clearance tests.

## v0.4.0 - Policy Decision Engine

Goal: turn security labels into deterministic planner decisions.

Deliverables:

- allow, redact, and reject decisions,
- output classification calculation,
- constant-shape denial model,
- policy proof skeleton,
- tests for dangerous join and redaction cases.

## v0.5.0 - World Identity And DAG Model

Goal: model worlds as branchable overlays, not copied databases.

Deliverables:

- world metadata,
- tenant-scoped identity derivation,
- parent pointers,
- added and hidden fact sets,
- deterministic world identity rules where `(tenant_id, kind, depth, parent,
  name)` is the idempotent uniqueness key,
- admitted BLAKE3-based collision-resistant scaffold world-id derivation for
  non-secret compact handles only,
- documented requirement that durable storage reject existing `WorldId` values
  for different `(tenant_id, kind, depth, parent, name)` tuples before world IDs
  become authoritative durable storage keys,
- documented requirement that production storage authority move to admitted
  SHA-3/SHAKE full-width world identity digests before durable storage keys are
  trusted,
- tests for branch isolation.

## v0.6.0 - World Diff And Promotion Preflight

Goal: compare worlds safely before merge or promotion.

Deliverables:

- deterministic world diff,
- conflict categories,
- promotion preflight,
- rollback preflight,
- tests for conflicting fact replacement.

## v0.7.0 - Key Hierarchy Model

Goal: define encryption authority before durable storage exists, and close
scaffold hardening gaps that would affect policy, signature, and key paths.

Deliverables:

- root trust, deployment, region, tenant, compartment, and segment/data key IDs,
- key hierarchy metadata,
- parent/child key relationship validation, including tenant deployment/region
  binding,
- no database-wide-key shortcut,
- constant-time policy-token comparison hardening, either through admitted
  `subtle` or documented local compiler-barrier/codegen evidence,
- fixed-shape policy-label evaluation over bounded compartment and
  releasability slots,
- fixed-slot policy-token representation for authorization hot paths,
- maximum signature-envelope count per `SignatureSet`,
- bounded signature key identifiers,
- closed allow-list validation for named and hybrid signature algorithm
  identifiers,
- O(n log n) fact-builder deduplication for evidence and causal links,
- fail-fast fact-builder bounds before incremental link vectors can grow
  beyond `FACT_LINK_LIST_MAX_ITEMS`,
- bounded fact text/byte payload size before durable ingest,
- constructor-enforced query-label limits and aggregate-only public query-plan
  proof surface,
- release-profile overflow checks,
- tests for invalid hierarchy edges, including segment/data key parent checks.

## v0.8.0 - Key Lifecycle And Epochs

Goal: model key creation, rotation, compromise, and destruction.

Deliverables:

- key lifecycle states,
- key epoch transitions,
- rotation preflight,
- compromise/quarantine state,
- crypto-erasure metadata,
- tests for invalid lifecycle transitions.

Follow-up:

- `v0.18.2` is reserved for separating crypto-material epoch advancement from
  lifecycle metadata changes. The current scaffold requires every lifecycle
  transition, including compromise and quarantine declarations, to advance
  `CryptoEpoch`. That is fail-closed, but it may not match HSM/KMS operations
  where compromise declaration is a metadata/audit state change for the same
  key material epoch.

## v0.9.0 - Encryption Domains

Goal: define blast-radius boundaries for encrypted data.

Deliverables:

- tenant encryption domain,
- region encryption domain,
- classification and compartment domains,
- world/branch domain metadata,
- backup/export/AI/WASM/audit domains,
- tests that incompatible domains cannot be merged.

## v0.10.0 - Query-Result Classification

Goal: classify derived results before query execution exists.

Deliverables:

- output classification join rules,
- sovereignty propagation rules,
- PII-derived output marker,
- AI-processing eligibility marker,
- confidence-threshold policy hooks for future propagated confidence,
- tests for classification escalation.

Follow-up:

- `v0.18.1` is reserved for the sovereignty overflow fix. The design gap
  originated in the `v0.10.0` result-classification scaffold, but the repair
  must use a future monotonic release number.

## v0.11.0 - Index And Projection Encryption Policy

Goal: make index leakage part of the threat model and planner.

Deliverables:

- encrypted secondary-index policy,
- encrypted graph/search/vector/columnar projection policy,
- projection encryption domain selection,
- compaction temporary file encryption policy,
- tests for no cross-compartment projection mixing.

## v0.12.0 - Memory Secrecy Policy And Secret Types

Goal: create safe secret-handling boundaries before real keys exist.

Deliverables:

- secret value wrapper,
- no-debug secret policy,
- no secrets in panic/log text tests,
- approved cleanup path using admitted `sanitization`,
- memory-secrecy documentation.

## v0.13.0 - Identity And Audit Event Model

Goal: make security-relevant actions attributable.

Deliverables:

- identity types for users, devices, workloads, services, nodes, replicas,
  plugins, AI workers, backup agents, and admin tools,
- attestation-evidence references for device and workload posture,
- audit event model,
- encrypted/signed audit-log metadata,
- break-glass audit event skeleton,
- tests for required actor attribution.

## v0.14.0 - WAL Frame Format

Goal: define and validate append-only WAL frames before persistence logic.

Deliverables:

- encrypted WAL frame header,
- record kind model,
- length and checksum fields,
- key epoch and encryption domain fields,
- non-zero epoch and CRC sentinel rejection,
- frame validation,
- parser tests for malformed frames.

## v0.15.0 - WAL Writer And Reader

Goal: write and read WAL frames through portable file I/O.

Deliverables:

- digest strength policy model with `Sha3_256`, `Sha3_384`, `Sha3_512`,
  `Shake256_256`, and `Shake256_512` planned profiles,
- full-width digest type skeletons for world identity, content, and manifests,
- documentation that BLAKE3 remains scaffold-only for compact non-secret
  `WorldId` derivation,
- release-gate checks that prevent durable storage authority from keying only
  on compact `WorldId`,
- append-only writer,
- sequential reader,
- CRC64-ECMA encrypted-body verification on write and read,
- encrypted frame metadata checks,
- Unix symlink rejection for WAL append paths,
- fsync boundary option,
- partial-write detection,
- tests using temporary host files.

## v0.16.0 - WAL Replay And Recovery States

Goal: recover committed state and reject ambiguous corruption.

Deliverables:

- replay state machine,
- committed/uncommitted transaction handling,
- truncated-frame handling,
- key epoch mismatch handling,
- bounded replay transaction summaries,
- fixed-width replay domain comparison,
- recovery report,
- crash matrix tests for WAL-only storage.

## v0.17.0 - Immutable Segment Format

Goal: define policy-scoped immutable fact segments.

Deliverables:

- segment header,
- footer,
- algorithm-agile content digest field,
- explicit checksum presence representation where CRC64 value `0` is rejected
  as a missing-integrity sentinel,
- policy and encryption metadata,
- key epoch and encryption domain fields,
- segment validation tests.

## v0.18.0 - Segment Writer And Reader

Goal: persist and read immutable segments without compaction.

Decision: keep the full mirrored 256-byte footer for the first durable segment
reader/writer pass. The extra fixed footer cost is intentional because it lets
the reader bind both ends of the immutable file, reject tail corruption, reject
header/footer metadata drift, and fail closed on trailing bytes. Compact
trailers can be reconsidered after segment sizing policy exists, but not before
corruption evidence is stronger than the current mirrored layout.

Deliverables:

- segment writer,
- segment reader,
- on-disk encoding decision for duplicated header/footer metadata,
- overhead measurement for very small segments where a full mirrored footer may
  be significant,
- explicit decision to keep full footer mirroring for corruption/tail
  validation or replace it with a compact trailer that still detects
  header/body/footer mismatch,
- checksum verification,
- content-hash verification,
- encryption metadata verification,
- cryptographic signature-verification boundary for any deserialized facts
  before they can be treated as committed,
- corruption rejection tests.

## v0.18.1 - Sovereignty Scope Overflow Sentinel

Goal: make sovereignty propagation fail restrictive without failing as an
invalid query when a result spans more jurisdictions than the exact bounded set
can represent.

Deliverables:

- replace raw result-sovereignty `PolicyTokenSet` storage with a typed
  sovereignty scope,
- exact bounded sovereignty set for up to `POLICY_TOKEN_SET_MAX_ITEMS`
  jurisdictions,
- saturated multi-jurisdiction sentinel for overflow,
- most-restrictive handling for the saturated state in policy proofs and query
  plans,
- redacted debug output for the new sovereignty scope,
- tests that more than 64 distinct sovereignty tokens produces a
  multi-jurisdiction result instead of `InvalidSecurityToken`,
- tests that non-allow query plans continue to expose only public sentinel
  metadata,
- documentation that the sentinel is not a grantable clearance token and must
  be handled as approval-required or deny by export, placement, indexing,
  backup, AI processing, and legal/compliance decisions.

## v0.18.2 - Key Lifecycle Event Epoch Semantics

Goal: make key compromise and quarantine operationally precise before manifests
and recovery logic treat crypto epochs as durable storage authority.

Deliverables:

- document the distinction between crypto-material epochs and lifecycle/audit
  event ordering,
- add a lifecycle event sequence, state epoch, or equivalent non-cryptographic
  ordering marker for metadata-only state changes,
- allow compromise, quarantine, destruction, and crypto-erasure metadata
  transitions to preserve the existing `CryptoEpoch` when no new key material
  is introduced,
- keep rotation and replacement-key activation on strictly advancing
  `CryptoEpoch` values,
- keep storage, WAL, segment, and manifest crypto-epoch fields tied only to key
  material/version identity, not administrative state-change ordering,
- ensure compromised, quarantined, destroyed, and crypto-erased states remain
  fail-closed for key hierarchy validation and storage access regardless of
  whether the crypto epoch changed,
- add tests for compromise declaration without epoch advancement,
- add tests that rotation and replacement still require epoch advancement,
- update encryption architecture and security controls to describe the split
  between crypto epoch and lifecycle event ordering.

## v0.18.3 - Production Digest And AEAD Engine Admission

Goal: turn digest and encryption metadata into real cryptographic storage
controls before manifests, checkpoints, or recovery can treat stored bytes as
tamper-resistant.

Deliverables:

- dependency-admission decision for the production SHA-3/SHAKE digest engine
  and AEAD implementation, including license, maintenance, advisory, `no_std`
  fit, unsafe-code boundary, platform support, and test evidence,
- configurable digest-strength policy wired to actual SHA3-256, SHA3-384,
  SHA3-512, SHAKE256-256, and SHAKE256-512 computation,
- production `ContentDigest`, `ManifestDigest`, and `WorldIdentityDigest`
  computation APIs,
- AEAD envelope format for WAL bodies and immutable segment bodies, including
  algorithm ID, nonce strategy, associated-data binding, key ID, crypto epoch,
  policy epoch, tenant, encryption domain, body length, and replay context,
- domain-separated key derivation contract from the key hierarchy to WAL,
  segment, projection, backup, export, AI artifact, and audit-log data keys,
- nonce uniqueness and crash-recovery analysis for append-only WAL writes and
  immutable segment writes,
- associated-data tests proving ciphertext cannot be replayed across tenant,
  compartment, world, WAL, segment, projection, backup, export, AI, or audit
  domains,
- corrupt ciphertext, swapped header/footer, wrong key, wrong epoch, wrong
  domain, truncated body, and replay rejection tests,
- explicit rule that CRC64 remains a structural corruption check only and that
  AEAD authentication plus signed/keyed manifests are the production integrity
  boundary,
- release-gate check that no durable storage path can claim tamper resistance
  unless the admitted digest and AEAD engine is in use.

## v0.18.4 - WAL And Segment Fuzz Baseline

Goal: fuzz the hand-written byte parsers immediately after WAL and segment
encoding exist, rather than waiting for the general fuzz baseline.

Deliverables:

- fuzz harness for WAL frame header parsing,
- fuzz harness for WAL replay frame sequences,
- fuzz harness for segment header/footer parsing,
- fuzz harness for segment file length/body/footer layout validation,
- seed corpus from existing malformed WAL and segment fixtures,
- bounded allocation assertions for body lengths and file lengths,
- CI/local gate mode that runs a deterministic short fuzz smoke without
  becoming flaky,
- documentation that `v0.44.0` remains the broader fuzz/property baseline but
  storage parser fuzzing is already required from this point forward.

## v0.19.0 - Manifest And Checkpoint Format

Goal: record the durable storage root.

Deliverables:

- manifest structure,
- checkpoint LSN,
- segment list,
- digest strength profile field,
- full-width world identity, content, and manifest digest fields,
- policy epoch field,
- crypto epoch field,
- encryption domain inventory,
- world-id collision detection that rejects an existing `WorldId` for a
  different `(tenant_id, kind, depth, parent, name)` tuple,
- rejection of manifests whose full-width digest profile does not match the
  active deployment policy,
- manifest validation tests.

## v0.20.0 - Startup Recovery Integration

Goal: combine manifests, segments, and WAL replay at startup.

Deliverables:

- recovery loader,
- manifest selection,
- WAL replay from checkpoint,
- storage-backed world ancestry verification before promotion or rollback
  execution can be authorized,
- internal storage-validated promotion and rollback preflight construction that
  can set the currently private storage-validation marker only after durable
  ancestor traversal succeeds,
- corrupted manifest rejection,
- missing-key and compromised-key rejection,
- deterministic recovery fixtures.

## v0.20.1 - Production Timing Evidence Gate

Goal: add statistical timing evidence before any production constant-time
claim.

Deliverables:

- dudect-style or equivalent timing harness for policy-token equality,
- `DigestValue::structurally_equal_ct` timing evidence,
- encryption-domain equality timing evidence,
- release-gate integration for timing-sensitive authorization and storage-root
  comparisons.

## v0.20.2 - Early Storage Performance And Recovery Smoke

Goal: collect lightweight performance and integration evidence while storage
layout, mirrored footers, fixed-slot policy metadata, and recovery flow are
still cheap to change.

Deliverables:

- microbenchmarks for WAL append/read, segment write/read, manifest selection,
  and startup recovery over small, medium, and large fixture sets,
- measurements for mirrored 256-byte segment footer overhead on very small
  segments and normal segment sizes,
- policy-token fixed-slot scan benchmark for common authority/label shapes,
- rootless Podman recovery smoke with realistic persisted volume layout,
- documented thresholds that are non-claims but catch obvious regressions,
- decision record on whether footer layout, token-slot bounds, or recovery
  sequencing need adjustment before transaction durability depends on them.

## v0.21.0 - In-Memory Transaction Model

Goal: model read sets, write sets, predicate sets, commit timestamps, and fact
identity allocation semantics.

Deliverables:

- transaction state type,
- read/write/predicate set tracking,
- read-your-writes overlay for uncommitted writes inside the owning
  transaction,
- deterministic lookup order for transaction-local writes, tombstones,
  predicate reads, and committed snapshot state,
- isolation rule that uncommitted writes are visible only to the owning
  transaction until durable commit succeeds,
- conflict model,
- commit timestamp allocation,
- fact ID allocation strategy decision before write-set validation hardens,
- explicit evaluation of monotone/timestamp-derived IDs, random IDs,
  content-derived IDs, and hybrid tenant/world scoped IDs,
- decision on whether `FactId` may reveal write ordering or must remain
  order-hiding,
- write-time uniqueness checks for whichever strategy is selected,
- MVCC implications for fact identity, transaction identity, valid time,
  commit time, and causal links,
- documentation that `FactId` is not an authorization capability and must not
  be used as a sole storage isolation key,
- tests that a transaction reads its own inserted, updated, hidden, superseded,
  and invalidated facts before commit,
- tests that other transactions cannot see those uncommitted writes,
- deterministic unit tests.

## v0.22.0 - Strict Serializable Validation

Goal: enforce strict serializable single-node transaction rules in memory.

Deliverables:

- read/write conflict validation,
- predicate conflict validation,
- write-write conflict validation,
- abort reasons,
- deterministic concurrency tests.

## v0.23.0 - Durable Transaction Commit

Goal: connect transaction validation to WAL and recovery.

Deliverables:

- prepare and commit records,
- durable commit boundary,
- audit event emission,
- replay of committed transactions,
- rollback of uncommitted transactions,
- crash tests around prepare/commit.

## v0.23.1 - Confidence Propagation Math

Goal: define the confidence semantics before fact indexes and query execution
depend on caused-by traversal.

Deliverables:

- choose and document the initial confidence propagation model,
- explicitly evaluate min-chain, decay, product, and Dempster-Shafer style
  alternatives,
- prototype the selected model with deterministic fixtures over evidence and
  caused-by chains,
- define source-reliability weighting and its bounded integer representation,
- define how direct fact confidence, evidence reliability, and causal depth
  combine into computed confidence,
- define confidence behavior for supersedes and invalidates links separately
  from ordinary caused-by links,
- define saturation, rounding, and overflow behavior for the `0..=1000`
  confidence scale,
- define how query results report computed confidence and confidence proof
  inputs without leaking restricted facts,
- define policy behavior for confidence thresholds, including allow, redact,
  reject, and approval-required outcomes,
- add tests that long causal chains decay or otherwise degrade confidence
  according to the selected model,
- add documentation that v0.28.0 query execution must implement this model
  rather than inventing propagation semantics during execution work.

## v0.23.2 - Transaction Storage Load And Crash Smoke

Goal: test the storage and transaction path under realistic local load before
query AST and execution decisions lock in planner assumptions.

Deliverables:

- write/read load smoke for transaction begin, fact batch, commit, abort, WAL
  replay, manifest checkpoint, and segment rollover,
- recovery-time measurements after clean shutdown, crash during append, crash
  after prepare, crash after commit, and crash during checkpoint,
- memory-use checks for transaction-local read-your-writes overlays,
- early evidence for whether fixed-shape policy evaluation, storage footer
  overhead, and sequential recovery are acceptable for 1.0 targets,
- documented adjustment decision before v0.24 snapshot reads and v0.25 query
  AST depend on the current storage/transaction shape.

## v0.24.0 - Fact Index And Snapshot Reads

Goal: read facts by world and snapshot from recovered state.

Deliverables:

- in-memory fact index,
- snapshot timestamp visibility,
- supersession and invalidation lookup,
- forward causal-edge lookup for future blast-radius traversal,
- confidence-propagation fixtures from `v0.23.1` available to index/query
  tests,
- fact history tests,
- unauthorized stale-read tests.

## v0.24.1 - Simulation Query Isolation Model

Goal: decide how `SimulateConsequences` queries are isolated before the native
query AST locks in simulation syntax and execution assumptions.

Deliverables:

- choose between writable temporary world forks and read-only counterfactual
  query-time reasoning for simulation queries,
- document how `QueryIntent::SimulateConsequences` relates to
  `WorldKind::Simulation`,
- if writable temporary forks are selected, define fork lifetime, deterministic
  identity, transaction boundaries, cleanup, promotion prohibition by default,
  audit requirements, and storage isolation,
- if read-only counterfactual reasoning is selected, define assumption syntax,
  overlay representation, result provenance, bounded traversal, and proof
  output without creating durable world state,
- define policy checks for simulated assumptions, including labels,
  compartments, releasability, AI eligibility, and confidence thresholds,
- define whether simulation results can feed context packs, projections, AI
  artifacts, or durable facts, and which signed promotion/declassification
  proofs would be required,
- add tests that simulation cannot mutate the source world unless an explicit
  future write/promote path authorizes it,
- add documentation that `v0.25.0` must use the selected model when defining
  simulation query AST nodes.

## v0.24.2 - Early Authenticated API And Planner Integration Smoke

Goal: validate that subject, device, workload, transport identity, and planner
context can be bound through a real request boundary before the native query
AST and execution pipeline are locked in.

Deliverables:

- local authenticated API smoke endpoint for a minimal read-plan request,
- authority-context extraction hook for subject, device, and workload identity,
- mTLS, passkey, signed-local-token, or equivalent identity-binding decision
  record for the future full API,
- tests that forged device/workload context cannot be supplied directly to
  policy evaluation through the API boundary,
- constant-shape public error response fixture for unauthenticated,
  unauthorized, redacted, and malformed requests,
- integration test that runs legal/security/sovereignty/minimisation planning
  stubs through the same request context shape the later query planner will
  use,
- documented decision on whether the planned seven-stage planner pipeline needs
  restructuring before v0.25 and v0.27.

## v0.25.0 - Native Query AST

Goal: define the first native query representation without execution.

Deliverables:

- world read AST,
- fact filter AST,
- causality explain AST,
- simulation query AST skeleton using the `v0.24.1` isolation model,
- AST validation tests.

## v0.26.0 - Native Query Parser

Goal: parse the minimal native query language.

Deliverables:

- parser,
- source spans,
- structured parse errors,
- parser fixtures,
- fuzz seed corpus.

## v0.26.1 - Break-Glass Access Model

Goal: define what break-glass access actually does before policy-aware query
planning can accidentally treat an audit event as an unrestricted bypass.

Deliverables:

- decide whether break-glass grants temporary scoped authority, opens a scoped
  target, creates an isolated `LegalAudit` world, or produces an
  approval-required access request without bypassing normal policy by itself,
- explicitly reject implicit global `TopSecret` escalation unless a future
  threat-model review approves it,
- define the break-glass request type, target scope, maximum duration,
  classification ceiling, world/fact/query scope, reason code, approver or
  threshold approval requirements, and revocation behavior,
- define how break-glass proofs bind to attested subject, device, workload,
  tenant, policy epoch, crypto epoch, and audit-log protection,
- define one-time emergency capability options, including hardware-backed
  identity, employee PKI, FIDO2/passkey, smart card, HSM-held key, encrypted
  one-time file, or identity-vault reference,
- document that passport, national ID, face template, or biometric evidence
  must not be stored as ordinary database payload; if used, store only encrypted
  identity-vault references, issuer proof, digest, expiration, and policy
  metadata in `skrifheim`,
- define AI-assisted identity-proofing as evidence only, not sole
  authorization; deterministic policy must still verify identity proof,
  attested device/workload, target scope, duration, approval threshold,
  protected audit logging, and one-time-use state,
- define whether reads happen against the source world, a policy-filtered
  legal/audit view, or a new `WorldKind::LegalAudit` isolation world,
- require deterministic non-secret denial and approval-required outcomes for
  failed break-glass preconditions,
- require tamper-evident audit records before, during, and after any granted
  break-glass operation,
- add tests that a `BreakGlass` audit event alone does not bypass
  `evaluate_read` or query-planning policy,
- add tests that stale attestation, missing workload context, overbroad target
  scope, expired approval, missing audit-log protection, reused one-time
  capability, or AI-only identity verification deny access,
- add persisted one-time-use state so the same attestation evidence identifier
  or emergency capability cannot authorize multiple break-glass operations.

## v0.26.2 - Approval Roles And Operational Authority Model

Goal: make "human-approved", "authority-approved", and "threshold-approved"
operations executable for a small deployment before break-glass, law-pack
admission, key ceremonies, and release gates depend on those words.

Deliverables:

- actor role model for owner, maintainer, security officer, legal reviewer,
  key guardian, emergency approver, auditor, and automated service,
- single-maintainer fallback policy that can still represent separation of
  duties through explicit self-approval records, time delay, hardware key, or
  external reviewer evidence,
- threshold approval metadata with required role set, quorum, validity window,
  revocation, and audit-log binding,
- approval request and approval proof skeleton shared by break-glass, law-pack
  admission, key lifecycle, declassification, export, backup restore, and
  release operations,
- tests that missing approver role, stale approval, self-approval where not
  allowed, reused approval nonce, and wrong tenant/policy epoch are rejected,
- documentation that deployments with no legal authority or threshold group
  must configure explicit local roles before enabling approval-gated features.

## v0.26.3 - Platform Identity And Product Boundary Model

Goal: support multi-application deployments where authentication, shared
account/profile data, operator identity, support identity, and product data stay
separated by service, tenant, policy, and encryption domain.

Deliverables:

- identity-authority metadata for member, operator, support-agent, service,
  guardian, and high-assurance certificate/public-key actors,
- shared-account profile boundary model that separates public display fields,
  private encrypted account fields, product activation state, app launcher
  state, and product-owned data,
- product/service passport model for product identifier, tenant scope,
  database/storage boundary, secret-policy boundary, allowed identity claims,
  and deletion/export contract,
- minimal derived-claim model for age band, minor status, child-mode required,
  guardian required, age-policy jurisdiction, and consent state without
  exposing raw birthdate or private identity fields to products by default,
- guardian consent and oversight proof skeleton with revocation, validity
  window, jurisdiction, actor, child account, and audit binding,
- service-secret and external key/secret provider policy boundary metadata
  showing which service may read which secret path, unwrap which key, or sign
  which token,
- tests that product services cannot read private account fields directly,
  operator permissions do not grant product-user authority, support-agent roles
  do not grant admin authority, and derived claims are sufficient for policy
  decisions without raw sensitive identity data.

## v0.27.0 - Policy-Aware Query Planning

Goal: convert query AST into a policy-checked plan.

Deliverables:

- logical plan,
- security checks before execution,
- break-glass planning uses the `v0.26.1` access model and does not treat audit
  metadata as a blanket policy bypass,
- rejection and redaction reports,
- policy proof skeleton,
- query-result classification,
- sovereignty-scope overflow handling: exact bounded jurisdictions stay exact,
  while more than the bounded exact set becomes a typed multi-jurisdiction
  sentinel that is treated as most-restrictive for export, placement, indexing,
  backup, AI processing, and legal/compliance decisions,
- confidence-aware allow/redact/reject policy hooks,
- tests for denied plans.

## v0.28.0 - Query Execution Prototype

Goal: execute read-only fact and causality queries on a single node.

Deliverables:

- fact scan execution,
- point lookup execution,
- causality edge traversal over fact links,
- bounded forward traversal for taint and blast-radius queries,
- implementation of the selected `v0.23.1` propagated-confidence model over
  evidence and caused-by chains,
- bounded result sets,
- tests for authorized and denied reads.

## v0.29.0 - Projection Registry

Goal: register rebuildable projections without implementing every projection type.

Deliverables:

- projection metadata,
- source fact range,
- consistency mode,
- watermark tracking,
- encryption domain tracking,
- rebuild command skeleton.

## v0.30.0 - Graph Projection

Goal: build the first projection from causal facts.

Deliverables:

- adjacency projection,
- source range tracking,
- rebuild from canonical facts,
- stale projection detection,
- tainted projection detection from causal blast-radius traversal,
- encrypted projection metadata,
- graph traversal tests.

## v0.31.0 - Search Projection Skeleton

Goal: create a policy-aware full-text/search projection boundary.

Deliverables:

- tokenizer boundary,
- source fact visibility checks,
- projection watermark,
- rebuild tests,
- no cross-compartment mixing tests.

## v0.32.0 - Vector And AI Projection Encryption Boundary

Goal: make vector and AI-derived projections safe before AI artifacts become useful.

Deliverables:

- vector projection encryption domain,
- AI artifact encryption domain,
- AI write capability ceiling metadata,
- derivation-cone key-domain metadata,
- source-fact visibility rules,
- no lower-domain embedding of higher-domain facts,
- tests for denied vector/AI projection writes.

## v0.32.1 - Social Graph Visibility And Timeline Projection Model

Goal: define policy-bound primitives for high-volume social-feed workloads
without letting feeds, search, counters, or caches bypass viewer-specific
authorization, legal basis, encryption-domain, or moderation policy.

Deliverables:

- social graph edge model for follow, follow request, block, mute, list
  membership, community membership, reply, quote, repost, like, bookmark, and
  report relationships,
- viewer-context visibility planner for public, unlisted, followers-only,
  close-circle, subscriber/group, community-only, deleted, tombstoned, and
  visibility-reduced content states,
- protected-account, block, mute, muted-word, reply-control, and community-role
  policy hooks,
- timeline projection metadata for pull feeds, materialized hot timelines,
  lazy fanout, notification timelines, and rebuildable replay,
- durable timeline-item record shape with source fact range, viewer or audience
  scope, policy epoch, consistency watermark, and rebuild command,
- deterministic counter model for replies, reposts, quotes, likes, bookmarks,
  shares, and privacy-preserving view counts,
- tests that search, profile feeds, home feeds, thread reads, media feeds,
  notification reads, counters, and cached timelines all apply the same
  viewer-context visibility rules, legal constraints, and policy epochs.

## v0.32.2 - Media Object Authorization And Processing State Model

Goal: provide database primitives for encrypted media metadata, normalized
variants, captions, takedowns, and short-lived access grants without storing or
serving raw blobs from `skrifheim`.

Deliverables:

- media asset, variant, caption/subtitle, processing job, takedown, and
  access-grant record shapes,
- encryption-domain and key metadata for original uploads, normalized display
  variants, captions, subtitles, transcripts, filenames, alt text, and
  moderation notes,
- processing-state model for quarantined, scanning, normalizing, ready, failed,
  rejected, deleted, and takedown states,
- short-lived media access-grant proof model with hashed token storage,
  expiration, viewer binding, asset state, and post/account visibility
  re-check requirements,
- object-reference policy that prevents account IDs, handles, filenames, email
  addresses, or local paths from leaking through storage keys,
- metadata-only event contract for upload sessions, processing state changes,
  takedowns, and delivery grants; binary media, object keys, wrapped keys,
  hashes, captions, alt text, and user text must not enter event streams,
- tests that stale grants fail after takedown, block, protected-account change,
  post visibility change, asset deletion, moderation state change, legal hold,
  expired consent, or key-domain mismatch.

## v0.32.3 - Social Realtime, Notification, And Rebuildable Event Streams

Goal: model realtime-adjacent social features as rebuildable projections and
metadata-only events, not as a second source of truth.

Deliverables:

- notification fact and projection model for follows, follow requests,
  mentions, replies, reposts, quotes, likes, bookmarks, DMs, poll results,
  community events, moderation decisions, security events, and compliance
  workflows,
- read/unread, clear/delete, snooze, priority, request-inbox, and per-account
  notification filter state,
- realtime hint event shape for WebSocket/SSE gateways that carries only
  identifiers, state, watermarks, and coarse event kinds,
- replay-after-disconnect cursor and watermark model,
- rebuild contract for notification counts, presence/typing hints, hot
  timelines, search indexes, and counters from canonical facts/events,
- tests that realtime hints never expose private content, DMs, post bodies,
  media metadata, account PII, or authorization decisions not visible through a
  normal API read, and that replay after policy changes re-evaluates
  visibility before delivery.

## v0.32.4 - Social Moderation, Appeals, And Safety Labels

Goal: make moderation and user-safety state first-class policy inputs before
social applications depend on timeline/search visibility.

Deliverables:

- moderation report, appeal, trusted-flagger notice, policy label, visibility
  reduction, account restriction, post restriction, and community moderation
  record shapes,
- statement-of-reasons proof model with actor, authority, target, policy
  version, legal basis where applicable, appeal status, and visibility effect,
- stackable safety-label source model that can warn, hide, or reduce content
  without granting third parties raw private data or unrestricted enforcement
  authority,
- user-selectable safety controls for hidden words, muted notification types,
  quote/reply controls, sensitive-media labels, and community-note requests,
- deterministic appeal workflow state machine,
- transparency aggregate fact model that avoids per-user exposure,
- tests that moderation-hidden content cannot leak through search, timelines,
  media reads, notifications, counters, exports without policy, or cached
  projections, and that appeal outcomes preserve immutable moderation history.

## v0.32.5 - Consent, Ads Transparency, And Ranking Explanation Model

Goal: support social applications that must explain ranking and advertising
decisions while respecting EU consent, privacy, and DSA-style transparency
constraints.

Deliverables:

- consent ledger model with purpose, version, grant/withdrawal state, timestamp,
  actor, device context, and legal basis,
- separate consent purposes for necessary service operation, security logging,
  personalized ranking, behavioral measurement, personalized ads, and
  non-essential cookies/local tracking,
- ranking explanation metadata for chronological, following-only, regional,
  contextual, and personalized feed modes,
- ad transparency record model with sponsor identity, campaign/ad identifiers,
  creative reference, targeting category summary, placement, review state,
  political-ad marker, and "why shown" explanation,
- policy hooks that reject sensitive-category targeting, minor profiling,
  consent-dark-pattern assumptions, and personalized delivery after withdrawal,
- aggregate advertiser reporting model that does not expose per-user ad
  histories to advertisers,
- tests for consent withdrawal, non-personalized fallback, ranking explanation
  redaction, political-ad disabled/default-deny behavior, and ad-label
  propagation into API-visible result metadata,
- documentation that ad and ranking primitives are compliance metadata only;
  application-owned ranking algorithms, campaign workflows, payment providers,
  and ad creative review rules stay outside the database core.

## v0.32.6 - Hierarchical Discussion And Scoped Permission Model

Goal: define generic primitives for nested discussion spaces, long-lived
threads, read state, watches, and scoped moderation without absorbing
application-owned discussion schema.

Deliverables:

- hierarchical container model for categories, forums, collections, channels,
  or equivalent application-owned discussion spaces,
- scoped role and capability grant model with inheritance, category/container
  scope, ownership checks, temporary grants, trust-level hooks, and escalation
  prevention,
- content lifecycle model for draft, preview, published, edited, soft-deleted,
  restored, locked, pinned, moved, merged, split, and visibility-reduced
  discussion state,
- sanitized-content provenance model that keeps source body, rendered body
  digest, sanitizer policy/version, renderer policy/version, edit revision,
  and author/actor attribution without making unsafe rendered output a trust
  root,
- read/unread and watch/subscription projection metadata for topics, threads,
  tags, containers, and replies,
- tests that scoped grants do not leak across containers, actors cannot grant
  capabilities they do not hold, sanitized output can be regenerated from
  canonical source and policy version, and soft-deleted or hidden content does
  not leak through search, read-state, counters, or notifications.

## v0.32.7 - Moderation Workflow Simulation And Delayed Action Model

Goal: make moderation workflows auditable, replayable, and policy-bound before
applications depend on reusable staff actions or delayed enforcement.

Deliverables:

- moderation queue, approval queue, warning/points, mute, ban, shadowban,
  delayed job, and reusable action-sequence metadata expressed as generic
  policy records,
- transactional moderation action bundle model where partial failure leaves
  queue, target, and audit state unchanged,
- delayed moderation job model with due time, policy epoch, actor authority,
  pending/completed/failed state, failure redaction, and single-execution
  guarantees,
- moderation replay/simulation model that previews how a rule, trust-level
  change, or anti-abuse filter would affect recent facts without mutating
  production worlds,
- workload-routing metadata for language, category/container, severity,
  conflict-of-interest, and staff availability without exposing private
  content to unauthorized moderators,
- tests that replay cannot mutate source worlds, delayed actions cannot execute
  twice, failed bundles do not partially commit, shadow-hidden content remains
  visible only where policy allows, and staff actions preserve previous/new
  state in tamper-evident audit records.

## v0.33.0 - Crypto-Agile Manifest Signatures

Goal: sign manifests without locking the project to one permanent algorithm.

Deliverables:

- algorithm registry,
- signature envelopes,
- key epoch metadata,
- manifest signature validation API,
- rejected unknown-algorithm tests.

## v0.33.1 - Threshold Signature And Quorum Proof Model

Goal: decide and implement the pre-1.0 threshold-signature posture before audit
proofs, backups, law-pack admission, break-glass approval, and release
operations depend on multi-party authority.

Deliverables:

- distinguish organizational threshold approval records from cryptographic
  threshold signatures,
- dependency-admission decision for any real threshold-signature implementation,
  including `no_std` fit, unsafe boundary, license, maintenance, advisory,
  quantum-resistance posture, test evidence, and platform support,
- explicit fallback model if no threshold-signature crate is admitted: bounded
  quorum multi-signature proofs using admitted signature envelopes,
- threshold key metadata model covering authority scope, tenant, policy epoch,
  crypto epoch, quorum size, participant identifiers, validity window,
  revocation, and rotation,
- rule that raw threshold key shares, recovery fragments, or guardian secrets
  are never stored as ordinary database payloads,
- quorum proof envelope for manifests, law-pack admission, key lifecycle
  ceremonies, declassification, break-glass grants, backup restore, and release
  operations,
- validation API that rejects insufficient quorum, duplicated participants,
  stale policy or crypto epoch, revoked participants, wrong tenant, wrong
  authority scope, and mixed incompatible algorithms,
- tests for threshold/quorum replay rejection, signer substitution, duplicate
  share/signature attempts, and downgrade to single-signer authority,
- documentation that the selected model is a pre-1.0 production requirement,
  while advanced distributed key generation may remain post-1.0 if explicitly
  deferred.

## v0.34.0 - Audit Proof Queries

Goal: prove what was known, under which policy, and from which manifest.

Deliverables:

- fact existence proof skeleton,
- policy epoch proof skeleton,
- confidence derivation proof skeleton,
- manifest root reference,
- audit query output type,
- tests for missing proof material.

## v0.35.0 - Backup And Restore Skeleton

Goal: export and import signed storage roots.

Deliverables:

- backup manifest,
- restore preflight,
- content hash verification,
- policy epoch verification,
- crypto epoch and key-domain verification,
- restore rejection tests.

## v0.35.1 - Backup Format Evolution And Schema Compatibility

Goal: prevent the backup skeleton from becoming a durable format that cannot
survive schema/catalog evolution.

Deliverables:

- backup-format version field and compatibility policy,
- reserved schema-catalog root field even before the full `v0.45.0` catalog
  exists,
- predicate/model registry placeholder binding for restored facts,
- restore preflight behavior for unknown schema roots, missing schema roots,
  forward-incompatible backup versions, and intentionally schema-less scaffold
  backups,
- migration hook shape for future schema catalog upgrades,
- read-only configuration export shape that includes non-secret instance,
  site, policy, storage, and recovery settings while excluding passkeys,
  recovery codes, sessions, installer/bootstrap tokens, password hashes, key
  material, wrapped secrets, and private metadata,
- tests that backup restore cannot silently treat unknown schema contracts as
  compatible,
- tests that configuration export never becomes a complete backup and never
  leaks authentication or encryption material,
- explicit note that `v0.35.0` remains a skeleton and v0.45.0 finalizes the
  catalog-backed compatibility model.

## v0.36.0 - Compromise And Recovery Playbooks

Goal: make key, tenant, compartment, node, backup, and replica compromise explicit.

Deliverables:

- lost-key playbook,
- compromised-key playbook,
- captured-node playbook,
- poisoned source, model, or AI-worker blast-radius quarantine plan,
- stale-replica quarantine plan,
- leaked-backup response plan,
- tests for compromised-key rejection paths.

## v0.37.0 - Rootless Podman Runtime

Goal: run `skrifheim` as a rootless container and a compiled host binary.

Deliverables:

- Containerfile,
- rootless smoke script,
- persistent volume layout,
- portable path policy,
- container release gate.

## v0.38.0 - Configuration And Admin CLI

Goal: make local operation explicit and testable.

Deliverables:

- config file format,
- config validator,
- admin CLI skeleton,
- diagnostics command,
- invalid config tests.

## v0.38.1 - First-Run Bootstrap And Instance Identity Model

Goal: support secure installer/bootstrap workflows for consuming applications
without accepting browser-submitted database secrets, reusable installer
endpoints, or unsafe default administrator assumptions.

Deliverables:

- instance identity metadata with stable instance ID, schema version, setup
  fingerprint, created/updated/completed timestamps, and install status,
- bootstrap state machine for not-started, environment-ready, identity-pending,
  policy-pending, complete, and locked states,
- one-time bootstrap token proof model with keyed token hash, token source
  metadata, expiry, single-use consumption, failed-attempt tracking, and
  automatic lockout after completion,
- bootstrap challenge metadata for first administrator or first owner
  registration that binds to public origin, setup fingerprint, expiry, policy
  epoch, device context, and single-use consumption,
- rule that database credentials, passkey challenges, recovery secrets,
  installer/bootstrap tokens, and TOTP or equivalent fallback secrets are never
  stored as ordinary facts or audit payload,
- environment validation record for storage readiness, key hierarchy readiness,
  public origin, trusted proxy/header policy, HTTPS expectations, and migration
  status,
- audit events for bootstrap progress that never record secrets,
- tests that completed installs cannot be re-run, bootstrap tokens cannot be
  reused, insecure local fixtures require explicit local-only policy, default
  administrator names are not assumed by the database, and bootstrap cannot
  overwrite existing tenants, worlds, or users.

## v0.38.2 - Site Identity, Public Origin, Alias, And Descriptor Model

Goal: provide generic public-site identity and descriptor primitives while
keeping administrative origins, passkey origins, and private mode policy
separate from public rendering aliases.

Deliverables:

- site/instance identity settings for public title, tagline, language, locale,
  timezone, date/time formats, reading/writing defaults, privacy state, and
  public-safe logo/icon/asset references,
- canonical public origin and explicit public alias origin metadata,
- strict separation between public serving aliases and administrator,
  passkey/WebAuthn, bootstrap, API, and trusted internal origins,
- descriptor record model for robots, security contact metadata, feeds,
  sitemaps, OpenSearch-style descriptors, web app manifests, and public asset
  icon references,
- private-site and maintenance-mode policy that forces conservative public
  descriptors regardless of owner overrides,
- audited narrow operations for enabling/disabling redirects, changing
  canonical origin, updating descriptor overrides, and changing search
  visibility,
- tests that public aliases do not expand admin or passkey origins, private and
  maintenance modes override public indexing, descriptor generation uses only
  public-safe fields, and redirects cannot be changed without audit.

## v0.38.3 - Scheduled Operation And Cache Control Model

Goal: make scheduled publishing, descriptor rebuilds, cache purge/warm actions,
and maintenance operations private, audited, policy-bound operations instead of
public cron URLs or application-side shortcuts.

Deliverables:

- scheduled operation metadata with due time, limit, actor/service identity,
  policy epoch, target scope, replay guard, and audit binding,
- publish-due operation model that keeps scheduled content private until the
  operation commits,
- cache eligibility metadata for immutable public assets and public
  projections, with explicit no-store policy for admin, auth, bootstrap,
  preview, API, private, and sensitive responses,
- cache purge/warm operation records for one URL, one asset, related public
  URLs after publish, public media, or all eligible public projections,
- trusted reverse-proxy/header context metadata for operations that depend on
  public origin or cache status,
- tests that scheduled operations require private authenticated authority,
  cannot publish early, cannot execute twice, cannot expose previews to public
  indexes, and cache operations never apply to admin/auth/bootstrap/private
  responses.

## v0.38.4 - Collaborative Text Convergence Model

Goal: choose the collaborative text model before CMS release primitives and
local-first world metadata depend on merge semantics.

Deliverables:

- choose Operational Transform, a CRDT family such as RGA, LOGOOT, YATA, or a
  documented custom model,
- document why the selected model fits `skrifheim` worlds, facts, policy
  labels, offline edits, and signed release workflows,
- define the canonical operation or state format for collaborative text fields,
- define stable actor/device identifiers, causal clocks, tombstones, deletion
  semantics, and compaction rules,
- define how collaborative text state is represented as facts without making
  every keystroke a permanent high-blast-radius fact unless explicitly chosen,
- define deterministic merge/convergence guarantees and conflict review
  behavior,
- define policy behavior for collaborative edits that cross classification,
  compartment, sovereignty, or legal boundaries,
- define whether public releases store materialized text, operation history,
  compacted CRDT state, or a signed projection,
- add fixtures for concurrent insert/delete, offline edit replay, actor
  ordering, tombstone retention, and malicious operation rejection,
- update CMS and local-first milestones to use the selected model rather than
  generic "CRDT fields" language.

## v0.39.0 - CMS World And Release Primitives

Goal: support the first CMS-style atomic publishing model.

Deliverables:

- public/private world split,
- CMS content field model follows the `v0.38.4` collaborative text decision,
- release object,
- publish preflight,
- atomic promote/rollback,
- tests for no half-published state.

## v0.40.0 - CMS Render Dependency Graph

Goal: track causal dependencies for rendered public output.

Deliverables:

- route render graph model,
- content dependency edges,
- invalidation calculation,
- blast-radius invalidation for rendered output,
- public projection boundary,
- tests for precise invalidation.

## v0.41.0 - AI Artifact Provenance

Goal: store AI output as untrusted derived artifacts with provenance.

Deliverables:

- source fact lineage,
- model and prompt hash metadata,
- capability-scoped AI write metadata,
- derivation-cone identity and invalidation,
- artifact invalidation,
- human promotion workflow,
- tests that AI artifacts are not authoritative facts.

## v0.42.0 - Local-First World Metadata

Goal: add policy-filtered local worlds and sync cursors.

Deliverables:

- device-bound world metadata,
- sync cursor model,
- encrypted sync envelope metadata,
- collaborative text field metadata using the selected `v0.38.4` convergence
  model,
- policy-filtered sync tests.

## v0.43.0 - Mission Capsule And Cross-Domain Export Skeleton

Goal: make explicit export/import boundaries for lower-side or disconnected use.

Deliverables:

- mission capsule metadata,
- expiration and device-binding fields,
- export policy proof skeleton,
- signed declassification proof skeleton for every write-down/export to a
  lower classification or release boundary,
- import verification preflight,
- rejected downgrade tests.

## v0.43.1 - Source-State Object And Bundle Backend Model

Goal: define policy-bound primitives for source-state and forge-style
applications without treating CMS publishing as the only application shape.

Deliverables:

- application object identity domain model with object type, hash/digest
  algorithm tag, canonical format version, and tenant/world scope,
- immutable source-state records for objects, state roots, changes, change
  revisions, operations, proof reports, bundles, and releases,
- mutable alias model that points human names such as project heads, review
  worlds, or release channels to immutable state records through transactions,
- proof-carrying bundle manifest model covering object ranges, fact ranges,
  world heads, policy epoch, crypto epoch, schema version, and required
  verification profile,
- remote decision fact types for accept, deny, quarantine, and
  more-evidence-required outcomes,
- quarantine world/state metadata for imported but untrusted bundles,
- tests for object-type confusion, digest-algorithm confusion, stale alias
  rollback, missing object references, and importing without proof material,
- documentation that application-owned source object formats remain owned by
  the consuming application, while `skrifheim` provides durable,
  policy-aware, compliance-aware backend primitives.

## v0.43.2 - Resource-Budgeted Verification Modes

Goal: let source-state applications verify large worlds and bundles through
bounded, policy-declared verification modes without unbounded graph admission
or all-or-nothing full-world scans.

Deliverables:

- verification profile model with bounded-batch, lazy-cone, and full-world
  modes,
- memory, object-count, edge-count, body-size, and parallelism budget metadata,
- remote/bundle preflight that compares declared verification requirements
  with local tenant and deployment budgets before trust or materialization,
- changed-cone proof model for normal small updates,
- full-world proof model for regulated or high-assurance worlds,
- proof-cache metadata for unchanged object and fact subgraphs,
- quarantine or no-materialization decision state when remote requirements
  exceed local budgets,
- tests for oversized manifests, cyclic graphs, missing references,
  stronger-than-local remote verification requirements, proof-cache reuse, and
  refusal to materialize untrusted state.

## v0.43.3 - Operation, Event, Explanation, And Context Records

Goal: define generic records for applications that need append-only operations,
bounded events, deterministic fact compilation, auditable explanations, and
context packs in addition to ordinary query results.

Deliverables:

- operation record model for user-visible mutations, imports, promotions,
  rollbacks, undo actions, vault actions, sync decisions, and administrative
  changes,
- bounded event envelope model that records command/runtime observations
  without making them authoritative facts by itself,
- deterministic event-to-fact compiler contract with missing-source rejection,
  duplicate derivation behavior, and policy epoch binding,
- undo/compensation model where reversing a user action creates a new operation
  and never deletes immutable facts, objects, or history,
- explanation object model with question, deterministic query plan, evidence
  edges, missing evidence, redaction notices, confidence, policy proof,
  optional AI-use marker, and signatures,
- bounded context-pack model for diagnostics, hosted support, and optional AI
  use that includes selected facts, object references, snippets, causal paths,
  redactions, and missing-evidence markers,
- tests that events cannot satisfy policy as facts until compiled, explanations
  cannot hide missing evidence, context packs cannot include unrelated private
  facts or secret material, and undo preserves immutable history.

## v0.43.4 - Sealed Private Realm And Blind Remote Backend Model

Goal: support encrypted source-state hosting where the backend may store,
sync, and verify allowed metadata without learning protected source content.

Deliverables:

- sealed private realm metadata for locked, unlocked, and materialized states,
- public storage ID over encrypted envelopes and private keyed object ID over
  canonical plaintext for membership-leak resistance,
- visible-versus-protected metadata policy for object paths, world names,
  change titles, actor identity, facts, symbols, dependency graphs, and context
  packs,
- recipient slot and key-wrapping metadata compatible with the key hierarchy
  and crypto epoch model,
- encrypted bundle and pack envelope metadata for trusted, blind, and
  split-trust remote modes,
- leak-scan result fact model for plaintext worktree, cache, log, index, and
  backup exposure findings,
- explicit dangerous plaintext export and realm-disable proof model,
- tests that public plaintext hashes are not exposed as storage IDs in sealed
  private mode, protected metadata is not indexed into public projections,
  recipient removal is not treated as retroactive revocation, and blind remote
  imports are verified before decrypt or materialization.

## v0.43.5 - Extension, Theme, And Plugin Capability Boundary Model

Goal: define database primitives for plugin/theme ecosystems without allowing
extension code or templates to bypass policy, leak protected facts, or become
implicit trusted code.

Deliverables:

- extension manifest metadata with version, compatibility window, declared
  hooks, requested capabilities, tenant scope, world scope, data categories,
  and policy epoch,
- host-grant proof model for read, write, emit-event, render, query,
  projection, and admin-like capabilities,
- plugin capability simulator that evaluates requested grants against a
  selected actor, tenant, world, data passport, law pack, and policy epoch
  without mutating state,
- hook invocation record model with bounded inputs, redacted outputs,
  execution result, timeout/memory budget, and failure isolation state,
- theme/template modification metadata with deterministic ordering, CSP/security
  policy version, accessibility/check evidence, and rollback pointer,
- tests that extension manifests cannot request undeclared authority, simulator
  denial matches runtime denial, plugin failures cannot corrupt canonical
  facts, template metadata cannot read arbitrary database state, and unsafe
  render output is not stored as canonical truth.

## v0.43.6 - Import, Migration, And Permission Gap Reporting Model

Goal: support application migrations and legacy imports through dry-run,
policy-checked planning rather than ad hoc direct writes.

Deliverables:

- import plan metadata for source system, schema version, actor mapping,
  object/fact mapping, attachment/media mapping, timestamp policy, and
  trust-level of imported evidence,
- dry-run import report with expected writes, skipped records, conflicts,
  malformed records, missing actors, permission gaps, policy gaps, and legal
  basis gaps,
- import quarantine world/state for accepted-but-untrusted imported data,
- idempotent import checkpoint model that prevents duplicate writes and
  supports resume after failure,
- migration compatibility proof tying schema catalog version, law pack, policy
  epoch, crypto epoch, and retention policy to the import result,
- tests that dry-run produces no writes, permission gaps block promotion,
  imported sanitized/rendered content is treated as untrusted until regenerated
  or verified, duplicate import chunks are idempotent, and attachments/media do
  not bypass encryption-domain policy.

## v0.44.0 - Fuzz And Property Test Baseline

Goal: expand verification before production hardening.

Deliverables:

- parser fuzz target,
- storage frame fuzz target,
- transaction property tests,
- policy decision property tests,
- CI wiring for non-flaky local runs.

## v0.45.0 - Schema Catalog And Versioned Contracts

Goal: make typed data contracts explicit before a production database claim.

Deliverables:

- schema/catalog model,
- schema version identity,
- model and predicate registry,
- compatibility window metadata,
- rejected incompatible schema-change tests.

## v0.46.0 - Retention, Tombstone, And Compaction Policy

Goal: define how old facts, tombstones, segments, and projections age safely.

Deliverables:

- retention policy model,
- tombstone retention rules,
- compaction eligibility checks,
- policy/key-domain preserving compaction rules,
- tests that compaction cannot erase required audit history.

## v0.47.0 - Authenticated API Boundary Hardening

Goal: harden the authenticated server API boundary after the early v0.24.2
integration smoke and before production hardening.

Deliverables:

- local server API skeleton,
- authenticated authority-context extraction for subject, device, and workload,
- mTLS or equivalent identity binding hook for device and workload context,
- service/node identity hook,
- break-glass request authentication and attestation binding using the
  `v0.26.1` access model,
- approval proof binding using the `v0.26.2` operational authority model,
- compatibility check against the v0.24.2 request-context shape so the full API
  does not drift from planner assumptions,
- constant-shape API errors,
- tests for unauthenticated and unauthorized requests.

## v0.48.0 - Resource Governance And Quotas

Goal: prevent one tenant, world, query, or projection from exhausting the database.

Deliverables:

- tenant quota model,
- query budget model,
- projection job limits,
- storage growth limits,
- tests for bounded memory/result/query behavior.

## v0.49.0 - Observability Without Secret Leakage

Goal: make diagnostics useful without leaking facts, keys, compartments, or policies.

Deliverables:

- health report model,
- metrics event model,
- redacted diagnostic output,
- no-secret log tests,
- operator troubleshooting runbook.

## v0.50.0 - Performance And Load Evidence

Goal: establish honest single-node performance limits before 1.0.

Deliverables:

- write/read benchmark harness,
- recovery benchmark harness,
- policy-planner benchmark harness,
- rootless Podman load smoke,
- comparison against early v0.20.2 and v0.23.2 performance evidence to catch
  regressions and validate earlier design decisions,
- documented capacity non-claims.

## v0.51.0 - Production Hardening Candidate

Goal: make the single-node engine ready for compliance-foundation review.

Deliverables:

- no known release-blocking panics,
- recovery runbook,
- backup/restore runbook,
- security control evidence update,
- known-limits review.

## v0.52.0 - Legal And Compliance Passport Foundations

Goal: make standalone legal/compliance planning possible before clustering.

Deliverables:

- node passport model,
- data passport model,
- operation passport model,
- jurisdiction and legal-basis identifiers,
- data-category and processing-mode identifiers,
- request-context markers such as actor, workload, source region, and device posture,
- legal handling for exact sovereignty scopes versus saturated
  multi-jurisdiction scopes,
- tests that unlabeled non-public data cannot be read, exported, indexed, processed by AI, backed up, or planned for movement.

## v0.52.1 - Privacy Rights, Legal Hold, And Deletion Workflow Model

Goal: support social applications that need GDPR-style export, deletion,
retention, deactivation, and legal-hold workflows without breaking immutable
audit or provenance guarantees.

Deliverables:

- privacy export job, delete job, deactivation job, retention job, and legal
  hold record shapes,
- subject-access export proof model that records scope, actor, requester,
  policy epoch, legal basis, redactions, excluded third-party data, and
  generated artifact metadata,
- deletion/tombstone/crypto-erasure decision model separating public removal,
  account deactivation, account deletion, retention-required audit state,
  legal hold, and irreversible key destruction,
- progress and failure-state metadata for long-running export, delete,
  retention, and breach-response jobs,
- processor-log and breach-response evidence record shapes,
- tests that legal hold blocks destructive erasure, deletion removes user
  visibility while preserving required audit facts, export does not leak other
  users' private facts, and retention expiry cannot erase protected audit
  history.

## v0.52.2 - End-To-End Encrypted Message Metadata Boundary

Goal: provide database support for E2EE direct-message products where the
server stores ciphertext and minimal routing metadata only.

Deliverables:

- DM thread, group membership, message ciphertext, attachment reference, read
  receipt, reaction, typing/presence hint, and abuse-report metadata shapes,
- minimal routing metadata policy for sender, recipient, thread identifiers,
  timestamps, delivery state, and device/key epoch references,
- encryption-domain and key-epoch metadata for message ciphertext, encrypted
  attachments, and client-managed key material references,
- search and analytics exclusion rules for message plaintext, private message
  bodies, and protected attachment metadata,
- abuse-report evidence package model that can include user-authorized
  decrypted excerpts or client-provided proofs without making plaintext
  generally queryable,
- tests that DM ciphertext is never indexed as plaintext, realtime events carry
  no message content, exports respect participant visibility, and moderation
  reports do not create a general server-side decrypt path.

## v0.52.3 - Cross-Product Export And Deletion Orchestration Model

Goal: coordinate privacy-rights workflows across multiple product boundaries
without letting one identity/account service directly own or silently delete
another product's data.

Deliverables:

- product exporter contract metadata with product ID, schema version, supported
  scopes, retention rules, legal basis, delivery mode, and failure behavior,
- product deletion contract metadata with dry-run support, finalization flag,
  row/object counts, crypto-erasure capability, legal-hold interaction,
  notification behavior, and idempotent resume key,
- orchestrated export package proof that records every contributing product,
  redactions, missing services, excluded third-party data, generated artifacts,
  delivery channel, expiry, and audit references,
- orchestrated deletion plan that requires final dry-run counts, operator or
  user authority proof, product-by-product before/after audit events, and
  resumable checkpoints,
- operator override model for early export processing, retry, link expiry,
  archive clearing, cleanup, and deletion finalization with fresh verification,
  reason capture, and immutable audit,
- tests that product finalization remains disabled until every required product
  contract is present, dry-run performs no writes, duplicate finalize attempts
  are idempotent, operators never see download tokens, and legal hold blocks
  destructive steps across all products.

## v0.52.4 - Searchable Encrypted Mailbox And Support Thread Model

Goal: support mailbox-style messaging products that intentionally use
server-authorized decryption for search, folders, support communication, and
abuse handling while remaining encrypted at rest with strict audit controls.

Deliverables:

- mailbox account, immutable address identity, alias, thread, participant,
  message, body-version, attachment, receipt, notification, delivery-job,
  folder, and per-user thread-folder membership metadata,
- rule that thread membership and folder placement are separate per-user
  concepts, including forced system views such as support or compliance inboxes
  that cannot be deleted as ordinary user folders,
- encryption-domain model for mailbox key, per-message content key, wrapped
  content key, encrypted body, encrypted draft, encrypted attachment, and
  searchable protected representation,
- server-authorized decrypt/index proof model with actor/service identity,
  purpose, legal basis, mailbox/thread scope, key epoch, policy epoch, and
  audit binding,
- support-thread link model that connects a support case/ticket to a user-facing
  message thread without making the support system own general mailbox data,
- tests that searchable encrypted content cannot be indexed without an
  authorized decrypt/index proof, support views expose only linked/authorized
  threads, forced system views cannot be removed as ordinary folders, and key
  rotation or mailbox deletion updates all wrapped-message metadata.

## v0.52.5 - Server-Blind Collaboration Envelope Model

Goal: support collaboration systems where the database stores workspace,
conversation, delivery, legal-hold, and encrypted-envelope metadata while
clients retain plaintext, message search, and group-key authority.

Deliverables:

- workspace, channel, direct conversation, small-group conversation, thread,
  membership, guest, invitation, device, read-cursor, delivery-state, reaction,
  pin, saved-item, draft, and scheduled-send metadata shapes,
- encrypted collaboration envelope model with opaque workspace/conversation
  identifiers, sender device identity, group epoch, envelope kind, ciphertext
  size bounds, key-package or welcome-message references, and replay-resistant
  sequencing metadata,
- strict server-blind boundary that rejects plaintext message bodies, channel
  names, direct conversation names, file names, search terms, client group
  secrets, raw invite addresses, raw invite tokens, notification bodies, and
  push-token material from canonical server-side records,
- client-owned search and local-state boundary model where decrypted indexes,
  local caches, profile-scoped state, and group-key state remain outside the
  server trust boundary,
- retention, legal-hold, export, and admin-policy model for ciphertext and
  metadata that does not imply server plaintext recovery; authorized exports
  must record whether decryption requires client-side or separately approved
  administrator tooling,
- plugin/integration metadata model for capability grants, webhook/event
  subscriptions, command invocations, workflow state, and outbound integration
  queues without giving server plugins plaintext or group-key material,
- tests that ciphertext envelopes accept only bounded opaque encrypted payloads,
  delivery/read cursors cannot carry snippets, server-side search/indexing is
  denied for client-owned plaintext, legal hold preserves ciphertext without
  declassification, and plugin metadata cannot request plaintext or key access
  by default.

## v0.53.0 - Law Pack Metadata And Admission

Goal: define how signed legal and compliance policy packs enter the system.

Deliverables:

- law-pack metadata model,
- issuer and authority references,
- validity window and version rules,
- review and approval status model,
- compliance test-case metadata,
- rollback-prevention and stale-pack rejection tests.

## v0.54.0 - Legal Operation Decision Engine Skeleton

Goal: require legal decisions before local access, replication, export, backup, AI processing, indexing, or failover plans.

Deliverables:

- legal operation request type,
- legal transfer request extension for future cluster boundaries,
- allow, constrained-allow, approval-required, and deny decision types,
- legal-basis proof skeleton,
- safe alternative suggestions such as redacted, aggregate, hash-witness, remote-query, and compute-to-data,
- deterministic denial-shape tests,
- tests for denied CMS reads from disallowed request contexts.

## v0.55.0 - Sovereign Placement Intent Compiler

Goal: compile declared placement intent into lawful single-node and future-cluster planning metadata.

Deliverables:

- placement intent model for worlds, projections, indexes, backups, AI workers, and public releases,
- jurisdiction and compliance constraints in placement metadata,
- derived-data passport inheritance rules,
- stale-placement marker when policy, law-pack, key, or data-passport epochs change,
- tests for denied cross-boundary placement.

## v0.56.0 - 1.0 Release Candidate

Goal: freeze the 1.0 feature set and run final release evidence.

Deliverables:

- release-candidate notes,
- complete security review checklist,
- rootless Podman release gate,
- final CMS integration checklist,
- no new feature work without explicit deferral decision.

## v1.0.0 - Production World Database For CMS Integration

Goal: first serious production-ready `skrifheim`.

Deliverables:

- durable single-node fact/world engine,
- strict transaction semantics,
- policy-aware query planning,
- causal blast-radius invalidation and quarantine support,
- key hierarchy and lifecycle,
- signed declassification proof model,
- capability-scoped AI derivation cones,
- propagated confidence fused with mandatory access control,
- encrypted WAL, segments, indexes, projections, backups, exports, and audit logs,
- query-result classification,
- legal/compliance passport foundations,
- law-pack metadata admission,
- legal operation and transfer decision skeleton,
- sovereign placement intent compiler,
- compromise and recovery playbooks,
- schema catalog and versioned contracts,
- retention, tombstone, and compaction policy,
- authenticated API boundary,
- resource governance and quotas,
- observability without secret leakage,
- performance and load evidence,
- tamper-evident manifests,
- rootless Podman deployment,
- backup/restore,
- secure first-run bootstrap and instance identity primitives,
- public origin, alias, descriptor, scheduled operation, and cache-control
  primitives,
- read-only secret-free configuration export,
- CMS release primitives,
- public/private world split,
- render dependency tracking,
- social graph visibility and timeline projection primitives,
- media metadata authorization and processing-state model,
- social moderation, safety-label, consent, ad-transparency, and ranking
  explanation primitives,
- hierarchical discussion, scoped permission, sanitized-content provenance,
  read-state, watch/subscription, and moderation workflow primitives,
- privacy rights, legal-hold, deletion, and E2EE message metadata boundaries,
- platform identity, shared-account, product-boundary, guardian-consent, and
  derived-claim primitives,
- cross-product export/deletion orchestration, searchable encrypted mailbox
  support-thread boundaries, and server-blind encrypted collaboration envelope
  support,
- source-state object and proof-carrying bundle backend primitives,
- resource-budgeted verification modes,
- operation, event, explanation, and context-pack records,
- sealed private realm and blind remote backend model,
- extension/theme capability boundary and import/migration dry-run model,
- AI artifact provenance,
- complete release runbook,
- security review PASS for exact commit.

Non-goals for 1.0:

- replacing every SQL database,
- distributed consensus as default,
- full multi-cell Hyve clustering,
- automatic cross-region tunnel management,
- automatic legal/compliance failover,
- unsandboxed plugins,
- AI as authoritative truth,
- exotic hardware requirement.

## Post-1.0 Cluster Roadmap

The full Hyve cluster fabric starts after the first production single-node
database unless explicitly re-scoped. Each item still needs its own clean stop,
pentest handoff, and release notes.

### v1.1.0 - Local Cell Cluster Runtime

Goal: run a local sovereign cell with multiple nodes.

Deliverables:

- node registry,
- local shard/range assignment,
- local consensus skeleton,
- local failover preflight,
- cell health model,
- tests for one-node loss inside a cell.

### v1.2.0 - Hyve Control Plane

Goal: make topology and placement intent first-class database state.

Deliverables:

- control-plane metadata store,
- world and projection registry,
- placement planner,
- health monitor,
- lease manager,
- policy, key, and law-pack epoch tracking,
- tests for control-plane proposals that cannot bypass local vetoes.

### v1.3.0 - Policy-Scoped Tunnel Fabric

Goal: open encrypted database tunnels with identity and legal scope.

Deliverables:

- node identity handshake,
- signed peer maps,
- tunnel policy model,
- operation and data-passport binding on streams,
- replication and health streams,
- tests that denied labels cannot cross an otherwise healthy tunnel.

### v1.4.0 - Geo Replication And Witness Roles

Goal: replicate safely across cells where policy permits it.

Deliverables:

- commit-log shipping,
- snapshot shipping,
- Merkle repair,
- hot and async secondary modes,
- hash-only witness/notary role,
- tests for stale, divergent, and witness-only replicas.

### v1.5.0 - Compliance-Aware Failover

Goal: fail over per world, data class, and legal basis.

Deliverables:

- failover eligibility planner,
- promote, read-only, sealed, and deny outcomes,
- authority-approval hooks,
- split-brain prevention,
- tests for public data failover and sensitive data failover denial.

### v1.6.0 - Cluster Compliance Autopilot

Goal: reconcile actual placement, tunnels, replicas, keys, and law packs against lawful desired state.

Deliverables:

- drift detector,
- lawful remediation proposals,
- tunnel freeze and replica seal actions,
- key-rotation proposal hooks,
- compliance incident record,
- tests for law-pack, node-passport, and certification drift.

### v1.7.0 - Multi-Region CMS Operation

Goal: serve CMS-style public reads locally while keeping private data and publishing controls lawful.

Deliverables:

- public projection replication,
- private draft home-region placement,
- regional authentication compartment planning,
- two-region publish approval workflow,
- public release pointer promotion,
- tests for country-loss survival where policy permits it.
