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
7. When the maintainer reports the pentest is green, Codex writes the permanent
   report at `security/pentest/<tag>.md`.
8. Codex commits the implementation state, release metadata, and permanent
   pentest report together.
9. The project waits for GitHub Actions.
10. If GitHub fails, Codex fixes the issue, updates tests/docs/release
    evidence when relevant, commits again, and the project waits for GitHub
    again.
11. If GitHub is green, the maintainer tells Codex. Tagging and pushing tags
    happen only when explicitly requested.

Root `PENTEST.md` is temporary scratch input. It must not be committed, and the
release metadata validator fails while it exists.

## v0.1.0 - Repository Foundation

Goal: initialize the serious Rust workspace and policy baseline.

Deliverables:

- Rust stable `1.97.1` pinned.
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

Goal: admit and implement production-ready cryptographic primitives before
WAL v2, block tables, manifests, checkpoints, or recovery instantiate them.
This milestone must not claim final production storage encryption; the final
WAL and segment AEAD envelopes are bound to the concrete storage formats in
`v0.18.11` and `v0.18.12`.

Deliverables:

- dependency-admission decision for the production SHA-3/SHAKE digest engine
  and AEAD implementation, including license, maintenance, advisory, `no_std`
  fit, unsafe-code boundary, platform support, and test evidence,
- entropy and CSPRNG provider admission for nonce generation, key generation,
  randomized staging names, salts, replay nonces, and any future random
  identifiers, including platform support, failure behavior, fork/VM rollback
  considerations, test-only deterministic provider boundaries, and `no_std`
  compatibility,
- configurable digest-strength policy wired to actual SHA3-256, SHA3-384,
  SHA3-512, SHAKE256-256, and SHAKE256-512 computation,
- production `ContentDigest`, `ManifestDigest`, and `WorldIdentityDigest`
  computation APIs,
- generic AEAD envelope primitive with algorithm ID, nonce strategy,
  associated-data transcript input, key ID, crypto epoch, encryption domain,
  plaintext length, ciphertext length, and versioned domain-separation tag,
- transcript-builder APIs for future WAL, block, segment, manifest, backup,
  export, projection, AI, and audit envelopes without freezing the final
  storage-frame fields in this milestone,
- separate closed registries for digest, AEAD, KDF/wrapping, signature, and
  quorum-proof suites, so digest algorithms cannot be confused with signing
  algorithms and each context admits only the suites it actually supports,
- provider trait contracts that consume canonical versioned transcripts and
  return sealed result types such as authenticated plaintext, verified
  signature, signed commit, or verified digest proof rather than caller-forged
  booleans,
- host/provider boundary rule that software crypto, HSM, KMS, TPM, threshold
  guardian, entropy, and freshness-anchor implementations stay outside pure
  protocol crates and cannot be required by `no_std` core crates,
- rule that no crate may bypass the admitted digest abstraction with a direct
  production hash dependency after this milestone; scaffold identifiers must
  be migrated or explicitly marked non-authoritative,
- storage metadata privacy split between a minimal plaintext outer header and
  encrypted inner headers for WAL frames and immutable segments,
- rule that tenant IDs, world or world-revision IDs, classification,
  compartment, policy epoch, key IDs, transaction ranges, content digests, and
  other sensitive storage metadata must move into encrypted inner metadata
  unless an explicit public-header exception is documented,
- opaque filename and optional padding-bucket decision for deployments that
  need copied-disk metadata minimisation,
- separation of plaintext semantic identity, ciphertext integrity digest, and
  keyed equality-safe content reference semantics,
- crypto-erasure granularity decision covering per-object or erasure-group
  DEKs, wrapped-key indexes, backup key-slot deletion, compaction and
  re-encryption protocol, snapshot/legal-hold exceptions, and proof that every
  readable key slot was removed,
- domain-separated key derivation contract from the key hierarchy to WAL,
  segment, projection, backup, export, AI artifact, and audit-log data keys,
- key-hierarchy ancestry fix so compartment, segment, and data-key metadata
  retain deployment and region scope below tenant and cannot validate under the
  wrong regional parent in multi-region deployments,
- nonce uniqueness and crash-recovery analysis for append-only WAL writes and
  immutable segment writes,
- immediate WAL-v1 stopgap hardening before any later storage milestone uses
  the scaffold writer: host writers must poison themselves after any append,
  partial write, flush, or sync error; existing tails must be scanned before
  append; boolean sync options must be replaced with explicit
  `DurabilityMode`; and append APIs must expose durable, failed, and ambiguous
  outcomes,
- durable LSN receipt, transaction idempotency key, and commit-status lookup
  scaffold so ambiguous sync results can be retried without duplicating a
  transaction before WAL v2 exists,
- tests proving production paths fail closed when entropy is unavailable,
  deterministic test RNGs cannot be compiled into production profiles, and
  generated nonces cannot repeat across restart/crash scenarios covered by the
  storage model,
- production secret-use API decision removing generic public secret-retention
  closures from real provider paths, replacing them with purpose-specific
  secret types and provider operations that return fixed public result types,
- secret-cleanup non-claims for stack copies, registers, optimizer-created
  copies, process dumps, swap, panic paths, and privileged memory reads, plus
  future codegen/fault-path evidence requirements once real providers exist,
- associated-data tests proving ciphertext cannot be replayed across tenant,
  compartment, world, WAL, segment, projection, backup, export, AI, or audit
  domains,
- corrupt ciphertext, swapped header/footer, wrong key, wrong epoch, wrong
  domain, truncated body, and replay rejection tests,
- golden compatibility fixtures for the admitted digest outputs, AEAD envelope
  primitive, nonce/transcript encoding, and WAL-v1 durability outcome records,
- explicit rule that CRC64 remains a structural corruption check only and that
  AEAD authentication plus signed/keyed manifests are the production integrity
  boundary,
- release-gate check that no durable storage path can claim final tamper
  resistance until the admitted primitives are instantiated by the concrete
  WAL, block-table, manifest, and segment formats.

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
  absence, and matching release version in the permanent pentest report,
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
  gate itself cannot silently regress,
- practical release provenance covering source commit, toolchain, dependency
  lockfile, container image digests, SBOM, release gate output, pentest digest,
  and GitHub Actions status, with bit-for-bit reproducibility explicitly
  non-claimed until binary/package publication exists,
- release-gate check that release evidence does not include secrets or local
  machine paths.

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

## v0.18.7 - Non-Rollbackable Freshness Anchor Contract

Goal: prevent a copied or replaced database directory from authenticating as
the newest valid state before manifests and startup recovery choose durable
roots.

Deliverables:

- `AnchoredDatabaseRoot` contract covering database identity, generation,
  manifest digest, policy epoch, key-state digest, audit root, and previous
  anchor digest,
- `FreshnessAnchor` trait shape with `current` and compare-and-advance
  semantics,
- supported reference provider decision, initially remote witness client,
  TPM-backed host adapter, or another explicitly reviewed non-rollbackable
  mechanism,
- host-boundary implementation rule for freshness providers, including
  authenticated and versioned transport, secure credential storage, redacted
  diagnostics, and no direct dependency from `no_std` core crates on network,
  TPM, HSM, filesystem, or process APIs,
- idempotent compare-and-advance implementation for `AnchoredDatabaseRoot`,
  including retry behavior for already-advanced, stale, concurrent, and
  ambiguous outcomes,
- timeout, unavailable-provider, degraded-mode, and fail-closed startup
  semantics for production profiles,
- provisioning, rotation, replacement, revocation, and disaster-recovery
  procedure for anchor credentials and anchor service identity,
- witness equivocation detection using signed or MACed generations, previous
  anchor digest, database identity, manifest digest, key-state digest, audit
  root, and policy epoch,
- permanent-unavailable-anchor recovery policy that allows only explicit
  historical inspection, recovery-world fork, or operator-approved
  re-provisioning without silently replacing the active anchor,
- production profile rule requiring at least one non-rollbackable freshness
  mechanism: TPM monotonic/NV state, HSM-backed counter, remote witness or
  transparency service, WORM/offline signed operator checkpoint, or
  threshold-held external checkpoint,
- explicit non-claim that signed manifests and AEAD prove authenticity of a
  state, not freshness of the newest state by themselves,
- recovery workflow distinction between active startup and historical rollback
  inspection,
- crash-ordering rule for anchor advancement versus manifest publication,
  including how to avoid accepting stale roots and how to recover from a crash
  after one side advances but before the other side is durable,
- rule that active startup must fail closed if the selected manifest generation
  is older than the anchor or if the anchor cannot be checked under a
  production profile,
- tests for stale root rejection, generation regression, anchor digest chain
  mismatch, missing production anchor, explicit historical recovery open,
  compare-and-advance idempotence, stale advance rejection,
  unavailable-provider fail-closed behavior, timeout behavior, witness
  equivocation, provider replacement, and disaster-recovery opening rules.

## v0.18.8 - Chained Audit Log Root Contract

Goal: make audit evidence append-only and anchorable before manifests record
storage roots.

Deliverables:

- chained audit record contract with tenant, sequence, previous-record digest,
  event digest, manifest root, policy epoch, and signatures,
- canonical audit-event transcript covering event kind, tenant, actor,
  target-specific identifiers, trusted time evidence, policy epoch, crypto
  epoch, manifest/WAL transaction root, and protected metadata before any
  audit signature is accepted,
- event-kind requirement matrix so fact writes name fact and transaction
  identifiers, world promotion names source/target/base revisions, policy
  decisions name policy identity, key events name key state, and break-glass
  events name scope and approval proof,
- audit stream generation, record sequence, previous-record digest,
  batch/segment root, and external receipt model for deletion, omission, and
  reordering detection,
- audit segment root model included in manifests and freshness anchors,
- separate audit key or scoped audit-signing provider unavailable as an
  unrestricted main-database key, with uniform behavior for stale, revoked, or
  missing audit-signing authority,
- transactionally coupled audit barrier requiring the audit record digest or
  audit outbox digest to be committed in the same WAL transcript as any
  mandatory-audit operation,
- mandatory-audit operation rule: protected reads, policy changes,
  declassification, key lifecycle changes, break-glass grants, backup restore,
  and release operations must fail closed if required audit emission cannot be
  made durable,
- deletion and reordering rejection rules for audit record sequences,
- audit compaction rule that preserves verifiable sequence roots and external
  anchor references,
- tests for removed audit record detection, reordered audit record detection,
  stale manifest-root binding, missing mandatory audit durability, and
  anchor-root mismatch.

## v0.18.9 - Scoped Key Release And No-God-Mode Boundary

Goal: make the "no god-mode database" claim enforceable before real storage
encryption depends on key release.

Deliverables:

- `ScopedKeyProvider` boundary that releases decrypt/sign/unwrap authority only
  for a specific encryption domain, tenant, compartment, purpose, policy epoch,
  workload identity, and bound policy proof,
- rule that the database process never holds root, deployment, or unrestricted
  tenant keys in production profiles,
- privilege-separated key-service, KMS, HSM, or equivalent provider profile
  for high-assurance deployments,
- explicit high-assurance option for separate processes or instances per
  classification or compartment domain,
- tests or mock-provider fixtures proving a process with one scoped proof
  cannot unwrap keys for another tenant, compartment, policy epoch, purpose, or
  workload,
- documentation that key hierarchy metadata alone is not least privilege unless
  key release is enforced outside the main database process.

## v0.18.10 - Trusted Time And Monotonic Sequence Types

Goal: remove ambiguous timestamp semantics before audit, expiry, freshness,
transactions, and approval windows depend on time.

Deliverables:

- separate wall-clock, trusted-time evidence, commit sequence, and hybrid
  logical timestamp types,
- documented units, UTC behavior, leap-second stance, uncertainty bounds, and
  clock source requirements for security expiry,
- rule that transaction ordering uses monotonic/logical sequence evidence, not
  unauthenticated wall-clock timestamps,
- trusted-time evidence model for attestation expiry, approval validity,
  freshness-anchor checkpoint time, audit event time, and break-glass windows,
- fixed-width counter exhaustion policy for LSNs, WAL generations, transaction
  IDs, commit sequences, file numbers, manifest generations, crypto epochs,
  lifecycle event sequences, anchor generations, and backup generations,
- checked arithmetic rule for every ordering or identity counter; wrapping,
  implicit truncation, and identifier reuse are fail-closed defects,
- exhaustion thresholds before numeric maximum, rollover mechanisms where safe
  such as new log or storage incarnation, and fail-closed behavior where
  rollover cannot preserve identity, ordering, nonce uniqueness, or deletion
  safety,
- exhaustion interaction model for nonces, freshness anchors, checkpoints,
  manifests, backups, rollback roots, and crypto-erasure proofs,
