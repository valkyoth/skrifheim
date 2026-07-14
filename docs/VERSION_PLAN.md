# skrifheim Version Plan

Status: planning document

Tags use:

```text
v0.N.0      milestone release
v0.N.P      patch/fix release
v1.0.0      first serious production-ready database
v1.N.0      optional application-extension releases over the stable database
v2.N.0      Hyve cluster releases after core and extension API stability
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

Application-inspired verticals must not be folded into the mandatory core
database unless they are genuinely generic primitives. When a feature is mainly
needed to support a product family, it should land as an optional extension
crate such as `skrifheim-ext-social`, `skrifheim-ext-messaging`, or
`skrifheim-ext-forge`. Extension crates may depend on stable core crates, but
core crates must not depend on extension crates. Each extension crate needs its
own release gate, pentest handoff, dependency admission, legal/compliance
review, and proof that it cannot bypass core policy, encryption-domain,
provenance, or audit rules.

Before any extension milestone starts, every proposed primitive must pass an
extension primitive review:

- classify the primitive as mandatory core, generic extension helper, or
  product-owned schema,
- prove why anything marked mandatory core is required by the world database
  itself and not just by one application family,
- keep product-owned schema inside the extension or consuming application,
- verify dependency direction from extension to core only,
- verify the primitive composes existing authorization, legal/compliance,
  encryption-domain, key-lifecycle, provenance, audit, and release-evidence
  semantics instead of redefining them,
- record deny, quarantine, redaction, rebuild, audit, and legal/compliance tests
  expected for the primitive,
- document the decision in the extension release notes before implementation.

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

- Rust stable `1.97.0` pinned.
- Focused workspace crates.
- `scripts/checks.sh`.
- CI, dependency policy, security policy, release notes.
- Implementation, version, modularity, threat-model, toolchain, and optional
  publishing extension target docs.

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

## v0.18.5 - Release Evidence And Dependency Tree Baseline

Goal: make release evidence stricter before manifests, recovery, and
cryptographic storage boundaries become durable claims.

Deliverables:

- release-readiness gate that requires committed release notes, committed
  permanent pentest digest, clean root `PENTEST.md` removal, current tag
  absence, and reviewed-commit binding for final report commits,
- SBOM generation and non-empty SBOM validation in the release gate,
- normal, all-features, and security-relevant feature dependency tree snapshots
  for every release where feature gates or dependency sets change,
- runtime/host dependency policy that proves `no_std` core crates do not gain
  network, filesystem, clock, TLS, parser, database, or process-runtime
  dependencies by accident,
- optional-boundary policy that proves parser, host-I/O, crypto-provider,
  container, and future network/API features stay opt-in and do not leak into
  default core builds,
- current-tool and current-crate check evidence recorded in release notes
  whenever dependency, toolchain, or GitHub Actions versions change,
- tests for the release-readiness gate and dependency-policy scripts so the
  gate itself cannot silently regress.

## v0.18.6 - Cross-Platform Portability Baseline

Goal: prevent Linux-only, Unix-only, or x86-only assumptions from entering the
storage and runtime design before manifests and recovery become durable.

Deliverables:

- explicit platform support matrix for Linux, Windows, macOS, BSD, Android,
  iOS, x86_64, AArch64, RISC-V, and other realistic Rust targets,
- CI or local-gate compile checks for OS-neutral `no_std` core crates on at
  least Linux, Windows, macOS, and BSD-compatible targets where the toolchain
  can run them,
- CI or local-gate compile checks for architecture-neutral core crates on
  x86_64 and AArch64, with RISC-V tracked as a non-blocking future target until
  the runner/toolchain support is practical,
- host-storage adapter plan that replaces the current Unix-only
  `skrifheim-storage-host` scaffold with explicit Unix, Windows, and BSD/macOS
  file-opening, permission, symlink/reparse-point, directory-sync, and atomicity
  semantics,
- fail-closed unsupported-platform behavior for any host adapter that cannot
  provide equivalent security semantics,
- release-gate check rejecting `target_arch`, `target_feature`, `std::arch`,
  or `core::arch` use in database crates unless a portable baseline and
  optional fast-path admission record exists,
- durable-format review proving WAL, segment, manifest, backup, audit, export,
  and future network encodings use explicit endianness, explicit lengths,
  checked conversions, and no native pointer-width or alignment assumptions,
- documentation that Linux-specific io_uring, direct-I/O, mmap, fsync variants,
  or filesystem hints are optional performance paths only and never required
  for correctness or security.

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

## v0.31.1 - Optional Extension Crate Boundary

Goal: keep application-family support out of the mandatory database core unless
the feature is truly generic.

Deliverables:

- extension-crate policy for product-family primitives, including naming,
  dependency direction, default-build behavior, release gates, and pentest
  requirements,
- extension primitive review template that every `v1.x` extension release must
  complete before implementation,
- workspace layout plan for optional crates such as `skrifheim-ext-social`,
  `skrifheim-ext-messaging`, `skrifheim-ext-forge`, and future application
  families,
- rule that extension crates depend on `skrifheim` core APIs and may not cause
  any core crate to depend on an extension crate,
- rule that extension crates cannot redefine authorization, legal/compliance,
  encryption-domain, key-lifecycle, provenance, audit, or release-evidence
  semantics; they can only compose them,
- gate proving default core builds remain independent of application-family
  extension crates,
- tests or compile checks proving extension crates can be omitted by a
  deployment that does not need that product family,
- documentation review that rewrites app-specific schema names into generic
  relationship, object, workflow, projection, and policy-extension primitives
  before implementation starts.

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

## v0.32.1 - Extension Deferral And Core API Proof Gate

Goal: confirm that product-family primitives remain outside the mandatory
pre-1.0 world database.

Deliverables:

- default workspace build proves no `skrifheim-ext-*` crate is required by the
  core database,
- release-gate check that every proposed extension primitive is classified as
  mandatory core, generic extension helper, or product-owned schema before
  implementation,
- dependency-direction gate that rejects core crates depending on extension
  crates,
- one compile-only stub extension that depends on the stable public core API
  without adding product schema to core,
- documentation that publishing, messenger, forum, forge/source-state,
  relationship/feed/media, mailbox, collaboration, and cluster workflows are
  post-1.0 tracks with their own pentest gates,
- release-gate check that pre-1.0 deliverables cannot add product-specific
  schema to core crates without a deferral decision.

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

## v0.35.2 - Local Snapshot And Rollback Retention Model

Goal: provide local Btrfs/Snapper-style restore points without treating
rollback as replication, backup, audit deletion, or silent production rewind.

Deliverables:

- snapshot-root metadata for manifest root, world head, policy epoch, crypto
  epoch, schema/catalog root, retention class, creation reason, triggering
  operation, actor/service authority, tenant, and protected data scope,
- automatic restore-point policy for before migration, before compaction,
  before bulk import, before key rotation, before release/publish, scheduled
  hourly/daily/monthly retention, and explicitly named operator checkpoints,
- locked rollback archive world model that can hold old facts, manifests,
  segments, blobs, tombstones, and key-domain references as protected recovery
  evidence without making them mutable production state,
- restore modes for read-only inspection, recovery-world fork, simulation
  replay, and signed promotion back into production through normal world
  promotion policy,
- signed rollback proof model with requester, approver or quorum proof, reason,
  source snapshot root, target recovery/archive/production world, legal basis,
  policy epoch, crypto epoch, affected fact ranges, and audit-log binding,
- policy rules that rollback reads still use the active or explicitly selected
  historical policy epoch, and that restoration cannot resurrect privacy-erased
  or legally deleted material unless a legal hold, retention rule, or explicit
  break-glass style override authorizes the recovery,
- purge/override design for exceptional removal of rollback-protected material,
  requiring scoped authority, reason, legal basis, quorum or local fallback
  approval, audit proof, and preferably crypto-erasure over raw byte deletion,
- compaction eligibility rules that treat protected snapshot roots as live
  references, account for space retained only by rollback policy, and refuse to
  compact segments/blobs/keys needed by non-expired rollback archives,
- tests that rollback cannot bypass policy, cannot delete or rewrite audit
  facts, cannot silently become production, cannot resurrect erased data
  without explicit authorization, and prevents compaction from dropping
  protected snapshot material.

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

## v0.38.4 - Extension API Compatibility Freeze

Goal: freeze the public core APIs that optional post-1.0 extension crates will
compile against, without implementing any product-family extension in the
mandatory database.

Deliverables:

- public API review for facts, worlds, labels, policy proofs, query plans,
  storage roots, encryption domains, audit records, and legal/compliance
  passports,
- compile-only extension fixture that depends on the core APIs without being a
  default workspace dependency,
- documentation of which APIs are stable for v1.1.0 through v1.6.x extension
  crates,
- release-gate check proving optional extension fixtures can be omitted from
  the production core build,
- explicit deferral notes that collaborative text, publishing releases, render
  graphs, forum/discussion, messenger, feed/media, mailbox, collaboration, and
  forge/source-state behavior starts after v1.0.0.

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
- extension-compatible placeholder metadata for future collaborative text
  extension crates without choosing or implementing the convergence algorithm
  before v1.1.1,
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

## v0.43.1 - Extension Trust Boundary And Import Deferral

Goal: define the trust boundary for future extension, plugin, source-state, and
import workflows without implementing those application-family systems before
the core database is stable.

Deliverables:

- core capability vocabulary for future extension crates: read, write,
  emit-event, render, query, projection, import, export, and admin-like
  operations,
- policy rule that extension, plugin, theme, source-state, import, and
  migration behavior starts after v1.0.0 unless explicitly promoted as generic
  core functionality,
- compile-only omission test proving the core database does not require
  plugin/theme/import/source-state crates,
- documentation mapping source-state/forge work to v1.4.0 through v1.4.4,
  import/migration work to v1.4.4, and extension/plugin/theme work to the
  relevant post-1.0 extension crate,
- tests that the core API rejects undeclared capabilities and policy-bypassing
  extension proofs even before real extension crates exist.

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

## v0.52.1 - External Source Lock And Evidence Model

Goal: prevent legal, compliance, cryptographic, storage-format, and external
conformance claims from being implemented from memory or shifting upstream
sources.

Deliverables:

- source-lock manifest format for external standards, law packs, compliance
  packs, cryptographic specifications, file-format references, fixture
  repositories, and imported conformance suites,
- exact source metadata for issuer, jurisdiction or standards body, URL,
  revision/tag/date, hash, license, validity window, local reference-store
  location, and whether the source is normative, advisory, test-only, or
  deprecated,
- policy that a milestone changing law-pack, legal-operation, storage-format,
  crypto-format, parser, or conformance behavior must pin the consulted source
  material before implementation and name it in release notes,
- local reference-store sync/check script shape that can verify pinned external
  materials without vendoring large upstream repositories into the main repo,
- ambiguity workflow requiring deny/defer/more-evidence decisions when sources
  conflict, are stale, or lack test fixtures,
- source-matrix documentation mapping each external claim to tests, fixtures,
  implemented version, non-claims, and next-review trigger,
- tests that release validation fails when source-lock entries are missing,
  stale, malformed, or not referenced by the relevant law-pack/compliance or
  parser/conformance milestone.

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
- tests for denied optional publishing-extension reads from disallowed request
  contexts.

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
- final optional extension integration checklist,
- no new feature work without explicit deferral decision.

## v1.0.0 - Production World Database

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
- SBOM, dependency-tree, source-lock, and release-evidence gates,
- local snapshot and rollback retention with locked archive/recovery worlds,
- rootless Podman deployment,
- backup/restore,
- secure first-run bootstrap and instance identity primitives,
- read-only secret-free configuration export,
- platform identity, shared-account, product-boundary, guardian-consent, and
  derived-claim primitives,
- resource-budgeted verification modes,
- operation, event, explanation, and context-pack records,
- extension API compatibility freeze proving optional application-family crates
  can compile against core without being mandatory dependencies,
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

## Post-1.0 Extension Roadmap

The first post-1.0 releases prove the stable world database through optional
compiled-in extension crates. Each extension release has its own clean stop,
pentest handoff, release notes, dependency review, and extension primitive
review. Core crates must not depend on extension crates.

### v1.1.0 - Publishing/CMS Extension Crate

Goal: add `skrifheim-ext-publishing` as the first optional extension crate.

Deliverables:

- optional crate wiring that can be omitted from a core-only build,
- public/private/review world mapping over core worlds,
- publishing release object and release proof model,
- publish preflight using core policy, legal, encryption-domain, and audit
  checks,
- atomic promote/rollback through world promotion rather than mutable published
  flags,
- public projection boundary for published materialized outputs,
- tests for no half-published state, denied private-world reads, denied
  missing-approval releases, and extension omission from core builds.

### v1.1.1 - Publishing Collaborative Text Model

Goal: choose and implement the text-convergence model required by the optional
publishing extension.

Deliverables:

- selected OT, established CRDT, or documented custom convergence model,
- operation/state representation with actor/device identifiers, causal clocks,
  tombstones, deletion semantics, and compaction rules,
- policy behavior for collaborative edits crossing classification,
  compartment, sovereignty, or legal boundaries,
- release representation decision: materialized text, operation history,
  compacted state, or signed projection,
- fixtures for concurrent insert/delete, offline edit replay, actor ordering,
  tombstone retention, and malicious operation rejection.

### v1.1.2 - Publishing Render Dependency Graph

Goal: track causal dependencies for rendered public output in the optional
publishing extension.

Deliverables:

- route/render graph model,
- content and projection dependency edges,
- invalidation calculation and blast-radius invalidation for rendered output,
- cache eligibility metadata for immutable public assets and public
  projections,
- scheduled publish/cache purge/warm operations as private authenticated
  operations,
- tests for precise invalidation, no private-preview leakage, and cache
  operations never applying to admin/auth/bootstrap/private responses.

### v1.2.0 - Messenger/Private-Channel Extension Crate

Goal: add `skrifheim-ext-messaging` for ciphertext-only private-channel
metadata.

Deliverables:

- optional crate wiring that can be omitted from a core-only build,
- private-channel thread, group membership, message ciphertext, attachment
  reference, delivery/read signal, interaction signal, presence hint, and
  abuse-report metadata shapes,
- minimal routing metadata policy for sender, recipient, thread identifiers,
  timestamps, delivery state, and device/key epoch references,
- encryption-domain and key-epoch metadata for message ciphertext, encrypted
  attachments, and client-managed key material references,
- search and analytics exclusion rules for message plaintext and protected
  attachment metadata,
- abuse-report evidence package model with user-authorized decrypted excerpts
  or client-provided proofs without making plaintext generally queryable,
- tests that ciphertext is never indexed as plaintext, realtime events carry no
  message content, exports respect participant visibility, and moderation
  reports do not create a general server-side decrypt path.

### v1.3.0 - Forum/Discussion Extension Crate

Goal: add `skrifheim-ext-discussion` for nested discussions, scoped
permissions, and moderation workflows.

Deliverables:

- optional crate wiring that can be omitted from a core-only build,
- hierarchical container model for categories, forums, collections, channels,
  or equivalent application-owned discussion spaces,
- scoped role and capability grants with inheritance, container scope,
  ownership checks, temporary grants, trust-level hooks, and escalation
  prevention,
- content lifecycle model for draft, preview, published, edited, soft-deleted,
  restored, locked, pinned, moved, merged, split, and visibility-reduced states,
- sanitized-content provenance with source body, rendered-body digest,
  sanitizer/renderer policy version, edit revision, and actor attribution,
- read/unread and watch/subscription projection metadata,
- tests that scoped grants do not leak across containers, actors cannot grant
  capabilities they do not hold, sanitized output can be regenerated, and
  hidden content does not leak through search, read-state, counters, or
  notifications.

### v1.3.1 - Forum Moderation Workflow Simulation

Goal: make discussion moderation replayable, auditable, delayed, and
policy-bound.

Deliverables:

- moderation queue, approval queue, warning/points, mute, ban, shadow-hide,
  delayed job, and reusable action-sequence metadata expressed as policy
  records,
- transactional moderation action bundle model where partial failure leaves
  queue, target, and audit state unchanged,
- delayed moderation job model with due time, policy epoch, actor authority,
  failure redaction, and single-execution guarantees,
- replay/simulation model that previews rule, trust-level, or abuse-filter
  changes without mutating production worlds,
- tests that replay cannot mutate source worlds, delayed actions cannot execute
  twice, failed bundles do not partially commit, and staff actions preserve
  previous/new state in tamper-evident audit records.

### v1.4.0 - Forge/Source-State Extension Crate

Goal: add `skrifheim-ext-forge` for policy-bound source-state and proof-carrying
bundle workflows.

Deliverables:

- optional crate wiring that can be omitted from a core-only build,
- application object identity domain model with object type, hash/digest
  algorithm tag, canonical format version, tenant, and world scope,
- immutable source-state records for objects, state roots, changes, revisions,
  proof reports, bundles, releases, and operation records,
- mutable alias model pointing human names such as project heads, review worlds,
  and release channels to immutable state through transactions,
- proof-carrying bundle manifest covering object ranges, fact ranges, world
  heads, policy epoch, crypto epoch, schema version, and required verification
  profile,
- remote decision facts for accept, deny, quarantine, and
  more-evidence-required outcomes,
- tests for object-type confusion, digest-algorithm confusion, stale alias
  rollback, missing object references, and importing without proof material.

### v1.4.1 - Forge Resource-Budgeted Verification

Goal: verify large source-state worlds and bundles through bounded declared
verification modes.

Deliverables:

- verification profile model with bounded-batch, lazy-cone, and full-world
  modes,
- memory, object-count, edge-count, body-size, and parallelism budgets,
- remote/bundle preflight comparing declared requirements with local tenant and
  deployment budgets,
- changed-cone proof model and full-world proof model,
- proof-cache metadata for unchanged object and fact subgraphs,
- quarantine or no-materialization decision when requirements exceed budgets,
- tests for oversized manifests, cyclic graphs, missing references,
  stronger-than-local verification requirements, proof-cache reuse, and refusal
  to materialize untrusted state.

### v1.4.2 - Forge Sealed Private Realm

Goal: support encrypted source-state hosting where the backend stores, syncs,
and verifies allowed metadata without learning protected source content.

Deliverables:

- sealed private realm metadata for locked, unlocked, and materialized states,
- public storage ID over encrypted envelopes and private keyed object ID over
  canonical plaintext for membership-leak resistance,
- visible-versus-protected metadata policy for object paths, world names,
  change titles, actor identity, facts, symbols, dependency graphs, and context
  packs,
- recipient slot and key-wrapping metadata compatible with the key hierarchy,
- encrypted bundle and pack envelope metadata for trusted, blind, and
  split-trust remote modes,
- leak-scan result facts and dangerous plaintext export proof model,
- tests that protected metadata is not indexed into public projections and
  blind remote imports are verified before decrypt or materialization.

### v1.4.3 - Forge Import And Migration Planning

Goal: support imports through dry-run, policy-checked planning rather than ad
hoc direct writes.

Deliverables:

- import plan metadata for source system, schema version, actor mapping,
  object/fact mapping, attachment/media mapping, timestamp policy, and
  trust-level of imported evidence,
- dry-run import report with expected writes, skipped records, conflicts,
  malformed records, missing actors, permission gaps, policy gaps, and legal
  basis gaps,
- import quarantine world/state for accepted-but-untrusted imported data,
- idempotent import checkpoint model,
- migration compatibility proof tying schema catalog version, law pack, policy
  epoch, crypto epoch, and retention policy to the import result,
- tests that dry-run produces no writes, permission gaps block promotion,
  duplicate import chunks are idempotent, and attachments/media do not bypass
  encryption-domain policy.

### v1.5.0 - Relationship/Feed Extension Crate

Goal: add `skrifheim-ext-relationship` for high-volume relationship graphs and
feed-like projections.

Deliverables:

- optional crate wiring that can be omitted from a core-only build,
- generic relationship edge classes for subscription, membership, denial,
  preference, interaction, reference, endorsement, collection, and report-like
  signals,
- viewer-context visibility planner for public, limited-audience, group-scoped,
  member-scoped, deleted, tombstoned, and visibility-reduced content states,
- policy hooks for relationship denial, audience restriction, user preference,
  content-control, and scoped-role decisions,
- timeline projection metadata for pull feeds, materialized hot timelines,
  lazy fanout, notification timelines, and rebuildable replay,
- deterministic counter model for relationship-derived interactions and
  privacy-preserving aggregate views,
- tests that search projections, actor feeds, audience feeds, thread reads,
  counters, media feeds, notification reads, and cached timelines all apply the
  same visibility, legal, and policy-epoch rules.

### v1.5.1 - Media Authorization Extension

Goal: add encrypted media metadata, processing states, takedowns, and
short-lived access grants without storing or serving raw blobs from core.

Deliverables:

- media asset, variant, caption/subtitle, processing job, takedown, and
  access-grant record shapes,
- encryption-domain and key metadata for original uploads, normalized display
  variants, captions, transcripts, filenames, alt text, and moderation notes,
- processing-state model for quarantined, scanning, normalizing, ready, failed,
  rejected, deleted, and takedown states,
- short-lived media access-grant proof model with hashed token storage,
  expiration, viewer binding, asset state, and visibility re-checks,
- object-reference policy that prevents identifiers, filenames, emails, or
  local paths from leaking through storage keys,
- tests that stale grants fail after takedown, visibility changes, deletion,
  moderation state changes, legal hold, expired consent, or key-domain
  mismatch.

### v1.5.2 - Realtime Hint And Notification Extension

Goal: model realtime-adjacent extension features as rebuildable projections and
metadata-only events.

Deliverables:

- generic notification fact and projection model for relationship changes,
  references, interactions, private-channel signals, voting/response signals,
  group events, moderation decisions, security events, and compliance
  workflows,
- read/unread, clear/delete, snooze, priority, request-inbox, and notification
  filter state,
- realtime hint event shape for WebSocket/SSE gateways carrying only
  identifiers, state, watermarks, and coarse event kinds,
- replay-after-disconnect cursor and watermark model,
- rebuild contract for notification counts, presence/typing hints, hot
  timelines, search indexes, and counters,
- tests that realtime hints never expose private content, ciphertext, content
  bodies, media metadata, account PII, or hidden authorization decisions.

### v1.5.3 - Moderation, Safety, Consent, And Transparency Extension

Goal: support relationship/feed extensions that need moderation, safety labels,
consent, transparency, ranking, advertising, recommendation, or promoted-content
explanations.

Deliverables:

- generic report, appeal, trusted-notice, policy label, visibility reduction,
  subject restriction, object restriction, and scoped moderation records,
- statement-of-reasons proof model,
- stackable safety-label source model,
- user-selectable safety controls for filtered terms, notification classes,
  interaction controls, sensitive-object labels, and review-note requests,
- consent ledger with purpose, version, grant/withdrawal state, timestamp,
  actor, device context, and legal basis,
- ranking/transparency metadata for chronological, subscription-only,
  regional, contextual, personalized, advertising, recommendation, and promoted
  modes,
- tests for consent withdrawal, non-personalized fallback, transparency
  redaction, sensitive-category targeting denial, and label propagation into
  API-visible result metadata.

### v1.6.0 - Privacy, Mailbox, And Collaboration Extension Track

Goal: add optional crates for cross-product privacy workflows, searchable
encrypted mailbox use cases, and server-blind collaboration envelopes.

Deliverables:

- `skrifheim-ext-privacy` export/delete/deactivation/retention/legal-hold job
  model,
- subject-access export proof model with scope, actor, requester, policy epoch,
  legal basis, redactions, excluded third-party data, and artifact metadata,
- deletion/tombstone/crypto-erasure decision model,
- cross-product export/deletion orchestration with product contracts, dry-run
  support, finalization guard, resumable checkpoints, and immutable audit,
- `skrifheim-ext-mailbox` searchable encrypted mailbox metadata with
  server-authorized decrypt/index proof model,
- `skrifheim-ext-collaboration` server-blind envelope metadata where clients
  retain plaintext, search, and group-key authority,
- tests that legal hold blocks destructive erasure, exports do not leak third
  parties, server-side search needs decrypt/index proof, and collaboration
  envelopes reject plaintext, filenames, search terms, client group secrets,
  invite tokens, notification bodies, and push-token material.

## Post-Extension Cluster Roadmap

Hyve clustering starts only after the core world database is production-ready
and the first optional extension tracks have proven the public API. Each item
still needs its own clean stop, pentest handoff, and release notes.

### v2.0.0 - Local Cell Cluster Runtime

Goal: run a local sovereign cell with multiple nodes.

Deliverables:

- node registry,
- local shard/range assignment,
- local consensus skeleton,
- local failover preflight,
- cell health model,
- tests for one-node loss inside a cell.

### v2.1.0 - Hyve Control Plane

Goal: make topology and placement intent first-class database state.

Deliverables:

- control-plane metadata store,
- world and projection registry,
- placement planner,
- health monitor,
- lease manager,
- policy, key, and law-pack epoch tracking,
- tests for control-plane proposals that cannot bypass local vetoes.

### v2.2.0 - Policy-Scoped Tunnel Fabric

Goal: open encrypted database tunnels with identity and legal scope.

Deliverables:

- node identity handshake,
- signed peer maps,
- tunnel policy model,
- operation and data-passport binding on streams,
- replication and health streams,
- tests that denied labels cannot cross an otherwise healthy tunnel.

### v2.3.0 - Geo Replication And Witness Roles

Goal: replicate safely across cells where policy permits it.

Deliverables:

- commit-log shipping,
- snapshot shipping,
- Merkle repair,
- hot and async secondary modes,
- hash-only witness/notary role,
- tests for stale, divergent, and witness-only replicas.

### v2.4.0 - Compliance-Aware Failover

Goal: fail over per world, data class, and legal basis.

Deliverables:

- failover eligibility planner,
- promote, read-only, sealed, and deny outcomes,
- authority-approval hooks,
- split-brain prevention,
- tests for public data failover and sensitive data failover denial.

### v2.5.0 - Cluster Compliance Autopilot

Goal: reconcile actual placement, tunnels, replicas, keys, and law packs
against lawful desired state.

Deliverables:

- drift detector,
- lawful remediation proposals,
- tunnel freeze and replica seal actions,
- key-rotation proposal hooks,
- compliance incident record,
- tests for law-pack, node-passport, and certification drift.

### v2.6.0 - Multi-Region Publishing Extension Operation

Goal: serve publishing-style public reads locally while keeping private data and
publishing controls lawful.

Deliverables:

- public projection replication,
- private draft home-region placement,
- regional authentication compartment planning,
- two-region publish approval workflow,
- public release pointer promotion,
- tests for country-loss survival where policy permits it.