- tests for stale/future trusted-time evidence, uncertainty overflow, inverted
  validity windows, wall-clock/logical-time type confusion, replayed approval
  windows, MAX-1 and MAX counter boundaries, replay after exhaustion, attempted
  reuse after rollover, and fail-closed exhaustion without rollover.

## v0.18.11 - Physical Storage Layout Decision

Goal: choose the physical engine shape before manifests harden around opaque
whole-segment bodies.

Deliverables:

- storage architecture decision record choosing an LSM-oriented canonical fact
  layout, hybrid LSM plus compact copy-on-write metadata trees, or another
  explicitly justified structure,
- canonical key ordering for facts, world revisions, transaction batches,
  schema roots, causal edges, supersession edges, invalidation edges, and
  snapshot visibility indexes,
- logical block size decision, likely 4 KiB or 16 KiB, independent of host page
  size and suitable for future optional direct-I/O adapters,
- block-aligned outer segment layout with padded outer header, independently
  readable encrypted blocks, and footer/trailer rules that preserve
  header/body/footer mismatch detection,
- block format with offset table or restart array, record count, bounded
  decompressed length, compression algorithm ID, per-block checksum, AEAD
  envelope, and schema/format marker,
- block AEAD associated-data transcript binding each encrypted block to
  database ID, storage generation, file/table/segment identity, block ordinal
  or canonical offset, block kind, format version, feature version, tenant,
  encryption domain, policy epoch, crypto epoch, compression format, and
  authenticated original length,
- segment metadata authentication rule binding every semantic header and
  footer field, including transaction range, policy ID, key ID, segment kind,
  encrypted-inner-header digest, body layout, and footer location, into AEAD
  associated data or the keyed table/segment commitment,
- sealed segment-verification boundary replacing publicly implementable
  verifier booleans with a provider registry that returns a typed
  `VerifiedSegment`-style proof naming the approved suite, provider, table
  commitment, and authenticated metadata transcript,
- legacy footer migration API split: legacy unbound footer decoding returns a
  distinct migration-only type and cannot become a normal footer or verified
  segment without explicit migration validation,
- canonical digest encoding rule requiring all unused digest bytes in fixed
  digest slots to be zero and rejecting multiple byte encodings for the same
  logical header or footer,
- checksum-presence format flag so CRC/checksum value zero is not overloaded
  as "missing"; CRC remains only an accidental-corruption signal outside the
  AEAD/table-commitment boundary,
- keyed table/segment commitment covering expected block count, ordered block
  identities, data block root, index block root, filter block root,
  range-tombstone block root, metadata block root, first and last internal key,
  sequence range, file length, footer location, format/features, database
  generation, file identity, commitment algorithm/version, encryption domain,
  policy epoch, crypto epoch, and key-slot identity,
- key-slot rotation semantics for table commitments: the commitment binds the
  immutable table-DEK identity and encryption domain, while mutable wrapping
  key locators or KEK slots are authenticated through manifest/wrapped-key
  metadata so routine KEK rewrap does not rewrite blocks or recompute the
  table commitment; DEK rotation requires re-encryption and a new commitment,
- manifest reference rule requiring the authenticated manifest to name the
  table/segment commitment, not only a file path or per-block digest set,
- durable-format compatibility contract for block and segment framing: major
  and minor version semantics, required versus safely ignorable feature bits,
  minimum reader and writer versions, canonical encoding rules,
  unknown-field behavior, forward-read policy, and backward-read policy,
- compression-before-encryption design for table/segment blocks, including
  adaptive per-block compression format IDs, authenticated original length,
  decompression memory/time budgets, dictionary versioning, domain-local
  dictionaries only, and a prohibition on attacker-controlled and secret
  material being co-compressed in a way that creates compression oracles,
- sparse index and filter format decision, including partitioned Bloom or XOR
  filters and workload-adaptive filter budgets,
- range tombstone encoding and semantics, including fragmentation,
  overlap/coalescing, snapshot visibility, compaction behavior, legal deletion
  interaction, and protected rollback-root interaction,
- physical partition key decision covering tenant, region, classification,
  compartment, key epoch, and policy compatibility class so policy isolation
  does not create uncontrolled small-file amplification,
- small-file aggregation and domain-local compaction plan for high-cardinality
  policy deployments,
- content-addressed blob threshold and encryption-domain boundary for large
  values that should not be inlined into fact records,
- production reader shape that supports block-level sparse reads, streaming
  verification, bounded decompression, and cache integration instead of
  relying on whole-segment materialization beyond the temporary scaffold path,
- tests or format fixtures for block alignment, malformed offset tables,
  restart bounds, compression-size limits, filter false-positive boundaries,
  range tombstone overlap, snapshot-visible range deletion, cross-domain
  partition rejection, cross-file block replay, cross-offset block replay,
  cross-generation block replay, wrong-block-kind substitution,
  duplicate-block replay, stale-block replay, missing final block, missing
  middle block, reordered block, substituted index/filter block, truncated
  table, valid-prefix attack, and attacker-rebuilt footer,
- tests for KEK rewrap without table rewrite, DEK rotation requiring
  re-encryption, previous wrapping-slot removal, crash during rewrap,
  compromised wrapping key versus compromised table DEK, and crypto-erasure of
  every readable key slot,
- golden compatibility fixtures for block table headers, restart arrays,
  compression metadata, range tombstones, sparse indexes, filters, and
  encrypted inner/outer segment framing, exercised by current and previous
  readers where previous-reader behavior is defined.

## v0.18.12 - WAL v2 And Torn-Tail Recovery

Goal: replace the scaffold WAL envelope with an authenticated, ordered,
repairable log format before manifests and recovery depend on it.

Deliverables:

- WAL v2 header with database ID, log incarnation, WAL generation, LSN,
  transaction ID, transaction frame ordinal, expected transaction frame count,
  previous-frame digest, header authentication, and commit transcript root,
- durable-format compatibility contract for WAL-v2 frames and commit records:
  major/minor version semantics, required versus safely ignorable feature bits,
  minimum reader and writer versions, canonical encoding rules,
  unknown-field behavior, forward-read policy, and backward-read policy,
- WAL-v2 AEAD envelope instantiated from `v0.18.3` over the concrete v2 frame
  and commit-record fields, with associated data binding database ID, log
  incarnation, WAL generation, LSN, transaction ID, frame ordinal, previous
  frame digest, expected frame count, commit transcript root, crypto epoch,
  policy epoch, tenant, and encryption domain,
- compression-before-encryption decision for WAL sub-batches, including
  authenticated original length, bounded decompression, explicit compression
  format IDs, domain-local dictionary rules, and tiny/incompressible-frame skip
  behavior,
- explicit rule for single-domain transactions versus atomic cross-domain
  transactions; if cross-domain atomic writes are needed, use a commit-root
  record containing separately encrypted domain sub-batches rather than
  weakening key isolation,
- ordered commit-record transcript binding all transaction frames and their
  digests,
- startup scanner that finds the last authenticated frame boundary,
  distinguishes recoverable torn tail from interior corruption, truncates or
  rotates the WAL safely, and fsyncs the repaired file and parent directory,
- writer preflight that validates an existing WAL tail before appending,
- migration rule from the `v0.18.3` WAL-v1 stopgap states to WAL v2 status
  records without accepting ambiguous commits as durable by default,
- redo-only ordinary failure model where pre-commit abort records are not
  required for transactions that never became durable,
- golden compatibility fixtures for WAL-v2 headers, encrypted frames,
  compressed and uncompressed sub-batches, commit records, repaired tails, and
  commit-status records, exercised by current and previous readers where
  previous-reader behavior is defined,
- tests for header mutation, frame deletion, duplication, reordering,
  transaction frame omission, commit-root mismatch, appending after corrupt
  tail, writer reuse after append error, ambiguous sync result, idempotent
  retry, torn header, torn body, torn commit record, and repaired-tail replay.

## v0.18.13 - Storage Crash Ordering And Failure Injection Harness

Goal: build reusable failure-injection and crash-oracle infrastructure before
manifest publication and checkpoint pruning become trusted. Concrete failpoint
coverage for future implementations is accepted in the milestones that
introduce those implementations.

Deliverables:

- formal crash-ordering state machine for current WAL append, current segment
  publication, future manifest write/swap, checkpoint, WAL pruning, audit-root
  publication, and freshness-anchor advancement,
- TLA+/PlusCal or equivalent lightweight model scaffold for WAL, manifest,
  checkpoint, and freshness-anchor ordering,
- deterministic failpoint harness for writes, vectored writes, short writes,
  interrupted syscalls, sync, link/rename, manifest swap, WAL truncation,
  directory fsync, ENOSPC, quota exhaustion, EIO, and fsync failure,
- database-directory lease rule covering active writers, staged-file cleanup,
  manifest changes, migration, checkpointing, and obsolete-file deletion so
  cleanup cannot race a writer or publisher,
- supported-filesystem matrix for local filesystems, explicit unsupported
  stance for network filesystems until locking, rename, fsync, directory sync,
  and durability semantics are proven,
- filesystem-specific power-cut tests using loopback, dm-flakey, or equivalent
  where practical,
- reusable subprocess crash oracle that kills the process at persistence
  failpoints and compares recovered state with an in-memory oracle,
- current WAL and segment failure tests that exercise the reusable harness
  without requiring future memtable, block table, manifest, compaction,
  checkpoint, or audit-root implementations,
- torn-sector and corrupted-block matrix fixture framework for WAL, future
  block table, manifest, and audit roots,
- long-running randomized state-machine scaffold comparing append, recover,
  compact, checkpoint, and rollback behavior with a reference model as those
  features become available,
- release-gate smoke mode for deterministic failure-injection cases that are
  stable enough for normal local and CI runs.

## v0.18.14 - Transcript Dependency And Engine Ownership Model

Goal: define durable transcript dependencies, publication order, and
cross-crate ownership before manifests, audit roots, fact signatures, and world
revisions can accidentally form circular commitments.

Deliverables:

- transcript dependency DAG covering transaction intent, fact signing
  transcripts, audit intent, WAL commit roots, commit receipts, world
  revisions, manifests, freshness anchors, and external anchor receipts,
- publication-order rule: actor fact signatures bind pre-state intent and
  base world revision, WAL commits bind fact digests and audit-intent digest,
  commit receipts bind resulting world revisions, manifests incorporate
  completed world/audit roots, and freshness anchors advance over completed
  manifest digests without embedding a circular anchor receipt in the same
  manifest,
- predecessor declaration requirement for every durable transcript, with a
  release-gate check or test fixture rejecting self-referential root graphs and
  cycles in signature, audit, manifest, and anchor dependencies,
- canonical placeholder ID/transcript types for schema roots, world revisions,
  commit receipts, audit roots, and manifest roots so early milestones can
  refer to them before full implementations land,
- `skrifheim-engine` or `skrifheim-txn` ownership decision for commit
  coordination, MVCC, world-head compare-and-swap, verified ingest,
  audit-coupled commit, idempotency status, and cross-crate invariants,
- `skrifheim-catalog` ownership decision for schema roots, predicate metadata,
  security-label resolution, PII/data-category annotations, policy
  compatibility classes, and catalog-derived query planning metadata,
- `skrifheim-projection-api` ownership decision for committed-generation feeds,
  projection capability declarations, projection journals, and rebuild
  contracts,
- `skrifheim-crypto-provider-api` ownership decision for sealed digest, AEAD,
  signature, KDF/wrapping, entropy, key-provider, and freshness-anchor provider
  contracts without host implementations,
- dependency-direction rule that storage, facts, worlds, policy, audit, query,
  catalog, engine, projection API, and provider API crates do not become
  mutually dependent; cross-subsystem invariants live in the engine/transaction
  coordinator instead of the main binary crate or storage crate,
- tests or compile checks proving the main `skrifheim` crate remains
  orchestration, provider implementations stay behind host boundaries, and
  core crates cannot import the engine in a way that creates cycles.

## v0.19.0 - Manifest And Checkpoint Format

Goal: record the durable storage root.

Deliverables:

- manifest structure,
- durable-format compatibility contract for the manifest: major/minor version
  semantics, required versus safely ignorable feature bits, minimum reader and
  writer versions, canonical encoding rules, unknown-field behavior,
  forward-read policy, and backward-read policy,
- minimal plaintext outer manifest header plus encrypted/authenticated inner
  manifest using the primitives admitted in `v0.18.3`,
- keyed authentication before any manifest contents are trusted; asymmetric
  signatures and quorum/non-repudiation remain later `v0.33.x` concerns,
- manifest-key bootstrap contract: outer-header fields are untrusted
  key-location hints only, database identity and manifest generation determine
  or locate the manifest key, and key-provider scope is constructed from
  trusted outer context plus configured database identity before encrypted
  metadata is trusted,
- bounded candidate-key lookup rule: outer manifest bytes may select only from
  configured database-local provider scopes, may try only a bounded number of
  candidate slots, and must never derive arbitrary tenant, provider, or key
  scope solely from disk-controlled bytes,
- uniform error behavior for nonexistent, unauthorized, wrong-database,
  compromised, destroyed, crypto-erased, and stale manifest key slots so
  startup does not become a KMS existence oracle,
- key-provider lookup rate limits, timeout budget, cancellation behavior, and
  bounded previous-key fallback attempts during rotation,
- opaque manifest key-slot identifier model when ordinary key IDs would reveal
  sensitive metadata, including rejection of attacker-controlled redirection to
  another valid key, database, generation, or provider scope,
- dual-key and previous-key fallback rules for manifest-key rotation, including
  crash between key activation, manifest publication, checkpoint update, and
  freshness-anchor advancement,
- missing, compromised, rotated, destroyed, crypto-erased, and permanently lost
  manifest-key behavior, including which paths may open read-only historical
  state and which fail closed,
- database identity, storage generation, previous-manifest digest,
  feature-version binding, checkpoint identity, and freshness-anchor reference
  in the authenticated manifest transcript,
- manifest/checkpoint replay, rollback, and substitution rejection rules,
- checkpoint record ordering contract binding checkpoint headers to manifest
  generation, WAL prefix root, audit root, recovery watermark, policy/schema
  epochs, key epoch, storage generation, and log-chain state before a
  checkpoint can participate in startup recovery,
- metadata-confidentiality rules for table inventories, tenant/domain
  information, world heads, schema roots, policy epochs, key-state digests, and
  audit-root references,
- rule that the manifest is the sole authority for live immutable tables,
  world heads, schema roots, key state, WAL checkpoint, audit root, freshness
  anchor, and protected roots; directory scans may discover candidates only for
  recovery, quarantine, or cleanup workflows,
- version-edit format for adding/removing live tables, advancing world heads,
  publishing schema/key/audit roots, and recording obsolete files,
- storage-directory identity and single-writer lease contract so two database
  processes cannot concurrently advance the same manifest/WAL directory,
- checkpoint LSN,
- segment list,
- digest strength profile field,
- full-width world identity, content, and manifest digest fields,
- policy epoch field,
- crypto epoch field,
- audit root field from the chained audit-log contract,
- freshness-anchor generation and anchor digest fields from `v0.18.7`,
- encryption domain inventory,
- atomic manifest/checkpoint/WAL-pruning crash-ordering specification,
- stale lease recovery rule bound to database identity, process identity,
  storage generation, freshness anchor, and trusted time evidence,
- world-id collision detection that rejects an existing `WorldId` for a
  different `(tenant_id, kind, depth, parent, name)` tuple,
- rejection of manifests whose full-width digest profile does not match the
  active deployment policy,
- rejection of manifests whose audit root, policy epoch, key-state digest, or
  freshness-anchor generation cannot be reconciled,
- manifest validation tests,
- golden compatibility fixtures for manifest headers, version edits,
  checkpoint records, storage-directory identity, lease records, audit-root
  references, freshness-anchor references, and live-table inventories,
- current-reader and previous-reader fixture tests for manifest compatibility
  behavior, unknown feature bits, required feature bits, and canonical encoding,
- failure-injection acceptance using the `v0.18.13` harness for manifest
  write, manifest swap, checkpoint write, WAL-pruning decision, directory sync,
  and crash-after-swap recovery,
- tests for concurrent opener rejection, stale lease recovery, wrong database
  identity, wrong storage generation, lease/anchor mismatch, manifest key-slot
  substitution, cross-database key redirection, previous-key fallback, lost
  manifest key, compromised manifest key, malicious key-lookup amplification,
  KMS existence-oracle behavior, and rotation crash recovery.

## v0.19.1 - Storage Spine Vertical Slice

Goal: prove the narrow end-to-end database kernel path immediately after
manifests, before startup recovery, transactions, query execution, projections,
and extensions depend on the storage format.

Deliverables:

- one complete vertical path:
  `WriteBatch -> WAL v2 -> durable barrier -> memtable -> immutable table
  flush -> manifest swap -> restart recovery -> point read -> domain-local
  compaction`,
- minimal `InternalKey`, `WriteBatch`, `MemTable`, `Snapshot`, `TableBuilder`,
  `TableReader`, block iterator, merge iterator, filter policy,
  manifest/version edit, version set, compaction job, obsolete-file manager,
  database identity/superblock, environment/filesystem abstraction, and
  resource governor,
- snapshot lifetime and garbage-collection watermark model with oldest active
  snapshot tracking, maximum snapshot/read-transaction age, explicit snapshot
  leases, per-tenant pin quotas, abandoned-client handling, and expired-snapshot
  failure behavior,
- compaction, iterator, and recovery SLO baselines that are measurable only
  after the spine exists: read/write/space amplification, recovery MiB/s
  through table replay, compaction debt, stall time, protected-root liveness,
  and maximum memory per iterator/snapshot; cache-hit and cache-residency SLOs
  move to `v0.20.8`,
- recovery test proving committed batches reappear after restart and
  uncommitted or non-durable batches do not,
- point-read test proving a single fact can be located without reading a whole
  segment/table body,
- compaction equivalence test proving compacted output preserves snapshot
  visibility, tombstones, policy domains, and rollback-protected roots,
- failure-injection acceptance using the `v0.18.13` harness for memtable flush,
  table publication, manifest version edit, restart recovery, point read after
  crash, and compaction output publication,
- release-gate smoke that exercises this full spine with a tiny fixture.

## v0.20.0 - Startup Recovery Integration

Goal: provide full production startup recovery, building on the narrow
storage-spine restart recovery from `v0.19.1`.

Deliverables:

- recovery loader,
- manifest selection,
- freshness-anchor check before selecting an active manifest under production
  profiles,
- WAL replay from checkpoint,
- memtable/table/version-set reconstruction through the `v0.19.1` storage
  spine,
- full operational startup diagnostics for selected manifest, anchor
  generation, WAL checkpoint, key-state digest, audit root, schema root,
  rollback roots, degraded files, quarantine state, and recovery mode,
- fail-closed startup when the newest valid local manifest is older than the
  external anchor,
- explicit historical recovery workflow for rollback roots that may be opened
  for inspection, recovery-world forks, or simulation, but may not silently
  replace the active freshness anchor,
- storage-backed world ancestry verification before promotion or rollback
  execution can be authorized,
- internal storage-validated promotion and rollback preflight construction that
  can set the currently private storage-validation marker only after durable
  ancestor traversal succeeds,
- corrupted manifest rejection,
- missing-key and compromised-key rejection,
- explicit operational recovery modes for normal active open, read-only
  degraded open, historical inspection, recovery-world fork, simulation open,
  and operator-approved anchor/provider re-provisioning,
- graceful shutdown protocol that stops write admission, cancels or drains
  in-flight transactions, resolves group commits and ambiguous commits, stops
  background publication safely, chooses optional flush/checkpoint behavior,
  orders audit emission and freshness-anchor advancement, and releases the
  storage-directory lease only after durable state is safe,
- bounded shutdown timeout and forced-termination behavior, including what
  state recovery must expect if the process is killed at each shutdown stage,
- restart-storm protection for repeated startup failures, including bounded
  repair attempts, quarantine of repeatedly failing roots, operator-visible
  diagnostics, and fail-closed active open under production profiles,
- non-destructive restart repair rule: automated repair works only on staged
  or copy-on-write state, preserves the original manifest, WAL, and damaged
  files immutably until authorized disposition, never repeatedly truncates or
  mutates the only recovery copy, and fails closed when disk-space exhaustion
  prevents evidence preservation,
- repair provenance and audit records for every automated repair outcome, with
  destructive salvage requiring explicit operator authority and a chosen
  recovery point,
- failure-injection tests for every shutdown stage, restart after forced
  termination, repeated startup failure, and directory-lease release ordering,
- tests for crashes during repair, repeated repair attempts, rollback to the
  untouched original evidence set, failed evidence-copy staging, and rejected
  destructive salvage without operator authority,
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

- microbenchmarks for WAL append/read, segment write/read, raw block I/O,
  recovery scanning, and startup manifest selection over small, medium, and
  large fixture sets,
- measurements for mirrored 256-byte segment footer overhead on very small
  segments and normal segment sizes,
- policy-token fixed-slot scan benchmark for common authority/label shapes,
- first measurable pre-transaction SLOs for WAL bytes per appended batch, fsyncs
  per durable append, raw block read/write throughput, recovery scan MiB/s,
  manifest selection time, maximum startup scan time, policy-token scan time,
  and memory used by recovery scanning,
- benchmark methodology template recording hardware, filesystem, mount options,
  drive-cache mode, dataset distribution, compression ratio, encryption
  profile, warm/cold cache state, concurrency, run variance, and rootless
  container bind-volume versus overlay-filesystem behavior,
- rootless Podman recovery smoke with realistic persisted volume layout,
- documented thresholds that are non-claims but catch obvious regressions,
- decision record on whether footer layout, block sizing, token-slot bounds, or
  recovery sequencing need adjustment before transaction durability depends on
  them.

## v0.20.3 - Production Signature Engine And Canonical Fact Transcript

Goal: admit production signature providers and define exactly what a fact
signature means before durable transaction commit accepts signed facts. The
final durable ingest gate is integrated in `v0.20.10` after schema roots and
world revisions exist.

Deliverables:

- production signature-provider admission and implementation for the selected
  classical baseline and any selected post-quantum or hybrid profile,
  including license, maintenance, advisory, `no_std` fit, unsafe boundary,
  platform support, side-channel posture, key parsing, and test evidence,
- Ed25519 verification decision if retained as the classical baseline, with
  strict public-key parsing, canonical signature parsing, invalid-key tests,
  malleability rejection, batch-verification failure semantics, and provider
  fault behavior,
- ML-DSA, SLH-DSA, or other post-quantum signature implementation decision, or
  an explicit temporary post-quantum signature non-claim until a reviewed
  provider is admitted,
- structured hybrid verification rule requiring classical and post-quantum
  components to verify over the exact same transcript, with downgrade
  prevention and uniform failure behavior,
- crypto-suite lifecycle policy for signatures: active write suite, minimum
  accepted read suite, read-old/write-new migration, deprecation/sunset,
  mixed-suite manifest behavior, downgrade prevention, and release evidence for
  suite retirement,
- canonical fact encoding profile with domain-separation tag and format
  version,
- FactId allocation and binding decision: choose content-derived,
  engine-assigned, randomized, or hybrid identifiers before MVCC stores them,
  and document ordering leakage, uniqueness checks, replay behavior, and why
  FactId is not an authorization capability,
- signed transcript fields covering tenant, world revision, schema root,
  predicate/schema version, policy epoch, actor, valid time, causality links,
  evidence, payload digest, classification label, and all signature-covered
  fact metadata,
- fact record binding for tenant ID, crypto epoch, policy epoch, schema root,
  canonical transcript digest, and world revision so durable ingest cannot
  accept structurally valid but context-free facts,
- transaction-time reference validation for causal links, supersession,
  invalidation, evidence, and world membership, including same-tenant checks,
  committed-or-same-transaction existence, snapshot visibility, and rejection
  of future or cross-world references unless an explicit import proof exists,
- actor-to-key authorization model for assertion time,
- key lifecycle and revocation checks at assertion time,
- verified fact constructor shape that requires a key registry and trusted
  verification result; durable ingest cannot enable it until the concrete
  schema-root and world-revision contracts from `v0.20.4` and `v0.20.5` are
  present,
- signature algorithm/context rule that rejects algorithm confusion and
  signature-context misuse,
- tests for omitted fields, reordered fields, wrong tenant, wrong world
  revision, wrong schema root, stale policy epoch, revoked key, wrong actor key,
  and transcript version downgrade.

## v0.20.4 - Early Schema Root And Predicate Contract

Goal: move enough schema identity before durable commits so fact signatures,
backups, manifests, and world revisions do not bind to an undefined data
contract.

Deliverables:

- minimal schema root digest type and canonical schema-root transcript,
- predicate/model identity and version placeholders used by fact signing,
  world revisions, manifests, backups, and transaction validation,
- minimal authenticated security catalog containing predicate/schema security
  labels, PII and data-category annotations, required compartments,
  releasability metadata, policy compatibility class, schema root, predicate
  version, and catalog epoch before the full `v0.45.0` schema catalog exists,
- catalog-derived label-resolution contract for `v0.24.3` trusted planning, so
  planners have an authoritative source for labels and result metadata before
  query execution begins,
- compatibility stance for unknown schema roots before the full `v0.45.0`
  schema catalog exists,
- rule that durable facts cannot be committed without a schema/predicate root
  binding, even if the schema is a scaffolded opaque root,
- tests for missing schema root, mismatched predicate version,
  forward-incompatible schema marker, and backup/manifest/schema-root mismatch.

## v0.20.5 - Immutable WorldRevision And CAS Head Model

Goal: separate stable world identity from mutable world state before
transactions, manifests, and promotion rely on world heads.

Deliverables:

- `WorldRevisionId` immutable content/root identity distinct from `WorldId`,
- world revision record covering stable world ID, revision ID, previous
  revision, fork-base revision, added fact root, hidden fact root, schema root,
  policy epoch, and committing transaction ID,
- world-head compare-and-swap contract:
  `advance_head(world_id, expected_revision, next_revision)`,
- fork rule that child worlds record the exact parent revision used as fork
  base, not only the stable parent world ID,
- promotion rule requiring a three-way merge over fork base, current parent
  head, and candidate head,
- manifest world-head inventory that names world IDs and revision IDs,
- durable fact-membership proof model so a world revision records committed,
  tenant-scoped, snapshot-visible fact roots rather than accepting arbitrary
  caller-provided fact identifiers,
- large-overlay representation using persistent sorted runs, copy-on-write
  pages, or another bounded structure instead of cloned million-item vectors
  for production world heads,
- per-world and per-tenant aggregate limits for overlay size, mutation rate,
  pending promotion size, and branch/fork fanout,
- tests for divergent clones with the same `WorldId`, lost-update rejection,
  stale fork-base rejection, same-ID/different-state diff rejection, and CAS
  head advancement.

## v0.20.6 - Causal DAG And Truth Resolution Algebra

Goal: define fact visibility, precedence, and graph validity before indexes and
transactions make caused-by, supersedes, invalidates, hiding, deletion, and
policy revocation authoritative.

Deliverables:

- commit-time cycle validation for caused-by, supersedes, and invalidates
  graphs, or an explicit general-graph model with bounded visited sets and
  deterministic cycle behavior,
- deterministic state-resolution specification covering caused-by,
  supersedes, invalidates, world hiding, valid-time overlap, tombstones, legal
  deletion, policy revocation, and rollback/archive visibility,
- precedence rules for conflicting facts and overlapping valid-time intervals,
- bounded forward and reverse traversal rules for taint/blast-radius queries,
- tests for multi-fact cycles, invalidation chains, supersession precedence,
  hidden fact reintroduction, tombstone/legal-deletion conflict, and policy
  revocation visibility.

## v0.20.7 - Storage Kernel Expansion And Scrubbing

Goal: expand the `v0.19.1` storage spine with stronger compaction,
reclamation, scrubbing, and tiering behavior before transaction and query
milestones depend on sustained storage load.

Deliverables:

- hardened WAL-backed memtable, immutable sorted run, version-set, iterator,
  merge-iterator, and flush behavior from the `v0.19.1` storage spine,
- minimal background-work scheduler interface for flush, compaction, scrub,
  obsolete-file deletion, and checkpoint-adjacent cleanup before the full
  v0.40 worker-pool model exists,
- concurrency rules for flush, compaction, scrub, and deletion, including
  foreground read/write protection, audit and recovery priority, cancellation
  points, per-tenant fairness, starvation metrics, and bounded queue depth,
- minimal domain-local compaction implementation before durable transaction
  load tests, including tombstone and snapshot retention awareness,
- compaction picker with bounded debt accounting, tombstone retention windows,
  protected-root retention, and policy/encryption-domain compatibility checks,
- space-reclamation model covering WAL rotation and pruning, obsolete-file
  sets, snapshot and iterator pinning, rollback/archive/legal-hold roots,
  orphan-table recovery, and delayed deletion,
- file-number allocation bound to database ID, storage generation, and manifest
  generation,
- no-reuse rule for file numbers across active, obsolete, quarantined,
  migrated, or recovered files within one database generation,
- protected-root reference accounting for snapshots, rollback roots, backups,
  audit roots, and legal holds,
- online integrity scrub skeleton that verifies blocks, indexes, filters,
  manifests, and audit roots and quarantines suspect files instead of silently
  repairing them,
- hot/cold tiering decision for immutable tables, blobs, indexes, and
  projections while preserving tenant, policy, encryption-domain, snapshot,
  rollback, and legal-hold boundaries,
- content-addressed blob deduplication plan limited to equality-safe encrypted
  domains, with no cross-tenant, cross-compartment, cross-policy, or
  cross-key-epoch plaintext equality leakage,
- scrub salvage policy covering operator-driven quarantine review,
  backup-based repair, lost-range reporting, read-only degraded opening, and
  proof that repair cannot silently manufacture valid data,
- failure-injection acceptance using the `v0.18.13` harness for compaction
  input selection, output write, output publication, obsolete-file marking,
  orphan-table recovery, scrub quarantine, and degraded opening,
- tests for sorted-run lookup, merge iteration, flush/recover equivalence,
  compacted-state equivalence, protected-root liveness, file-number reuse
  rejection, background scheduler fairness, foreground-write protection,
  cancellation, starvation accounting, quarantine on scrub failure,
  cross-domain dedup rejection, and protected hot/cold tier movement.

## v0.20.8 - Block Cache, I/O Profile, And Capacity Governance

Goal: establish bounded I/O and cache behavior before realistic storage load
and query execution.

Deliverables:

- sharded block cache with separate budgets for data blocks, index/filter
  blocks, decoded metadata, and projection blocks,
- cache-aware scheduler hooks so background flush, compaction, scrub, and
  deletion work cannot evict protected foreground, audit, recovery, or anchor
  working sets without policy and budget approval,
- cache keys that include database generation, file number, block offset,
  encryption domain, policy epoch, and key epoch,
- per-tenant and per-security-domain cache accounting with pin counts for
  active iterators,
- reusable aligned-buffer pools, allocation limits, and copy-count budgets for
  WAL encoding, compression, AEAD, block reads, decompression, and secure
  zeroization paths,
- cache and buffer-pool revocation policy requiring decrypted entries,
  decoded metadata, projection cache entries, spill buffers, and reusable
  buffers to be evicted and scrubbed when a key is compromised, destroyed, or
  crypto-erased; when a policy epoch is revoked; or when a compartment/domain
  is sealed,
- stale-decryption denial rule proving a block cannot remain readable from
  cache after its key slot, policy epoch, or domain authority has been removed,
- rule that iterator and snapshot pins prevent physical deletion of referenced
  files but must not force all pinned blocks to remain resident in cache,
- explicit rule that decrypted blocks cannot be cached across incompatible
  authority contexts or policy epochs,
- deployment profile decision between page-cache-heavy operation and larger
  userspace block cache to avoid uncontrolled double caching,
- portable buffered positional/vectored I/O baseline, with optional direct-I/O,
  io_uring, mmap, SIMD, and aligned-buffer paths isolated behind reviewed host
  adapters,
- unsafe or platform-specific fast paths must live in dedicated host-boundary
  crates, retain the portable fallback, be optional per deployment profile, and
  pass Miri where applicable plus ASan/TSan or equivalent checks before release
  gates accept them,
- descriptor-relative host path plan that keeps trusted directory handles and
  treats Linux `openat2` constraints as an optional hardened implementation,
- filesystem capacity, ENOSPC, quota, file-count, open-file, and temporary-disk
  governance before writes reserve resources,
- WAL/table extent preallocation policy, table size classes, temporary-space
  reservation, minimum free-space margins, compaction-debt admission, and
  per-tenant I/O throttling before commits can exhaust shared storage,
- disk-exhaustion escape reserve for WAL repair, manifest/checkpoint
  publication, audit emission, and orderly shutdown, with write-stall and
  recovery behavior when compaction cannot reserve enough temporary space,
- cache-hit, cache-residency, copy-count, and allocation SLOs by block class
  and security domain, measured only after the cache and aligned-buffer pools
  exist,
- tests for cache budget enforcement, iterator pinning, cross-authority cache
  rejection, ENOSPC/quota handling, file-count limits, descriptor-relative
  path replacement attempts, aligned-buffer exhaustion, copy-count budget
  enforcement, and secure zeroization on buffer reuse.

## v0.20.9 - Policy Token Catalog And Compact Hot-Path Tags

Goal: reduce authorization hot-path cost without reintroducing token leakage or
unbounded string comparison.

Deliverables:

- catalog-assigned compact policy token IDs or keyed fixed-width token tags for
  compartments, releasability, sovereignty, and policy compatibility classes,
- canonical string validation remains at ingestion/catalog boundaries, while
  policy hot paths use compact fixed-width representations,
- constant-shape comparison and set membership over compact tags with timing
  evidence tied to `v0.20.1`,
- migration path from scaffold `PolicyTokenSet` slots to catalog-backed tags,
- tests that homograph rejection, redaction, overflow behavior, non-allow
  masking, and constant-shape scans remain intact after compacting hot paths.

## v0.20.10 - Verified Fact Ingest Integration Gate

Goal: turn the signature transcript, schema-root contract, world-revision
model, and transaction reference checks into the first non-forgeable verified
fact ingest path before transaction milestones accept durable writes.

Deliverables:

- engine-owned verified ingest API that accepts only canonical fact
  transcripts, trusted signature verification results, catalog-derived schema
  roots, exact world revision context, policy epoch, crypto epoch, actor
  authority, and transaction intent from the `v0.18.14` dependency order,
- durable ingest rule that actor signatures bind the pre-state/base world
  revision and fact payload digests, while engine commit receipts bind the
  resulting `WorldRevisionId` after WAL commit and manifest publication,
- transaction-time validation for same-tenant fact references, existence,
  snapshot visibility, causal/supersession/invalidation edges, evidence
  references, schema compatibility, key lifecycle state, and actor authority,
- rejection of caller-supplied fact IDs or context fields that conflict with
  the selected `FactId` allocation strategy, catalog state, world revision, or
  transaction intent,
- idempotent retry behavior for verified ingest after ambiguous durability
  outcomes using the durable LSN receipt and commit-status scaffold,
- tests for forged verification results, wrong schema root, wrong world base
  revision, stale catalog epoch, revoked signer, future fact reference,
  cross-tenant edge, replayed commit intent, duplicate idempotency key, and
  circular transcript dependency.

## v0.21.0 - In-Memory Transaction Model

Goal: model read sets, write sets, predicate sets, commit timestamps, and fact
identity allocation semantics.

Deliverables:

- transaction state type,
- read/write/predicate set tracking,
- explicit transaction resource limits for read-set entries and bytes,
  write-set entries and bytes, predicate-set entries, causal-link expansion,
  savepoints if supported, transaction-local payload bytes, and total
  transaction memory,
- transaction lifetime and snapshot-pin age limits with fail-closed behavior
  for abandoned clients and long-running transactions that would block
  compaction, tombstone reclamation, rollback cleanup, or crypto-erasure,
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
- cross-tenant and cross-domain transaction policy, including whether such
  transactions are denied, split into separately authorized subtransactions,
  or admitted only through an explicit multi-domain commit protocol,
- idempotency and commit-status record retention, garbage collection, and
  privacy policy so retry safety does not create an unbounded history oracle,
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
- OCC fairness policy for hot contention, including retry budgets,
  starvation metrics, bounded backoff, and the criteria for switching to an
  adaptive pessimistic intent-lock path,
- abort reasons,
- deterministic concurrency tests for starvation, retry-budget exhaustion, hot
  write contention, bounded backoff, and adaptive intent-lock admission.

## v0.22.1 - Concurrency Model Checking

Goal: validate the single-node concurrency protocol before durable commit,
checkpoint, compaction, and world-head publication rely on it.

Deliverables:

- Loom, Shuttle, or equivalent model tests for commit coordinator ordering,
  world-head compare-and-swap, version-set publication, checkpoint/WAL rotation,
  cache pinning, memtable flush, compaction publication, key rotation versus
  segment flush, and cleanup versus active segment publication,
- global latch ordering document for short-lived latches over world heads,
  version sets, index shards, cache shards, file deletion, checkpoint, and WAL
  rotation,
- linearizability or serializability history checker for generated concurrent
  transaction histories,
- tests for lock-order inversion, publication race, stale snapshot deletion,
  compaction liveness race, group-commit race, and key-rotation race.

## v0.22.2 - Group Commit And Durability SLO

Goal: make WAL durability efficient and explicit before durable transactions
become the normal write path.

Deliverables:

- group-commit coordinator that batches transaction commit records behind one
  durable barrier without exposing unflushed commits as visible,
- configurable sync modes with explicit guarantees and non-claims,
- durable LSN receipts and commit-status lookup integrated with transaction
  idempotency keys from `v0.18.12`,
- commit sequence: policy validation, immutable write batch, sequence
  reservation, conflict validation, WAL append, group fsync, version/world-head
  publication, acknowledgement,
- mandatory-audit atomicity design: either the WAL commit transcript contains
  and authenticates the audit-record digest made durable by the same barrier,
  or a transactional audit outbox is committed in the same WAL transaction and
  deterministically drained after recovery,
- ordering rules for commit-sequence allocation, validation, group durability,
  publication, cancellation, timeout, and client disconnect so an ambiguous
  client session cannot create duplicate or lost commits,
- crash rule proving a crash after the durable barrier but before publication
  is recovered by redo, while a crash before the barrier is never visible as
  committed,
- p50/p99 commit-latency evidence and throughput evidence for single
  transaction, batched transaction, and fsync-heavy profiles,
- SLO evidence for WAL bytes per commit, fsyncs per commit, and ambiguous
  commit-status resolution latency,
- failure-injection acceptance using the `v0.18.13` harness for group fsync,
  batch publication, audit digest/outbox durability, timeout, cancellation, and
  client disconnect,
- tests for batch barrier ordering, failed fsync, partial batch commit, replay
  after barrier-before-publication crash, audit-before-publication crash,
  failed audit storage, duplicate recovery emission, ambiguous client
  acknowledgement, and sync-mode misconfiguration.

## v0.23.0 - Durable Transaction Commit

Goal: connect transaction validation to WAL and recovery.

Deliverables:

- prepare and commit records,
- durable commit boundary,
- audit event emission through the atomic audit transcript or transactional
  audit outbox model selected in `v0.22.2`,
- replay of committed transactions,
- rollback of uncommitted transactions,
- failure-injection acceptance using the `v0.18.13` harness for prepare write,
  commit write, audit write/outbox write, replay, rollback, and publication,
- crash tests around prepare/commit, commit-before-audit rejection,
  audit-before-publication recovery, duplicate recovery emission prevention,
  failed mandatory-audit storage, and ambiguous client acknowledgement.

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
- transaction SLO evidence for read-your-writes memory, predicate-set memory,
  abort/retry rate under contention, hot-key backoff behavior, durable commit
  throughput, and client-disconnect/ambiguous-commit status lookup,
- benchmark methodology filled out with hardware, filesystem, mount options,
  drive-cache mode, dataset distribution, compression ratio, encryption
  profile, warm/cold cache state, concurrency, run variance, and rootless
  container bind-volume versus overlay-filesystem behavior,
- early evidence for whether fixed-shape policy evaluation, storage footer
  overhead, and sequential recovery are acceptable for 1.0 targets,
- documented adjustment decision before v0.24 snapshot reads and v0.25 query
  AST depend on the current storage/transaction shape.

## v0.23.3 - Durable Primary And Causal Indexes

Goal: provide durable indexes before snapshot reads and query execution depend
on in-memory-only fact lookup.

Deliverables:

- durable primary fact index keyed by tenant, world revision or snapshot,
  fact identity, commit sequence, and visibility state,
- durable reverse indexes for caused-by, supersedes, invalidates, hidden facts,
  tombstones, and snapshot visibility,
- sparse and block-level index use from the `v0.18.11` physical layout,
- iterator semantics over point lookups, temporal scans, causal traversal,
  branch overlays, and snapshot visibility,
- index rebuild and verification path from canonical fact blocks,
- tests for point lookup, temporal range scan, reverse causal traversal,
  supersession/invalidation lookup, branch overlay lookup, index corruption,
  rebuild equivalence, and stale-index rejection.

## v0.23.4 - Storage Reference Models And Property Tests

Goal: keep the storage and transaction engine checked against simpler models
before query execution and compaction grow complex.

Deliverables:

- deterministic reference-model differential tests for facts, worlds,
  visibility, transactions, indexes, and compaction,
- property tests for serialization, replay idempotence, snapshot visibility,
  compaction equivalence, manifest selection, and rollback protected-root
  liveness,
- golden compatibility corpus for every durable format version introduced so
  far,
- randomized state-machine tests comparing recovered state with an in-memory
  oracle over append, flush, compact, checkpoint, crash, recover, and query
  lookup operations.

## v0.24.0 - Fact Index And Snapshot Reads

Goal: read facts by world and snapshot from recovered state.

Deliverables:

- durable fact indexes from `v0.23.3` integrated into snapshot reads,
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
- authenticated authority-context minting rule: subject, device, workload,
  tenant, purpose, clearance, compartments, attestation, credential epoch, and
  revocation state come from the API/session boundary and policy registry, not
  from caller-constructed public structs,
- tests that forged device/workload context cannot be supplied directly to
  policy evaluation through the API boundary,
- constant-shape public error response fixture for unauthenticated,
  unauthorized, redacted, and malformed requests,
- integration test that runs legal/security/sovereignty/minimisation planning
  stubs through the same request context shape the later query planner will
  use,
- documented decision on whether the planned seven-stage planner pipeline needs
  restructuring before v0.25 and v0.27.

## v0.24.3 - Trusted Planner Context And Canonical Label Resolution

Goal: prevent callers from declaring the security labels or result metadata of
stored data before public query construction and policy planning harden.

Deliverables:

- trusted plan-context contract binding tenant, principal, device evidence,
  workload evidence, world revision, manifest root, policy epoch, query digest,
  purpose, expiry, and replay nonce,
- rule that public query requests carry query intent/AST only; stored fact
  labels, schema labels, result-classification inputs, and release constraints
  are resolved from validated storage, schema metadata, and policy registries,
- `BoundQueryPlan` shape whose policy proof is bound to authenticated context,
  snapshot, policy epoch, query digest, purpose, expiry, and nonce,
- non-forgeable plan outcome shape:
  `Executable(AuthorizedPlan)`, `Redacted(RedactionPlan)`, or `Denied`, where
  only `AuthorizedPlan` implements the future executor input trait and denied
  plans cannot become executable by ignoring an advisory flag,
- execution-time revalidation that snapshot root, policy epoch, identity,
  attestation evidence, and proof binding are still current before execution,
- denial behavior for stale proof, wrong tenant, wrong principal, wrong world
  revision, wrong manifest, wrong policy epoch, wrong query digest, wrong
  purpose, expired context, and replayed nonce,
- tests that untrusted callers cannot request a Public label for TopSecret
  facts, cannot supply fake result-classification inputs, cannot reuse proofs
  across queries, and cannot execute against a different snapshot than planned.

## v0.24.4 - Full Result Security Label Join Rules

Goal: make derived result labels carry all mandatory access-control and
dissemination constraints before query planning, projections, exports, caches,
or AI pipelines consume result metadata.

Deliverables:

- `ResultSecurityLabel` or equivalent result label model with classification,
  required compartments, dissemination/releasability constraints, sovereignty
  scope, policy epoch proof set, PII state, AI eligibility, and confidence
  policy metadata,
- formal join rules: classification uses the dominating maximum, required
  compartments use union, releasability uses intersection or deny-all on
  conflict, sovereignty uses exact-or-saturated scope, policy epochs use a
  bounded proof set or approval-required/deny state, and overflow uses typed
  most-restrictive states instead of truncation,
- downstream contract for caches, projections, export, backup, legal planning,
  AI processing, and declassification so they cannot treat a derived result as
  less constrained than its inputs,
- tests for combining same-classification different-compartment facts, mixed
  releasability sets, deny-all dissemination, policy-epoch conflicts,
  compartment overflow, saturated sovereignty, and non-allow metadata masking.

## v0.24.5 - Query Inference Budget And Aggregation Controls

Goal: define an effective response to statistical inference and differencing
attacks before aggregate query execution exists.

Deliverables:

- inference-budget model keyed by principal, tenant, purpose, dataset digest,
  privacy budget, and time window,
- minimum cohort-size, contribution-bound, rate-limit, result-budget,
  consistent-suppression, and query-history-aware differencing controls,
- cross-session, cross-device, and cross-service budget aggregation rules,
- differential-privacy decision for cases where exact aggregates are not
  required,
- purpose-specific audit for inference-sensitive queries,
- planner behavior that denies, redacts, suppresses, or requires approval when
  inference budget or cohort requirements fail,
- tests for differencing attempts, repeated small cohorts, budget exhaustion,
  cross-device budget reuse, inconsistent suppression, and purpose mismatch.

## v0.24.6 - Access-Pattern And Query-Shape Leakage Model

Goal: explicitly classify and mitigate what encrypted storage and redacted
results still leak through access patterns, response sizes, timing, and query
shape before query execution and projections make those surfaces real.

Deliverables:

- leakage classification for touched worlds, segments, indexes, projections,
  result counts, response sizes, execution time, cache hits, and repeated query
  shapes,
- decision record on which profiles accept access-pattern leakage and which
  require padding, batching, fixed-size pages, private-query modes, delayed
  responses, cover traffic, or offline/export-only workflows,
- planner metadata that records the leakage profile used for a query plan,
- tests that high-assurance profiles reject query shapes that would reveal
  protected compartment existence, small cohort presence, or sensitive index
  membership through access patterns,
- explicit non-claim if ORAM-style protection is not implemented before 1.0.

## v0.25.0 - Native Query AST

Goal: define the first native query representation without execution.

Deliverables:

- world read AST,
- fact filter AST,
- causality explain AST,
- simulation query AST skeleton using the `v0.24.1` isolation model,
- logical node metadata slots for monotone security transfer functions,
  schema-derived labels, PII state, sovereignty, releasability,
  inference-budget class, and leakage profile so scans, filters, joins,
  aggregates, expressions, caches, and projections cannot be planned without
  policy propagation hooks,
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
- require a witnessed, one-time, scope-limited grant with nonce consumption,
  approval proof, revocation state, off-node alert hook, and post-event review
  workflow before any break-glass access can release protected data,
- require the grant and the protected operation to pass through the
  transactionally coupled audit barrier from `v0.18.8`, so protected access
  fails closed if the audit evidence cannot become durable,
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
- no caller-supplied labels or result inputs in public planning paths; labels
  and result security metadata come from validated snapshot/schema/policy
  state using the `v0.24.3` trusted planner context,
- bound policy proof tied to tenant, principal, attestation evidence, world
  revision, manifest root, policy epoch, query digest, purpose, expiry, and
  nonce,
- break-glass planning uses the `v0.26.1` access model and does not treat audit
  metadata as a blanket policy bypass,
- rejection and redaction reports,
- policy proof skeleton,
- full query-result security label propagation from `v0.24.4`, including
  compartments, releasability/dissemination, policy epochs, sovereignty, PII,
  AI eligibility, and confidence hooks,
- sovereignty-scope overflow handling: exact bounded jurisdictions stay exact,
  while more than the bounded exact set becomes a typed multi-jurisdiction
  sentinel that is treated as most-restrictive for export, placement, indexing,
  backup, AI processing, and legal/compliance decisions,
- inference-budget planning from `v0.24.5` for aggregate and repeated-query
  surfaces,
- access-pattern and query-shape leakage handling from `v0.24.6`,
- confidence-aware allow/redact/reject policy hooks,
- tests for denied plans.

## v0.27.1 - Cost, Leakage, And Vectorized Execution Plan

Goal: choose the execution architecture before the query prototype hardens
around row-at-a-time scans.

Deliverables:

- batch-oriented operator model with selection vectors, columnar intermediate
  batches where useful, bounded arenas, and bounded spilling,
- cost model that includes storage I/O, CPU, memory, policy evaluation,
  leakage profile, projection freshness, and legal/compliance checks,
- policy-partitioned statistics rule: optimizer statistics, histograms,
  samples, cardinality estimates, and cost feedback are partitioned or
  redacted by authority, tenant, policy epoch, classification, compartment,
  purpose, and world revision so optimization cannot inspect unauthorized
  metadata,
- rewrite correctness property tests proving predicate pushdown, projection
  pruning, join reorder, aggregate rewrite, index-only scan, projection use,
  and plan-cache lookup preserve or tighten security annotations,
- plan-cache isolation rule forbidding reuse across authority, tenant,
  device/workload evidence, policy epoch, world revision, purpose, leakage
  profile, or query digest boundaries,
- explicit interpreter-first rule; optional JIT is deferred until profiling
  proves CPU-bound expression evaluation dominates and an interpreter fallback,
  code-cache limit, and sandbox/admission review exist,
- execution plan compatibility with the storage-format compression decisions
  from `v0.18.11` and WAL sub-batch compression decisions from `v0.18.12`,
- benchmark fixtures for YCSB-like point workloads, temporal scans, causal
  traversal, branch promotion, checkpoint recovery, compaction debt, and
  policy-heavy queries,
- tests for bounded batch memory, spill limits, policy/leakage cost influence,
  compression-bomb rejection, and interpreter/JIT deferral guards.

## v0.28.0 - Query Execution Prototype

Goal: execute read-only fact and causality queries on a single node.

Deliverables:

- fact scan execution,
- point lookup execution,
- causality edge traversal over fact links,
- batch-oriented execution using the `v0.27.1` vectorized plan where it
  improves bounded scans and joins,
- bounded forward traversal for taint and blast-radius queries,
- implementation of the selected `v0.23.1` propagated-confidence model over
  evidence and caused-by chains,
- execution-time revalidation that the bound plan's snapshot root, policy
  epoch, proof, identity, expiry, purpose, and nonce are still valid,
- periodic authority, policy, revocation, expiry, and snapshot-lease
  revalidation for long-running or streaming queries,
- final output gate before every result batch so mid-query revocation,
  authority expiry, policy change, or domain sealing cannot leak a partially
  unauthorized stream,
- encrypted, domain-bound query spill files with secure cleanup, quota
  accounting, and rejection of spill across incompatible policy/key domains,
- inference-budget enforcement for aggregate query execution,
- execution behavior that follows the selected `v0.24.6` leakage profile,
- bounded result sets,
- tests for authorized and denied reads.

## v0.29.0 - Projection Registry

Goal: register rebuildable projections without implementing every projection type.

Deliverables:

- projection metadata,
- source fact range,
- consistency mode,
- watermark tracking,
- transactional projection outbox or journal rooted in the WAL commit and
  manifest generation, with at-least-once delivery, idempotent application,
  replay after crash, and deterministic duplicate suppression,
- monotonic per-partition watermarks, lag limits, backpressure behavior,
  rebuild fallback, and stale-generation cleanup,
- policy/key revocation handling for queued but not-yet-applied projection
  work, including quarantine or cancellation before stale work can publish,
- crash-safe projection publication protocol that writes projection output,
  verifies compatibility key, publishes the watermark, and cleans stale
  artifacts without sharing the canonical WAL,
- incremental materialized projection model with source watermarks, manifest
  generation binding, rebuild range, stale marker, and policy/legal epoch
  compatibility,
- complete projection compatibility key covering tenant, source world
  revision, classification, compartments, releasability/dissemination,
  policy/law-pack epochs, schema root, crypto epoch, projection type/version,
  model version where relevant, and encryption domain,
- projection isolation rule requiring separate files, keys, caches, quotas,
  temporary directories, rebuild watermarks, and manifest references for
  incompatible compatibility keys,
- projection worker rule that consumes only authenticated committed-generation
  events, never writes canonical facts, never shares the canonical WAL, and
  quarantines stale partitions when policy, model, key, or source revisions
  change,
- encryption domain tracking,
- rebuild command skeleton.

## v0.30.0 - Graph Projection

Goal: build the first projection from causal facts.

Deliverables:

- adjacency projection,
- source range tracking,
- rebuild from canonical facts,
- versioned `TaintEvent` and `InvalidationEpoch` records for compromised
  sources, workers, models, keys, imports, decisions, releases, exports,
  backups, projections, and AI artifacts,
- explicit blast-radius states such as suspect, tainted, quarantined,
  superseded, revalidated, and retained-for-audit,
- checkpointed and resumable traversal jobs with hard node, edge, depth,
  memory, time, and output budgets plus partial-result semantics,
- unified dependency-edge model from facts to decisions, projections, releases,
  exports, backups, AI artifacts, declassification proofs, and cache entries,
- no in-place untaint rule: revalidation creates a new signed derivation,
  revision, projection, or release record rather than silently clearing old
  taint state,
- confidence memoization keying by world revision, fact ID, and propagation
  policy digest rather than duplicating full path state in canonical facts,
- stale projection detection,
- tainted projection detection from causal blast-radius traversal,
- encrypted projection metadata,
- graph traversal tests.

## v0.31.0 - Search Projection Skeleton

Goal: create a policy-aware full-text/search projection boundary.

Deliverables:

- tokenizer boundary,
- source fact visibility checks,
- policy-partitioned full-text index plan that partitions or rejects index
  mixing across incompatible tenant, compartment, classification,
  releasability, policy epoch, legal basis, and encryption-domain boundaries,
- projection watermark,
- rebuild tests,
- no cross-compartment mixing tests.

## v0.31.1 - Columnar Projection Skeleton

Goal: define policy-partitioned columnar projection storage before vectorized
execution and analytical reads depend on columnar layouts.

Deliverables:

- columnar projection metadata for source world revision, schema root,
  projection version, policy/legal epoch, encryption domain, key epoch, and
  manifest generation,
- column chunk format decision with encrypted chunk bodies, authenticated
  column statistics, row-group visibility metadata, deletion/tombstone masks,
  and rebuild watermarks,
- policy-partitioned statistics rule so min/max, bloom filters, dictionaries,
  histograms, and null counts cannot reveal unauthorized values across
  tenant, classification, compartment, releasability, legal, or key domains,
- vectorized execution compatibility for bounded batches, selection vectors,
  encrypted/domain-bound spill, and final output security gates,
- rebuild and invalidation protocol from the projection journal in `v0.29.0`,
- tests for cross-domain column merge rejection, stale policy/key epoch,
  unauthorized statistics access, tombstone visibility, rebuild after crash,
  and vectorized scan result-label preservation.

## v0.31.2 - Optional Extension Crate Boundary

Goal: keep application-family support out of the mandatory database core unless
the feature is truly generic.

Deliverables:

- extension-crate policy for product-family primitives, including naming,
  dependency direction, default-build behavior, release gates, and pentest
  requirements,
- `skrifheim-extension-api` boundary decision containing only stable generic
  capabilities for fact proposals, authorized queries, namespaced schemas,
  projection declarations, migrations, and audit submission,
- compiled-in explicit extension registration model; no implicit global
  constructors, runtime discovery with ambient authority, or extension access
  to raw WAL, segment, key-provider, filesystem, or unrestricted database
  handles,
- extension manifest schema covering API version, schema namespace,
  capabilities, projection domains, resource budgets, migration
  compatibility, provenance, SBOM, and pentest/fuzz evidence,
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
- independent extension-versioning rule: each optional extension has its own
  semver, compatibility fixtures, denial/redaction/quarantine tests, resource
  budgets, and release/pentest gate while the core database remains buildable
  without it,
- documentation review that rewrites app-specific schema names into generic
  relationship, object, workflow, projection, and policy-extension primitives
  before implementation starts.

## v0.32.0 - Vector And AI Projection Encryption Boundary

Goal: make vector and AI-derived projections safe before AI artifacts become useful.

Deliverables:

- vector projection encryption domain,
- AI artifact encryption domain,
- policy-partitioned vector projection plan, including explicit decision on
  mutable working-set indexes versus immutable disk ANN projections and
  rejection of cross-policy embedding/index mixing,
- vector-search design as a rebuildable projection: mutable HNSW or flat-search
  delta for fresh writes, immutable DiskANN/IVF-PQ-style snapshots for scale,
  manifest-bound watermarks and snapshot visibility, policy-first filtering
  where possible, embedding model/version/provenance fields, and recall
  regression fixtures,
- rule that HNSW/ANN graph edges never cross incompatible tenant, compartment,
  classification, policy, key-epoch, legal, or embedding-model domains,
- bounded oversampling for filtered ANN, exact-search fallback when filtering
  would make approximate search unsafe, deterministic rebuild criteria, and
  recall targets per policy partition,
- AI write capability ceiling metadata,
- derivation-cone key-domain metadata,
- source-fact visibility rules,
- no lower-domain embedding of higher-domain facts,
- tests for denied vector/AI projection writes, vector deletion visibility,
  stale-policy rejection, recall regression, cross-domain leakage rejection,
  filtered-ANN oversampling bounds, exact-search fallback, and
  model-version/provenance mismatch.

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
- quorum proof transcript binding guardian identity, guardian role, tenant,
  operation type, authority scope, policy epoch, crypto epoch, transcript
  digest, threshold rule, signer set, validity window, and anti-replay nonce,
- hybrid classical/post-quantum signature rule requiring all structured
  components to verify over the identical transcript with no downgrade to a
  single surviving component unless a later policy explicitly admits that
  profile,
- validation API that rejects insufficient quorum, duplicated participants,
  stale policy or crypto epoch, revoked participants, wrong tenant, wrong
  authority scope, and mixed incompatible algorithms,
- tests for threshold/quorum replay rejection, signer substitution, duplicate
  share/signature attempts, and downgrade to single-signer authority,
- documentation that the selected model is a pre-1.0 production requirement,
  while advanced distributed key generation may remain post-1.0 if explicitly
  deferred.

## v0.34.0 - Audit Storage, External Receipts, And Proof Queries

Goal: implement the durable audit storage and verification surface needed to
prove what was known, under which policy, and from which manifest.

Deliverables:

- encrypted audit-segment writer and reader using the chained audit-log root
  contract from `v0.18.8`,
- chain continuity verification for stream generation, sequence,
  previous-record digest, event digest, policy epoch, manifest root, WAL
  transaction root, and audit segment root,
- Merkle inclusion and consistency proof format for audit records, audit
  segments, audit roots, and manifest-bound audit inventories,
- external receipt and witness persistence model with verification against
  freshness anchors, WORM/offline checkpoints, or remote transparency
  services where configured,
- audit-key rotation and verification across old and new audit signing or
  encryption keys without breaking proof continuity,
- recovery of transactional audit outbox records without duplicate emission
  and without reporting protected operation success before mandatory audit is
  durable,
- durable pre-release audit path for protected reads and other mandatory-audit
  operations that do not otherwise mutate canonical facts,
- audit segment retention and compaction policy preserving proof continuity,
  external receipts, legal holds, and mandatory retention windows,
- offline verifier CLI/API that can verify audit segments, manifest roots,
  receipt chains, and selected proof queries without trusting the live database
  process,
- fact existence proof skeleton,
- policy epoch proof skeleton,
- confidence derivation proof skeleton,
- manifest root reference,
- audit query output type,
- tests for missing proof material, removed audit record, reordered record,
  duplicate outbox replay, stale receipt, key-rotation boundary, protected read
  without durable audit, compacted audit segment proof, and offline verifier
  mismatch.

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

## v0.35.3 - Point-In-Time Recovery And Change Streams

Goal: make committed manifest generations usable for recovery, downstream
sync, and incident reconstruction without bypassing policy or freshness rules.

Deliverables:

- point-in-time recovery model selecting a manifest generation, world revision,
  policy epoch, schema root, audit root, key-state digest, and freshness-anchor
  generation,
- PITR restore modes aligned with `v0.35.2`: read-only inspection,
  recovery-world fork, simulation replay, and policy-checked promotion back to
  production,
- change-stream records tied to committed manifest generations, transaction
  commit roots, world revisions, policy epoch, schema root, and redacted
  operation categories,
- subscriber/cursor model with tenant, purpose, authority, policy epoch,
  leakage profile, replay window, and revocation behavior,
- deterministic replay tracing for production incident reconstruction,
  covering manifest generation, WAL commit roots, applied batches, projection
  rebuilds, compaction events, policy decisions, and redacted error outcomes,
- tests for stale cursor rejection, unauthorized stream reads, replay across
  policy epoch changes, missing manifest generation, rollback/PITR confusion,
  deterministic trace reproduction, and redacted trace output.

## v0.35.4 - Backup Engine Completion

Goal: complete the backup/restore engine and evidence model without claiming
final production qualification before later schema, retention, quota,
observability, runtime-scheduling, and legal-policy integrations exist.

Deliverables:

- online consistent backup protocol during concurrent writes, binding the
  selected manifest generation, world revisions, schema root, policy epoch,
  key-state digest, audit root, freshness-anchor generation, and protected-root
  pins for the backup duration,
- full and incremental backup formats with explicit base generation, delta
  chain, WAL/archive requirements, exact recovery-point semantics, and
  compatibility with the schema/catalog and durable-format feature policies,
- bounded incremental-backup chain length, periodic synthetic-full
  consolidation, restore-amplification budget, and crash-safe deletion of
  superseded chains,
- resumable upload/download model with independently verified chunks, chunk
  manifests, retry idempotency, orphaned-upload cleanup, and failure-safe
  temporary storage,
- backup encryption domain, backup key rotation, key-slot deletion, wrapped-key
  index, crypto-erasure behavior, lost-key behavior, and retention/legal-hold
  interaction,
- retention, deletion, expiration, quarantine, and orphaned-backup cleanup
  policy that cannot delete audit, rollback, legal-hold, or active restore
  material without explicit authority,
- restore modes for new database identity, read-only historical inspection,
  recovery-world fork, simulation, and authorized in-place recovery with
  freshness-anchor and manifest-generation checks,
- regular automated restore-drill procedure with corruption injection,
  missing-chunk tests, wrong-key tests, stale-policy tests, incompatible-schema
  tests, and operator evidence recording,
- measured RPO, RTO, restore throughput, backup throughput, temporary-space
  requirement, chunk verification cost, and concurrent-write impact,
- tests for online backup consistency, incremental-chain validation,
  chain-length overflow, synthetic-full consolidation, restore-amplification
  limits, crash-safe superseded-chain deletion, resumable upload idempotence,
  orphan cleanup, wrong database identity, unauthorized in-place restore,
  backup key rotation, crypto-erasure, and corrupted backup rejection.

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

## v0.38.4 - Opaque Extension Capability API Preview

Goal: freeze only the smallest opaque capability surface that optional
post-1.0 extension crates will compile against, while leaving schema,
migration, legal/compliance, operation-passport, and placement interfaces
unstable until their later pre-1.0 milestones complete.

Deliverables:

- public API review for opaque capabilities over facts, worlds, labels, policy
  proofs, authorized query handles, projection declarations, audit submission,
  and storage-root references that are already stable enough to expose,
- explicit non-freeze list for schema/catalog migration, legal/compliance
  passports, retention/deletion workflows, placement APIs, operation passports,
  backup/restore contracts, and product-family extension schemas until the
  final pre-1.0 extension compatibility review,
- compile-only extension fixture that depends on the core APIs without being a
  default workspace dependency,
- documentation of which opaque APIs are stable for early v1.1.0 through
  v1.6.x extension planning and which APIs remain intentionally unstable,
- extension lifecycle semantics covering removal without making canonical
  facts or schema descriptors unreadable, atomic and resumable extension
  upgrade/migration, capability revocation for existing sessions and
  background jobs, panic/resource-exhaustion containment, and cross-extension
  dependency denial or explicit versioning,
- trust-boundary statement that compiled-in extensions are trusted code and
  part of the TCB unless isolated through a future process or WASM boundary,
- release-gate check proving optional extension fixtures can be omitted from
  the production core build,
- explicit deferral notes that collaborative text, publishing releases, render
  graphs, forum/discussion, messenger, feed/media, mailbox, collaboration, and
  forge/source-state behavior starts after v1.0.0.

## v0.39.0 - Storage Format Migration And Downgrade Protection

Goal: make database upgrades, storage-format migrations, and downgrade
attempts explicit before production hardening.

Deliverables:

- storage-format feature manifest for WAL, segment, manifest, audit, backup,
  schema, and projection format versions,
- supported storage-upgrade window defining directly supported source
  versions, versions requiring sequential migration, read-only recovery support
  for older formats, support lifetime for each durable format, and whether any
  pre-1.0 format receives compatibility guarantees,
- golden database corpus retained for every supported upgrade path, including
  WAL, table/segment, manifest, audit, backup, schema, projection, and
  freshness-anchor state,
- cross-platform migration expectations for every production-supported
  OS/filesystem pair and explicit non-claims for preview-only targets,
- migration plan model with preflight, dry-run, required source version,
  target version, affected roots, expected duration class, rollback/restore
  point, and operator authority,
- downgrade policy that rejects opening newer storage with older binaries
  unless an explicit read-only recovery profile supports it,
- idempotent migration checkpointing so interrupted migrations resume or fail
  closed without partial trust,
- signed migration proof facts and audit events bound to manifest root,
  freshness anchor, schema root, policy epoch, crypto epoch, and operator
  approval,
- tests for direct upgrade, sequential migration, read-only old-format
  recovery, unsupported old-format rejection, interrupted migration, replayed
  migration proof, downgrade attack, unknown feature bit, partial projection
  migration, cross-platform migration, and rollback-protected snapshot
  preservation.

## v0.40.0 - Host Runtime Isolation And Resource Pools

Goal: prevent optional host/runtime behavior from becoming an implicit shared
privilege or noisy-neighbor side channel before API and projection execution.

Deliverables:

- tenant and classification-aware worker-pool model for queries, projection
  rebuilds, compaction, backup, export, legal planning, and AI artifact jobs,
- admission control for CPU, memory, file handles, temporary disk, queue depth,
  and concurrent jobs before work starts,
- fail-closed policy for high-classification or high-assurance work that cannot
  run in a sufficiently isolated pool,
- tests that one tenant cannot starve another tenant's mandatory audit,
  recovery, or freshness-anchor work,
- explicit non-claim for microarchitectural isolation unless a deployment uses
  process, VM, hardware, or OS-level isolation profiles.

## v0.41.0 - AI Artifact Provenance

Goal: store AI output as untrusted derived artifacts with provenance.

Deliverables:

- source fact lineage,
- model and prompt hash metadata,
- prompt/context/tool transcript digest, model identity, model version,
  worker identity, toolchain version, reviewer identity where present, and
  derivation-cone key-domain metadata,
- capability-scoped AI write metadata,
- typestate boundary separating authoritative facts from unreviewed AI
  artifacts, with private or sealed constructors so extension crates cannot
  mint authoritative writes or AI work permits directly,
- policy-engine minted `AiWorkPermit` or equivalent bound to tenant, exact
  world revision, source root, policy epoch, purpose, model/worker identity,
  classification ceiling, expiry, nonce, and derivation-cone ID,
- executor API shape that accepts only authorized AI inputs plus an AI work
  permit, and always returns unreviewed artifacts until a separate reviewed or
  quorum-approved promotion creates a signed fact,
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
- declassification verifier requiring exact source roots, source and target
  labels, authorized signer roles or quorum proof, policy epoch, legal basis,
  purpose, expiry, nonce, audit binding, and proof revocation/supersession
  state before any write-down can proceed,
- mandatory write-down gate proving no export, cache, projection, AI artifact,
  mission capsule, public release, backup-derived artifact, or new fact write
  can lower a label without a verified declassification proof,
- import verification preflight,
- rejected downgrade tests, missing proof tests, stale proof tests, wrong
  target-label tests, revoked signer tests, and cache/projection/export
  bypass tests.

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
- sustained coverage-guided fuzz campaigns for WAL, table/block parsing,
  manifests, backups, query parsing/planning, and storage migrations,
- retained minimized regression corpora committed or archived as release
  evidence for every production-relevant parser and durable format,
- durable fuzz-evidence archive contract covering content digests,
  source-commit binding, authenticated producer identity, trusted timestamp,
  retention period, storage authority, reproducible retrieval instructions,
  toolchain and fuzz-engine versions, and protection against silently
  replacing a corpus or coverage report,
- cross-version and differential storage testing against the reference model,
- ASan, UBSan, TSan, Miri, or platform-equivalent sanitizer/evidence runs where
  applicable to host-boundary and parser crates,
- allocation, recursion, nesting, decoded-size, and execution-time limits for
  every parser and decoder, with fuzz targets asserting those limits,
- release-candidate fuzz threshold policy defining required campaign duration
  or execution count per target and the artifacts required for sign-off,
- corpus and coverage artifacts bound to the reviewed release commit,
- failure reproduction commands recorded in the permanent release evidence for
  every fuzz-discovered defect,
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

Goal: complete the retention and compaction policy over the minimal
domain-local compaction introduced earlier in the storage kernel.

Deliverables:

- retention policy model,
- tombstone retention rules,
- compaction eligibility checks,
- policy/key-domain preserving compaction rules,
- integration with snapshot roots, rollback roots, legal holds, backup roots,
  audit roots, and freshness-anchor generations,
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

## v0.47.1 - API Credential, Session, And Revocation Model

Goal: make authenticated API credentials bounded, replay-resistant, revocable,
and auditable before production clients rely on long-lived access.

Deliverables:

- credential/session model for mTLS identities, signed local tokens, passkeys,
  service tokens, bootstrap tokens, emergency capabilities, and future
  extension credentials,
- binding to tenant, principal, device, workload, purpose, policy epoch,
  allowed scopes, expiry, replay nonce, and revocation epoch,
- revocation list or revocation root included in manifests, audit roots, and
  freshness-anchor state where required,
- session renewal and rotation policy that does not silently expand authority,
- tests for replayed token, stale revocation epoch, wrong device/workload,
  wrong purpose, expired credential, scope escalation, and reused bootstrap or
  emergency capability.

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
- redacted metrics for freshness-anchor availability, anchor advance failures,
  WAL lag, checkpoint lag, compaction debt, scrub coverage, quarantined files,
  backup health, backup chain depth, restore-drill age, key-provider latency,
  KMS lookup rejection counts, resource-pool saturation, quota denials, and
  legal-policy denial categories,
- redacted diagnostic output,
- metric cardinality and label policy so tenant, compartment, fact, key, world,
  actor, policy-token, and query values cannot leak through metric labels,
- no-secret log tests,
- operator troubleshooting runbook,
- tests that metrics and health output remain useful while redacting secrets,
  labels, key slots, exact policy tokens, exact fact identifiers, and
  classified world names.

## v0.50.0 - Performance And Load Evidence

Goal: establish honest single-node performance limits before 1.0.

Deliverables:

- write/read benchmark harness,
- recovery benchmark harness,
- policy-planner benchmark harness,
- p50, p95, p99, and p99.9 latency measurements for reads, writes, commits,
  recovery, manifest selection, key-provider lookup, backup chunking, restore,
  and policy planning,
- steady-state compaction load evidence, mixed read/write/backup/key-rotation
  workload evidence, backup plus foreground-write interference evidence, and
  quota/resource-scheduler behavior under pressure,
- statistical regression budgets with stored historical artifacts,
  dedicated-runner profile, run variance, warm/cold cache state, dataset shape,
  compression ratio, encryption profile, filesystem, mount options, and
  rootless container volume mode,
- objective release pass/fail criteria: minimum sample count, warm-up policy,
  confidence interval, permitted measurement noise, maximum permitted p99 and
  p99.9 regressions, throughput floors, recovery-rate floors, amplification
  ceilings, and advisory versus release-blocking result classes,
- zero tolerance for committed-data loss, silent corruption, isolation
  violations, unauthorized policy bypass, freshness-anchor rollback, or audit
  deletion/reordering in performance or endurance evidence,
- baseline-change governance naming who may approve a new baseline, what
  evidence is required, and a rule that a slower build cannot replace the
  baseline merely to make future comparisons pass,
- 24-hour minimum endurance smoke and 72-hour target endurance run before
  production claims, including compaction, scrub, backup, restore drill,
  key-rotation, manifest checkpoint, and policy-planner activity,
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
- graceful shutdown and restart-storm runbook covering write-admission stop,
  transaction cancellation, group-commit drain, background publication stop,
  optional flush/checkpoint choice, audit and anchor ordering, directory-lease
  release, bounded timeout, forced termination, and repeated startup failure,
- shutdown/restart-storm failure-injection evidence for active writes,
  background compaction/scrub/deletion, audit emission, anchor advancement,
  manifest publication, checkpointing, and lease release,
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

## v0.52.2 - Policy And Law-Pack Evaluator Safety

Goal: make legal/compliance and policy-pack execution deterministic, bounded,
and non-privileged before signed packs can influence access decisions.

Deliverables:

- policy/law-pack evaluation model that is deterministic, resource-bounded,
  side-effect-free, and independent of network, wall-clock, filesystem, random,
  or process state during evaluation,
- explicit decision on data format or language: declarative tables, bytecode,
  embedded DSL, or compiled Rust admission, with a default preference for
  declarative data over executable code,
- gas, step, recursion, memory, input-size, and output-size limits for every
  evaluator path,
- versioned conformance test suite for allow, constrained-allow,
  approval-required, deny, stale-pack, conflict, and ambiguity outcomes,
- sandbox or compile-boundary rule for any future executable policy helper,
- tests for nontermination attempts, oversized rules, recursive references,
  conflicting packs, stale source locks, network/filesystem access attempts,
  and nondeterministic output.

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

## v0.56.0 - Final Backup And Restore Qualification

Goal: qualify backup and restore after schema catalog, retention, quotas,
runtime resource isolation, observability, and legal-policy integrations exist.

Deliverables:

- end-to-end backup qualification across the `v0.35.4` backup engine,
  `v0.40.0` runtime resource pools, `v0.45.0` schema catalog, `v0.46.0`
  retention policy, `v0.48.0` quotas, `v0.49.0` observability, and `v0.54.0`
  legal operation decision engine, plus `v0.55.0` sovereign placement intent,
- schema-evolution backup/restore tests covering compatible, incompatible,
  migrated, unknown, and intentionally deferred schema/catalog states,
- retention, legal-hold, privacy-erasure, tombstone, rollback-root,
  backup-root, and quota enforcement during backup creation, retention,
  deletion, restore, and in-place recovery,
- resource-scheduler and quota tests for online backup during foreground
  reads/writes, compaction, scrub, key rotation, and checkpoint work,
- observability qualification for backup health, backup chain depth,
  restore-drill age, backup RPO/RTO, restore throughput, temporary-space use,
  and redacted failure reasons,
- legal-policy enforcement tests for backup, restore, export-like restore,
  cross-jurisdiction restore, AI/search/index backup content, and denied
  in-place recovery,
- sovereign-placement enforcement for backup destination and storage-provider
  jurisdiction, temporary upload and restore-staging locations, cross-region
  restore, synthetic-full generation, and placement changes after policy, key,
  law-pack, or passport epochs change,
- deletion or quarantine workflow for unlawfully or incorrectly placed backup
  fragments, with audit proof, legal basis, key-slot handling, and no silent
  policy bypass,
- automated restore drill from full and incremental backups with corruption
  injection, missing chunks, wrong key, stale law pack, stale policy epoch,
  quota exhaustion, and anchor mismatch,
- final production backup/restore runbook with measured RPO, RTO, restore
  throughput, storage overhead, temporary-space requirement, operational
  authority, and non-claims.

## v0.56.1 - Cross-Platform Production Qualification

Goal: close the gap between portable design and actual v1.0 production
platform support.

Deliverables:

- explicit support tier for every OS/filesystem pair: production-supported,
  supported with caveats, compile-preview, experimental, or unsupported,
- real production host-storage adapters for every v1.0 production-supported
  OS family, covering Unix/Linux, Windows, macOS, and BSD if they remain in the
  production tier,
- permissions, ownership, ACL, symlink, reparse-point, path traversal,
  advisory/mandatory locking, positional I/O, atomic replacement, file sync,
  directory sync, rename, truncation, and temporary-file semantics documented
  and tested for each production-supported platform,
- crash recovery and power-loss tests per supported filesystem or an explicit
  non-claim that keeps that OS/filesystem pair out of the production tier,
- backup/restore and storage-format compatibility tests across production
  platforms, including backups created on one production platform and restored
  on another where the support matrix claims portability,
- x86_64 and AArch64 execution tests for core engine, host storage, recovery,
  backup/restore, and rootless/container profiles where applicable,
- downgrade or support-tier narrowing rule: if equivalent security/durability
  semantics cannot be proven for an OS/filesystem pair, the v1.0 checklist must
  list it as compile-preview or unsupported rather than production-supported,
- tests for Windows reparse-point attacks, Unix symlink attacks, BSD/macOS
  directory sync behavior, Linux filesystem matrix behavior, cross-platform
  backup/restore, and architecture-neutral format replay.

## v0.56.2 - Final Extension API Compatibility Freeze

Goal: freeze the complete optional-extension API only after schema,
retention, API credentials, legal/compliance passports, sovereign placement,
backup/restore, resource governance, and cross-platform support are known.

Deliverables:

- final public API review for facts, worlds, labels, catalog/schema contracts,
  policy proofs, authorized query handles, projection journals, audit
  submission, storage roots, encryption domains, operation passports,
  legal/compliance passports, retention/deletion hooks, backup/restore hooks,
  and migration interfaces exposed to extension crates,
- compatibility contract for `v1.1.0` through `v1.6.x` extension crates,
  including stable APIs, explicitly unstable APIs, deprecation windows,
  feature gates, and minimum supported core database version,
- extension removal behavior proving canonical facts, schema descriptors,
  audit records, projection metadata, and backups remain readable or
  quarantinable without the removed extension loaded,
- atomic and resumable extension upgrade/migration protocol with dry-run,
  checkpoint, rollback/restore point, idempotency key, audit proof, and
  resource budgets,
- capability revocation behavior for existing extension sessions, queued jobs,
  background projections, migration tasks, and long-running queries,
- cross-extension dependency policy: default deny, or explicit versioned
  dependency declarations with cycle rejection, capability narrowing, and
  upgrade ordering,
- extension panic, timeout, memory exhaustion, disk exhaustion, parser failure,
  and malformed manifest containment tests,
- final compile-only fixtures for publishing, messenger, forum, forge,
  relationship/feed/media, mailbox, collaboration, privacy, and source-state
  extension families proving they compile against the frozen API without
  entering the mandatory core build,
- tests for omitted extension, removed extension, upgraded extension,
  downgraded extension rejection, revoked capability, cross-extension
  dependency conflict, and extension attempting raw WAL, filesystem,
  key-provider, or unrestricted database access.

## v0.57.0 - 1.0 Release Candidate

Goal: freeze the 1.0 feature set and run final release evidence.

Deliverables:

- release-candidate notes,
- complete security review checklist,
- rootless Podman release gate,
- final performance and durability rerun after legal, placement, and backup
  qualification, including required 24-hour endurance and target 72-hour
  endurance over mixed backup, restore, legal-policy, sovereign-placement, key
  rotation, compaction, scrub, checkpoint, and foreground read/write traffic,
- crash/recovery and corrupted-backup matrix rerun after final backup and
  placement integration,
- upgrade from the oldest supported format and downgrade-rejection run against
  the final migration and compatibility rules,
- comparison against stored `v0.50.0` regression budgets for latency,
  throughput, recovery, compaction, backup, restore, and policy planning,
- final release pass/fail report applying the `v0.50.0` blocking criteria,
  including confidence intervals, p99/p99.9 decisions, throughput/recovery
  floors, amplification ceilings, and any approved baseline changes,
- final fuzz/property qualification rerun against the exact reviewed release
  candidate commit, including all `v0.44.0` campaign thresholds, sanitizer or
  Miri runs where applicable, property tests, cross-version tests,
  reference-model differential tests, coverage artifacts, minimized corpora,
  and zero unresolved crashes, hangs, resource-bound violations, or correctness
  divergences,
- permanent pentest digest committed after the maintainer reports the pentest
  is green,
- GitHub Actions must be green after the pentest digest/release-prep commit,
- if GitHub fails, fix the issue, update tests/docs/release evidence where
  relevant, commit again, and wait for GitHub to go green,
- tag and tag push happen only after the maintainer explicitly says GitHub is
  green and instructs Codex to tag/push,
- signed tag when local Git signing is available,
- final optional extension integration checklist,
- no new feature work without explicit deferral decision.

## v1.0.0 - Production World Database

Goal: first serious production-ready `skrifheim`.

Deliverables:

- durable single-node fact/world engine,
- physical block-structured storage layout with canonical key ordering,
- block-structured storage kernel with memtables, immutable sorted runs,
  version sets, iterators, filters, caches, and minimal compaction,
- strict transaction semantics,
- model-checked concurrency and group commit with explicit durability SLO,
- durable primary, causal, supersession, invalidation, and snapshot visibility
  indexes,
- supported non-rollbackable freshness-anchor provider implementation for
  production profiles,
- immutable world revisions with compare-and-swap world heads,
- canonical fact transcript, signature verification, and verified ingest gate,
- policy-aware query planning,
- cost, leakage, and vectorized execution plan,
- trusted planner context and bound query proofs,
- full result security label propagation, including compartments and
  releasability/dissemination constraints,
- query inference budgets and aggregation controls,
- access-pattern and query-shape leakage model,
- causal blast-radius invalidation and quarantine support,
- key hierarchy and lifecycle,
- scoped key release boundary so the main database process is not a god-mode
  key holder in production profiles,
- signed declassification proof model,
- capability-scoped AI derivation cones,
- propagated confidence fused with mandatory access control,
- encrypted WAL, segments, indexes, projections, backups, exports, and audit logs,
- WAL anti-splicing identity, chained frame digests, and transaction commit
  roots,
- WAL v2 torn-tail recovery and append-after-corruption prevention,
- crash-ordering and failure-injection harness for WAL, flush, manifest,
  checkpoint, pruning, audit, and freshness-anchor transitions,
- encrypted inner storage metadata with minimal plaintext outer framing,
- explicit crypto-erasure granularity and key-slot deletion proofs,
- admitted entropy/CSPRNG provider and nonce-generation policy,
- query-result classification,
- trusted time and monotonic sequence model,
- legal/compliance passport foundations,
- law-pack metadata admission,
- deterministic bounded policy/law-pack evaluator safety,
- legal operation and transfer decision skeleton,
- sovereign placement intent compiler,
- compromise and recovery playbooks,
- schema catalog and versioned contracts,
- retention, tombstone, and compaction policy,
- authenticated API boundary,
- API credential, session, and revocation model,
- resource governance and quotas,
- tenant/classification-aware runtime isolation and resource pools,
- observability without secret leakage,
- performance and load evidence,
- tamper-evident manifests,
- storage-directory single-writer lease and downgrade-protected format
  migration model,
- defined storage-upgrade support window with golden databases for every
  supported path,
- qualified production platform support matrix with real host adapters,
  crash/power-loss evidence, cross-platform backup/restore, and x86_64/AArch64
  execution tests for production-supported OS/filesystem pairs,
- online integrity scrubbing, protected-root reference accounting, and
  capacity/file-count governance,
- non-destructive restart repair with preserved original evidence, staged or
  copy-on-write repair, provenance/audit records, and explicit authority for
  destructive salvage,
- externally anchorable chained audit roots,
- production fuzz qualification rerun on the final release-candidate commit,
- SBOM, dependency-tree, source-lock, fuzz-evidence archive, and
  release-evidence gates,
- local snapshot and rollback retention with locked archive/recovery worlds,
- rootless Podman deployment,
- backup/restore engine plus final schema, retention, quota, observability,
  runtime-scheduling, and legal-policy qualification,
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

- `skrifheim-cluster-protocol` transport-independent signed message types and
  `skrifheim-cluster-host` networking/consensus implementation boundary, with
  no consensus or network dependency in core database crates,
- explicit crash-fault versus Byzantine-fault threat model for local cells,
  with non-claims for any failure class not implemented by the selected
  protocol,
- node registry,
- local shard/range assignment,
- local consensus skeleton,
- membership-change and joint-consensus model covering voting configuration,
  fencing tokens, term/index durability, snapshot installation, log
  compaction, and recovery after interrupted reconfiguration,
- sovereign-cell rule that consensus is local to a lawful cell and no global
  consensus system may bypass local policy, legal, key, or witness vetoes
  across legal boundaries,
- local failover preflight,
- split-brain prevention and fencing tests in the initial local-cell runtime,
  not deferred only to later failover work,
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
- signed-intent reconciliation rule: the control plane proposes placement,
  replication, backup, failover, projection, and tunnel changes but cannot
  grant data access or override destination-side policy decisions,
- control-plane storage rule requiring topology, placement intent, membership,
  witness state, and remediation proposals to use the same authenticated
  storage, freshness-anchor, audit, backup, and recovery rules as ordinary
  database state,
- policy, key, and law-pack epoch tracking,
- tests for control-plane proposals that cannot bypass local vetoes.

### v2.2.0 - Policy-Scoped Tunnel Fabric

Goal: open encrypted database tunnels with identity and legal scope.

Deliverables:

- node identity handshake,
- signed peer maps,
- tunnel policy model,
- operation and data-passport binding on streams,
- stream binding to source and destination passports, operation proof, exact
  manifest/world roots, law/policy/key epochs, expiry, and idempotency nonce,
- destination-side veto before protected bytes move, with stale policy,
  attestation loss, witness loss, or legal disagreement sealing the stream or
  world instead of silently degrading,
- replication and health streams,
- tests that denied labels cannot cross an otherwise healthy tunnel.

### v2.3.0 - Geo Replication And Witness Roles

Goal: replicate safely across cells where policy permits it.

Deliverables:

- commit-log shipping,
- snapshot shipping,
- replicate only authenticated encrypted blocks whose complete domain is
  admitted at the destination; no plaintext downgrade or cross-domain key
  weakening for convenience,
- ciphertext portability decision: either bind blocks to a cluster-wide
  immutable object identity in AEAD associated data, or verify and re-encrypt
  through an authorized destination transfer boundary when storage generation,
  file identity, block offset, or key domain differ,
- privacy-erasure, tombstone, and legal-hold propagation rules for lagging,
  partitioned, sealed, or recovering replicas, including garbage-collection
  watermarks and proof that erased key slots cannot remain readable on a
  delayed replica,
- explicit rejection of global cross-cell transactions before a narrowly
  scoped future protocol exists; use durable operation intents or sagas across
  sovereign cells by default,
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
