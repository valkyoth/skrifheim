# skrifheim Implementation Plan

Status: planning document

Database name: `skrifheim`

1.0 target: a production-ready causal world-state database for
security/compliance-first application backends. CMS, messenger, forum, forge,
collaboration, and other application-family support must be optional
compiled-in extension crates that compose facts, worlds, policies, releases,
projections, audit, and AI artifact provenance without becoming mandatory core
database semantics.

## Core Position

`skrifheim` stores signed, versioned, policy-bound facts about worlds. It is not a SQL compatibility project and not a generic multi-model database. Relational, document, graph, vector, search, render graphs, AI context packs, and analytics are projections over canonical facts.

The canonical layer must be boring, durable, testable, and auditable. The
futuristic behavior comes from world overlays, causal invalidation,
policy-aware planning, cryptographic declassification, capability-scoped AI
derivation, confidence propagation, and derived projections that can be rebuilt
from truth.

Application-specific plans are requirement sources, not architecture owners.
`skrifheim` must not copy product schemas, workflow shortcuts, or service-stack
assumptions from consuming projects. Those projects must adapt to
`skrifheim`'s security and compliance model: facts first, policy before read or
write, explicit legal basis where needed, encryption domains for protected
surfaces, bounded verification, tamper-evident audit, redacted diagnostics, and
rebuildable projections.

External authority inputs are also requirement sources, not memory-based
implementation shortcuts. Legal rules, compliance standards, cryptographic
specifications, storage formats, parser fixtures, and conformance suites must
be pinned to exact source revisions, hashes, dates, licenses, and non-claims
before `skrifheim` treats them as behavior evidence.

Multi-application deployments are a first-class planning constraint. The
database must support strict separation between identity authority, shared
account/profile data, operator identity, support identity, service identity,
guardian authority, and product-owned data. Products should receive minimal
derived claims, not raw private account facts, unless policy, legal basis,
consent, purpose, and audit all authorize the disclosure.

Cross-product privacy workflows must be orchestrated rather than centralized by
one product. Export, deletion, retention, deactivation, legal hold, and
breach-response jobs need product-owned contracts, dry-run counts, resumable
checkpoints, immutable audit, and fail-closed behavior when a required product
contract is missing.

Messaging workloads split into at least two database-supported modes: true
E2EE metadata boundaries where the server cannot read message bodies, and
mailbox-style encrypted-at-rest systems where server-authorized decrypt,
search, support access, and abuse handling require explicit proof records,
purpose limitation, key epochs, and audit binding.

Server-blind collaboration workloads add a stricter variant of the E2EE model:
the database may plan over workspaces, channels, conversations, memberships,
devices, invitations, encrypted envelopes, delivery state, read cursors, legal
hold, retention, exports, plugins, and integration metadata, but plaintext,
message search, client group secrets, attachment keys, local caches, and
decrypted indexes stay client-owned unless an explicit future declassification
or client-provided evidence path is used.

## Non-Negotiable Engineering Rules

- Rust stable `1.97.1`, edition 2024, workspace resolver `3`.
- Latest stable Rust and dependency versions are re-checked before dependency/toolchain changes.
- Core library crates use `#![no_std]` where possible.
- External crates are exceptional: discuss, verify, document, and test before use. Prefer local implementation.
- No unsafe Rust in core crates. If unsafe ever becomes unavoidable, isolate it in a dedicated boundary crate after policy admission.
- Main crate `skrifheim` orchestrates focused crates.
- Normal `.rs` files should stay under 300 lines and must stay under 500 lines unless documented.
- Every subsystem must be testable without a production cluster.
- Security, provenance, and policy checks are part of the planner, not application-side decoration.
- Cryptographic hash policy is production-critical. Scaffold hashes may be used
  only for non-secret identifiers until the production digest boundary lands.
  Durable storage authority must use admitted SHA-3/SHAKE full-width digests
  with a configurable strength profile.
- Release evidence is part of the product. A release is not complete unless its
  pentest digest, release notes, SBOM, relevant dependency-tree snapshots, and
  source-lock evidence are committed where the release gate can verify them.

## Workspace Shape

- `skrifheim-core`: IDs, timestamps, labels, values, and shared errors.
- `skrifheim-fact`: causal fact records, validation, evidence, confidence, supersession, invalidation.
- `skrifheim-world`: production, draft, simulation, legal/audit, user-local, mission-capsule worlds.
- `skrifheim-policy`: clearance lattice, compartment checks, releasability, planner decisions.
- `skrifheim-crypto`: crypto-agile envelopes, crypto-material epochs, lifecycle event ordering, algorithm identifiers, threshold signatures, and quorum proofs.
- `skrifheim-audit`: identities, attestation evidence references, audit events, and audit-log protection metadata.
- `skrifheim-storage`: WAL, immutable segment metadata, Merkle manifests, content-addressed blobs.
- `skrifheim-storage-host`: host-file WAL and segment helpers under `crates/`
  that use `std` as the explicit boundary outside the `no_std` core crates.
- `skrifheim-query`: query intent, policy-aware planning, context-pack planning.
- `skrifheim-compliance`: future legal/compliance passports, law-pack metadata, and legal transfer decisions.
- `skrifheim-cluster`: future cell, control-plane, tunnel, placement, and failover planning primitives.
- `skrifheim-ext-*`: future optional application-family extension crates that
  depend on core crates but are not required for the mandatory database core.
- `skrifheim`: CLI/server orchestration.
- `xtask`: repeatable local automation.

## Phase 1: Secure Fact Kernel

Build append-only facts with:

- entity, predicate, object,
- valid time and transaction time,
- source and evidence,
- classification label and compartments,
- policy epoch and crypto epoch,
- confidence,
- caused-by, supersedes, and invalidates links,
- signatures and content identity.

`FactId` allocation is a database design decision, not just an identifier
helper. Before the transaction model hardens, `skrifheim` must decide whether
fact IDs are monotone/timestamp-derived, random, content-derived, or a hybrid
tenant/world scoped identity. Monotone IDs can expose write ordering. Random IDs
need write-time uniqueness checks. Content-derived IDs couple identity to
canonical serialization and signature policy. The chosen strategy must be
documented before MVCC, indexes, manifests, and causal graph traversal rely on
fact identity.

The first correctness question is: can `skrifheim` prove what was true, who asserted it, which policy governed it, and which evidence supported it?

Fact payloads are bounded at the canonical layer. Text and byte values must be
rejected before durable ingest when they exceed the active fact-object limit.

Causal links are not passive metadata. They must become a live dependency graph
that can answer blast-radius questions such as: which facts, decisions,
projections, releases, and AI artifacts become tainted when a source, model,
worker, key, or upstream fact is revoked or marked compromised?

## Phase 2: World Branches

Worlds are Merkle/DAG overlays, not full copies.

Required operations:

- create,
- inherit,
- fork,
- diff,
- merge,
- discard,
- promote,
- rollback.

World kinds include production, staging, user-local, agent scratchpad, simulation, legal/audit, and mission capsule.

Deterministic world ID derivation is length-separated, tenant-scoped, and
non-secret. The current BLAKE3 derivation is scaffold-only and must not become
the long-term storage trust root. Before world identity becomes durable storage
authority, `skrifheim` must introduce admitted SHA-3/SHAKE digest primitives,
configurable digest strength, and full-width world identity digests.

`WorldId` is a compact handle, not a bearer capability or sole authorization
input. Storage must still reject creation when a compact `WorldId` already
exists for a different `(tenant_id, kind, depth, parent, name)` tuple, but
future storage authority must key and verify the full `WorldIdentityDigest`.
Planned digest strength profiles are:

- `Sha3_256` for normal high-security deployments,
- `Sha3_384` for conservative long-horizon deployments,
- `Sha3_512` for military or post-quantum cautious deployments,
- `Shake256_256` and `Shake256_512` where extendable-output hashing is the
  better fit for manifests or protocol transcript binding.

Storage and orchestration must enforce tenant-level aggregate world budgets,
including maximum world count and total tracked fact references across all
world overlays. Per-world list bounds are not a substitute for tenant quota
enforcement.

Stable world identity must not be confused with mutable world state.
`WorldId` identifies a branch; `WorldRevisionId` identifies an immutable
content/root revision. Forks must record the exact parent revision they forked
from, world-head updates must use compare-and-swap over the expected revision,
and promotion must be a three-way merge over fork base, current parent head,
and candidate head.

## Phase 3: Storage Kernel

Implement:

- append-only WAL,
- strict recovery state machine,
- immutable policy-scoped block tables/segments,
- manifest snapshots,
- admitted SHA-3/SHAKE digest strength policy before durable storage keys,
- full-width `WorldIdentityDigest`, `ContentDigest`, and `ManifestDigest`
  primitives,
- canonical record and transaction-batch encodings,
- primary, secondary, causal, supersession, invalidation, and snapshot
  visibility key formats,
- logical storage blocks with offset tables or restart arrays, sparse indexes,
  filters, compression IDs, checksums, AEAD envelopes, and bounded
  decompression,
- WAL-backed memtables, immutable sorted runs, version sets, streaming
  iterators, merge iterators, filters, and block/index caches,
- physical partitioning by tenant, region, classification, compartment, key
  epoch, and policy compatibility class,
- checksums,
- content-addressed blocks,
- policy and encryption boundary metadata,
- corruption rejection instead of silent repair.

Local rollback is a storage and policy feature, not a mutable undo button.
`skrifheim` should retain selected manifest/world roots as protected snapshot
facts, then let operators inspect or fork those roots into locked archive,
recovery, simulation, or production-promotion workflows. Rollback must create
signed proof facts that record who requested it, why, which snapshot root was
used, which policy and crypto epochs applied, and which target world received
the recovery. Production state is changed only through normal policy-checked
world promotion.

Rollback retention is allowed to cost extra space. Compaction must treat
protected snapshot roots as live references and report how much space is kept
only for rollback. Exceptional purge of rollback-protected material is a
separate dangerous operation requiring explicit authority, legal basis, audit
proof, and preferably crypto-erasure; it must not be an accidental side effect
of retention expiry or compaction.

The first segment encoding pass must evaluate the cost of duplicating metadata
in both the segment header and footer. Full mirroring improves tail validation
and corruption detection, but it doubles those fields on disk. That is
negligible for large immutable segments and potentially visible for very small
segments. `v0.18.0` must either keep full mirroring with documented overhead or
define a compact footer/trailer that preserves mismatch detection.

`v0.18.0` keeps full mirrored fixed-size segment headers and footers. The
segment reader must reject unexpected domains, body CRC mismatches,
header/footer metadata mismatches, truncated files, and trailing bytes before a
segment can be accepted. Content-digest verification is a required injected
trust-boundary operation until the admitted production digest engine computes
the actual SHA-3/SHAKE body hash.

The opaque whole-segment scaffold must not become the production layout.
Before manifests harden, `skrifheim` needs a physical storage decision: likely
an LSM-oriented canonical fact store with a small copy-on-write metadata tree
for compact mutable metadata. Segment bodies should become aligned encrypted
blocks with sparse indexes and filters so point lookups do not allocate or read
entire files.

`v0.18.3` is the release that must admit and implement the production digest
and AEAD primitives. After that milestone the cryptographic building blocks and
generic envelope transcript APIs exist, but concrete segment/block encryption
arrives with the physical storage layout in `v0.18.11`, and WAL-v2 encryption
arrives with the ordered log format in `v0.18.12`. Manifests, checkpoints, and
recovery must not claim tamper resistance until those concrete formats
instantiate the admitted primitives. CRC64 remains only a structural corruption
check.

Nonce, key, salt, replay nonce, and random identifier generation require an
admitted entropy/CSPRNG provider before production crypto paths exist.
Production builds must fail closed when entropy is unavailable, and
deterministic test providers must stay test-only.

The storage crypto boundary must also handle log splicing and metadata
confidentiality. WAL frames need database/log generation, LSN, transaction
ordinal, previous-frame digest, and commit-root binding so valid encrypted
frames cannot be deleted or reordered. WAL and segment formats must split a
minimal plaintext outer header from encrypted inner metadata; tenant, world
revision, classification, compartment, policy epoch, key ID, transaction range,
and sensitive content identity should not be plaintext by default.

Signed manifests and AEAD authenticate a state, not freshness. Before manifest
selection and startup recovery become authoritative, production profiles need a
non-rollbackable freshness anchor: TPM or HSM monotonic state, remote witness,
transparency service, WORM/offline operator checkpoint, or threshold-held
external checkpoint. Active startup must fail closed when local storage is
older than the anchor; historical roots can be opened only through explicit
recovery workflows.

The freshness anchor needs a real provider before startup recovery depends on
it. At least one reference provider, such as a remote witness client or
TPM-backed host adapter, must implement idempotent compare-and-advance,
timeout and unavailable-provider behavior, provisioning, replacement,
equivocation detection, and disaster-recovery rules.

The storage directory itself needs an identity and single-writer lease. Two
processes must not be able to advance the same WAL/manifest directory
concurrently. Storage-format upgrades must be idempotent, signed, audited,
downgrade-protected, and recoverable after interruption without partially
trusted state.

WAL v2 must be ordered, authenticated, and repairable. Startup needs to find
the last authenticated frame boundary, distinguish a recoverable torn tail from
interior corruption, truncate or rotate safely, and validate the existing tail
before appending. Ordinary pre-commit failures should not need abort records if
uncommitted state is never flushed into canonical tables.

Until WAL v2 replaces the scaffold format, WAL-v1 writers must be treated as
dangerous immediately, not at the end of the WAL-v2 work. They should poison
themselves after any append, partial write, flush, or sync error, force tail
validation before further append, report durable LSN or ambiguous status where
possible, and use transaction idempotency keys so retry cannot duplicate a
commit.

Production digest and AEAD admission is separate from production storage
encryption. The crypto milestone admits primitives and generic envelopes; the
block, segment, WAL, and manifest milestones instantiate those primitives only
after their concrete byte formats are chosen. Each durable format milestone
must commit golden compatibility fixtures at the same time it introduces the
format.

Every durable format starts with a compatibility contract, not only a byte
layout: major/minor version semantics, required and safely ignorable feature
bits, minimum reader and writer versions, canonical encoding, unknown-field
behavior, and forward/backward read policy. Later migration work can automate
upgrades, but the compatibility rules must exist as soon as WAL v2, block
tables, and manifests become durable.

The database process must not become a god-mode key holder. Production key
release must go through a scoped KMS, HSM, privilege-separated key service, or
equivalent provider that checks tenant, compartment, purpose, policy epoch,
workload identity, encryption domain, and bound policy proof before releasing a
key operation. High-assurance deployments may split processes or instances by
classification or compartment.

Crypto-erasure granularity is a storage-layout decision. Before real encrypted
segments are trusted, `skrifheim` must choose per-object or erasure-group data
keys, wrapped-key indexes, backup key-slot deletion, compaction/re-encryption
rules, snapshot and legal-hold exceptions, and proof that every readable key
slot was removed.

`v0.18.4` pulls fuzzing forward for the WAL and segment byte parsers. The
general fuzz/property baseline remains later, but hand-written parsing of
untrusted WAL and segment bytes must have a deterministic fuzz smoke from this
storage phase onward.

Failure injection is part of the storage design, not late hardening. The
engine first needs reusable failpoints for writes, short writes, interrupted
syscalls, sync, link/rename, manifest swaps, WAL truncation, directory fsync,
ENOSPC, quota exhaustion, and EIO, with subprocess crash tests comparing
recovered state to an in-memory oracle. Concrete coverage for manifest swaps,
memtable flush, block tables, compaction, group commit, and durable commit
belongs in the milestones that introduce those implementations.

Filesystem behavior is part of the trust boundary. `skrifheim` needs a
supported-filesystem matrix, explicit network-filesystem non-claims until
locking and sync semantics are proven, and practical power-cut tests using
loopback/dm-flakey or an equivalent mechanism.

Compaction must be designed and minimally implemented early. It determines key
ordering, tombstone and range-tombstone semantics, snapshot retention,
rollback-root liveness, encryption-domain grouping, file-count growth, write
amplification, iterator semantics, and WAL checkpointing. Compaction must
preserve tenant, policy, region, encryption, and MVCC boundaries.

The manifest is the authoritative storage root. It must name live immutable
tables, world heads, schema roots, key state, WAL checkpoint, audit root,
freshness anchor, and protected roots. Directory scans may help recovery,
quarantine, and cleanup, but they must not become an alternative source of
truth for active state.

An authoritative manifest must be keyed-authenticated before its contents are
trusted. Its minimal plaintext outer header should reveal only what is needed
to locate and authenticate the encrypted inner manifest; table inventories,
tenant/domain metadata, world heads, schema roots, key-state digests, and audit
roots belong inside authenticated encrypted metadata unless an explicit
public-header exception is documented.

Manifest-key bootstrap must not trust the encrypted manifest before the key is
located. Outer-header fields are only key-location hints; database identity and
manifest generation construct the key-provider scope, opaque key slots prevent
unnecessary key-ID disclosure, dual-key rotation handles previous-key fallback,
and crashes between key activation, manifest publication, checkpoint update,
and anchor advancement must recover without accepting key redirection.

Fixed-width counters are security boundaries. LSNs, WAL generations,
transaction IDs, commit sequences, file numbers, manifest generations, crypto
epochs, lifecycle sequences, anchor generations, and backup generations use
checked arithmetic, pre-maximum exhaustion thresholds, explicit rollover
incarnations where safe, and fail-closed behavior where rollover would break
ordering, nonce uniqueness, deletion safety, checkpoints, backups, or
freshness anchors.

Space reclamation is a correctness feature, not only housekeeping. WAL pruning,
obsolete-file sets, oldest-active-snapshot watermarks, explicit snapshot
leases, maximum snapshot/read-transaction age, per-tenant pin quotas, snapshot
and iterator pins, tombstone retention, rollback/archive/legal-hold roots,
orphan-table recovery, staged-file cleanup, and file-number no-reuse all need
explicit rules before compaction or cleanup can delete bytes. Cleanup,
migration, manifest changes, writers, checkpointing, and obsolete-file deletion
must be serialized by the database-directory lease.

Hot/cold tiering and blob deduplication are storage features only inside
compatible security domains. Dedup must not reveal plaintext equality across
tenant, compartment, policy, key epoch, or legal boundaries. Tier movement must
preserve snapshot, rollback, backup, audit, and legal-hold liveness.

Backup and restore must graduate from skeleton to production before 1.0.
Online backups need manifest-generation pinning during concurrent writes,
full/incremental recovery-point semantics, resumable verified chunks,
backup-specific key rotation and crypto-erasure behavior, retention and
orphan-upload cleanup, restore into a new database identity versus authorized
in-place recovery, automated restore drills, corruption injection, and measured
RPO, RTO, throughput, and temporary-space requirements.

The storage engine needs a block cache rather than a traditional dirty-page
buffer pool. Dirty data primarily lives in WAL-backed memtables; immutable
tables can use sharded data/index/filter caches with tenant and security-domain
accounting, cache keys that include database generation, file number, block
offset, encryption domain, policy epoch, and key epoch, active-iterator
pinning, and rules preventing decrypted blocks from being reused across
incompatible authority contexts. Iterator and snapshot pins prevent physical
deletion of referenced files, but they must not force every referenced block to
remain resident in cache. Deployment profiles should choose page-cache-heavy or
userspace-cache-heavy behavior instead of accidentally double-caching at full
size.

Resource governance starts with storage, not query execution. WAL/table extent
preallocation, table size classes, temporary-space reservation, minimum
free-space margins, a disk-exhaustion escape reserve for WAL repair, manifest
publication, audit emission, and orderly shutdown, compaction-debt admission,
file-count/open-file limits, and per-tenant I/O throttling must fail before a
commit can exhaust shared storage.

The first real database milestone is a narrow storage spine, not another
metadata-only model: `WriteBatch -> WAL v2 -> durable barrier -> memtable ->
immutable table flush -> manifest swap -> restart recovery -> point read ->
domain-local compaction`. This belongs immediately after manifests, so startup
recovery can recover authoritative database state instead of only structural
metadata. Later transaction, query, projection, and extension work should
consume that spine instead of evolving beside an unstable storage format.

The in-memory transaction model must provide read-your-writes behavior before
durable commit. Reads inside a transaction should consult transaction-local
inserts, hides, supersessions, invalidations, and predicate changes before the
committed snapshot, while other transactions must not see those uncommitted
writes. This belongs in the v0.21.0 transaction state model, before strict
serializable validation and WAL commit are wired in.

Durable commit should use MVCC plus optimistic serializable validation, not
long-lived database-wide locks. The intended commit path is: validate policy,
construct immutable write batch, reserve commit sequence, validate conflicts,
append ordered WAL frames, bind mandatory audit evidence through either an
authenticated audit-record digest in the commit transcript or a transactional
audit outbox in the same WAL transaction, group fsync, publish versions and
world-head CAS updates, then acknowledge. A crash after the durable barrier but
before publication is redone during recovery; a crash before the barrier must
never expose the transaction as committed, and mandatory-audit operations must
not report success without durable audit evidence or a recoverable outbox item.

Durability and performance must have measurable SLOs early: WAL bytes and
fsyncs per append before transactions exist, raw block throughput, recovery
scan rate, maximum startup time, and policy-token scan cost. Amplification,
cache, compaction, recovery-through-table, commit-latency, contention, and
transaction-memory SLOs move to the milestones where the relevant machinery
exists. Benchmark evidence must record hardware, filesystem, mount options,
drive-cache mode, dataset distribution, compression ratio, encryption profile,
warm/cold cache state, concurrency, run variance, and rootless container
bind-volume versus overlay-filesystem behavior.

Early performance and integration evidence must be gathered before the storage,
transaction, query, and API shapes are too expensive to change. `v0.20.2`
measures storage/recovery overhead, including mirrored footers and fixed-slot
policy checks. `v0.23.2` exercises transaction storage under local load and
crash scenarios. `v0.24.2` validates a minimal authenticated request boundary
and planner context shape before the native query AST and policy-aware planner
freeze their assumptions.

## Phase 4: Policy And Classification Planner

The planner must answer:

- can this subject know the fact,
- can these facts be joined,
- can this result be cached,
- can this result sync to a device,
- can an AI worker process it,
- can a projection mix these labels,
- can an export lower classification,
- can confidence and provenance support this classification decision,
- what classification does the output carry.

Rejected plans must be deterministic and constant-shape where practical.

Public query construction must not accept caller-supplied labels or
result-classification inputs for stored data. Labels and result security
metadata must be resolved from validated snapshot, schema, and policy state.
Plans must bind tenant, principal, device and workload evidence, world
revision, manifest root, policy epoch, query digest, purpose, expiry, and nonce,
then execution must revalidate that binding.

Derived result security labels must propagate full access-control state:
classification by maximum dominance, required compartments by union,
releasability/dissemination by intersection or deny-all on conflict,
sovereignty by exact-or-saturated scope, bounded policy epoch proof state,
PII-derived state, AI eligibility, and confidence policy metadata. Caches,
projections, exports, backups, AI pipelines, and legal planners must not treat
derived data as less constrained than its inputs.

Aggregation needs real inference controls, not just redacted planner
diagnostics. Before aggregate execution, `skrifheim` must define inference
budgets, minimum cohort sizes, contribution bounds, query-history differencing
detection, consistent suppression, purpose-specific audit, and cross-session
budget aggregation.

Encryption and redaction do not hide every query side channel. The planner must
classify access-pattern and query-shape leakage such as touched segments,
indexes, response size, timing, cache hits, and repeated query forms.
High-assurance profiles must either mitigate those leaks with padding,
batching, private-query, delayed-response, or offline workflows, or reject the
query shape with an explicit non-claim.

Result sovereignty is represented as either an exact bounded jurisdiction set
or a saturated multi-jurisdiction scope. The saturated scope is not a
clearance, compartment, or releasability token. It is a most-restrictive signal
that future export, placement, indexing, backup, AI processing, and
legal/compliance planners must treat as approval-required or deny unless a
later policy layer proves a more specific lawful route.

Break-glass access is not an audit event by itself. The audit layer can record
and validate a break-glass event shape, including device and workload
attestation, but the policy planner must separately define the access effect.
The default security posture is that break-glass must be scoped, time-bounded,
audited before use, and approval-gated; it must not silently grant global
`TopSecret` clearance. `v0.26.1` is reserved to decide whether emergency access
uses temporary scoped authority, a policy-filtered legal/audit view, a
`WorldKind::LegalAudit` isolation world, or an approval-required request that
does not bypass normal policy until approved.

One possible break-glass mechanism is an identity-proofed one-time emergency
capability. The preferred proof should be hardware-backed or institution-backed
where possible, such as FIDO2/passkey, smart card, employee PKI, HSM-held key,
or an encrypted one-time file. Passport, national ID, face template, and
biometric evidence are highly sensitive and should live in a separate encrypted
identity vault if used at all; `skrifheim` should store references, issuer
proofs, digests, expiration, and policy metadata rather than raw scans or
photos. AI may assist with document, face, or liveness checks, but it must be
treated as evidence for deterministic policy, never as the sole unlock
authority.

Approval language must map to executable local roles before it appears in
break-glass, key lifecycle, law-pack admission, declassification, backup, or
release workflows. `v0.26.2` defines owner, maintainer, security officer, legal
reviewer, key guardian, emergency approver, auditor, and service roles, plus a
single-maintainer fallback that still records explicit approvals and evidence.

API authentication needs credential and session lifecycle rules, not only a
transport identity. Tokens, passkeys, service credentials, emergency
capabilities, and future extension credentials must bind tenant, principal,
device, workload, purpose, policy epoch, scope, expiry, replay nonce, and
revocation epoch.

Before key hierarchy work proceeds beyond metadata, the policy planner must
close known scaffold timing leaks: policy-token comparison needs admitted
constant-time evidence, policy-token storage must use fixed-slot authorization
sets, and label evaluation must avoid exposing compartment or releasability set
size through variable loop shape.

The current fixed string-slot token sets are safe scaffolding but poor
long-term hot-path representation. Before query planning becomes performance
critical, canonical policy strings should move to catalog boundaries while
authorization hot paths use catalog-assigned compact IDs or keyed fixed-width
tags with constant-shape set operations and timing evidence.

Future causal-DAG, query, and policy traversals must avoid recursive call paths
that clone or stack-nest `AuthorityContext`, `SecurityLabel`, or policy-token
sets. These structures intentionally carry fixed-size token slots for
constant-shape evaluation; traversal code should use iterative work queues or
explicit heap ownership when evaluating many graph hops.

Query execution should become batch-oriented before serious execution work:
selection vectors, bounded columnar intermediate batches, arenas, bounded
spilling, and a cost model that includes I/O, CPU, memory, policy evaluation,
leakage, projection freshness, and legal/compliance checks. Optional JIT stays
deferred until profiling proves it is worth the extra attack surface and an
interpreter fallback remains mandatory.

Vector search is a rebuildable projection, not canonical truth. The planned
shape is a policy-partitioned mutable HNSW or flat-search delta for fresh
writes plus immutable disk ANN snapshots for scale, manifest-bound watermarks,
snapshot visibility, embedding model/version/provenance fields, recall
regression fixtures, and stale-policy/deletion/cross-domain leakage tests.

## Phase 5: Cryptographic Control Plane

Build crypto-agile metadata before hard-coding any final algorithm:

- algorithm registry,
- signature envelopes,
- bounded signature-set size,
- key hierarchy from root trust to deployment, region, tenant, compartment, and segment/data keys,
- tenant key metadata explicitly bound to deployment and region scope,
- key lifecycle states for creation, activation, rotation, expiration, compromise, quarantine, and destruction,
- encryption domains for tenant, region, classification, compartment, world, WAL, segment, projection, backup, export capsule, AI artifact, WASM/plugin secret, and audit log,
- per-compartment key epochs,
- per-segment encryption epochs,
- query-result classification rules,
- encrypted index and projection boundaries,
- memory secrecy rules for key material and secret buffers,
- audit-log protection metadata and actor attribution for dangerous
  control-plane operations,
- threshold approval records,
- threshold-signature or bounded quorum multi-signature proof model,
- signed manifests,
- encrypted and signed audit logs,
- recovery and compromise playbooks,
- signed declassification proofs for every write-down across a classification
  or release boundary,
- capability-scoped AI write credentials with classification ceilings and
  derivation-cone identifiers,
- offline verification.

Post-quantum readiness is a metadata and migration requirement from day one.

Fact and signature validation must stay bounded before durable ingest exists:
fact-builder deduplication should use O(n log n) behavior, and signature sets
must cap the number of envelopes, signature key identifiers, and individual
signature payload sizes. Named or hybrid algorithm identifiers must be checked
against a closed approved list before any verifier dispatch is allowed.

See [Encryption Architecture](encryption-architecture.md).

## Phase 6: Compliance, Legal, And Sovereign Placement

Build the policy foundations that let a standalone database understand legal
and compliance constraints before reading, writing, exporting, backing up,
indexing, or deriving data. A future cluster extends the same model before
moving data between nodes or jurisdictions.

Required models:

- node passports for jurisdiction, operator, approvals, crypto profile, and law-pack versions,
- data passports for origin, classification, data category, purpose, retention, export, AI, search, vector, backup, and transfer policy,
- operation passports for replication, query, backup, restore, indexing, embedding, export, failover, and tunnel creation,
- signed law and compliance pack metadata with review status, test cases, validity windows, and rollback rules,
- legal operation and transfer decisions that can allow, constrain, require approval, or deny,
- legal-basis proof skeletons for planner output,
- local compliance-law veto model.

The database must not invent law. It consumes signed, reviewed, versioned packs
and turns them into deterministic planning inputs.

Law and policy packs must not become unbounded scripts. Evaluation must be
deterministic, resource-bounded, side-effect-free, and independent of network,
filesystem, random, wall-clock, or process state. Declarative data is preferred
over executable policy code; any executable helper needs an explicit sandbox or
compile-boundary admission.

See [Hyve Cluster And Compliance Roadmap](hyve-cluster-and-compliance-roadmap.md).

## Phase 7: Projection Compiler

Canonical facts drive projections:

- graph adjacency,
- full-text search,
- vector indexes,
- columnar analytics,
- realtime subscriptions,
- publishing extension render graphs,
- relationship/feed extension timelines and notification feeds,
- privacy-preserving counters,
- media authorization indexes,
- moderation and safety-label indexes,
- hierarchical discussion indexes,
- read-state and watch/subscription indexes,
- AI artifacts and context packs.

Every projection records its source fact range, consistency level, policy
boundary, source watermark, committed manifest generation, and rebuild command.
Materialized projections should be incremental where possible, using source
watermarks tied to committed manifest generations.

Change streams expose committed-generation events and redacted operation
categories, not raw internal state. Point-in-time recovery and deterministic
replay tracing must use the same manifest-generation, WAL commit-root,
audit-root, and world-revision evidence instead of inventing a separate history
mechanism.

Projection metadata must also support taint propagation. If a source fact,
worker, model, or key is compromised, `skrifheim` must be able to find the
downstream projection and artifact cone and mark it stale, quarantine it into a
separate world, or make it eligible for crypto-erasure where the key hierarchy
allows that response.

High-volume relationship-feed workloads are a first-class target alongside
publishing and source-state hosting, but they must be implemented as optional
extension crates unless a primitive is truly generic. The database core must
provide reusable fact, world, policy, encryption, projection, and audit
primitives; an extension crate such as `skrifheim-ext-social` composes those
primitives for a product family without making that product family part of the
mandatory core.

Those extension crates must express their needs through generic primitives:

- viewer-context visibility planning for public, limited-audience,
  member-scoped, group-scoped, deleted, tombstoned, and visibility-reduced
  content,
- relationship policy edges for subscription, membership, denial, preference,
  interaction, reference, endorsement, collection, and report-like signals,
- timeline projection metadata for pull feeds, materialized hot timelines,
  lazy fanout, notification timelines, and deterministic rebuild from
  canonical facts/events,
- privacy-preserving counter and analytics models that do not expose per-viewer
  identities unless a product feature has explicit consent and legal basis,
- media metadata records for encrypted object references, variants, captions,
  takedowns, short-lived access grants, and processing states while keeping raw
  media blobs outside the database,
- moderation, appeal, trusted-flagger, statement-of-reasons, and stackable
  safety-label records that are policy inputs for search, feeds, media,
  notifications, and counters,
- consent ledger, ranking explanation, ad-transparency, and ad-measurement
  metadata that can support EU-style non-personalized fallback and DSA-style
  explanations,
- E2EE private-channel metadata boundaries where the server stores ciphertext
  and minimal routing metadata only,
- server-blind collaboration envelope boundaries where workspaces, channels,
  memberships, invitations, device/key epochs, read cursors, delivery state,
  legal hold, retention, exports, plugins, and integration metadata can be
  represented without admitting plaintext, search terms, file names,
  notification bodies, push-token material, client group secrets, or attachment
  keys into canonical server-side records,
- mailbox-style encrypted-at-rest messaging boundaries where server-authorized
  search, folders, support threads, and abuse handling require explicit
  decrypt/index proofs, purpose limitation, key epochs, and audit binding.

Core crates must not depend on those extension crates. A deployment that only
needs the base world database must be able to omit publishing, social,
messaging, mailbox, forge, forum, or collaboration extension crates from its
trusted runtime.

Hierarchical discussion workloads are also a first-class shape. The database
must provide generic, policy-bound primitives for nested spaces, long-lived
threads, rich moderation, and safe extensibility without owning an
application-owned discussion schema:

- hierarchical containers with scoped capabilities, inheritance,
  trust-level hooks, ownership checks, temporary grants, and escalation
  prevention,
- content lifecycle and sanitized-content provenance that records source body,
  rendered-body digest, sanitizer/renderer policy version, edit revision, and
  actor attribution without making rendered HTML or template output a canonical
  trust root,
- read/unread and watch/subscription projections that rebuild from canonical
  facts and re-check policy before notifications or counters,
- moderation workflow records for queues, approvals, warning/point systems,
  reusable action bundles, delayed jobs, replay/simulation, workload routing,
  and immutable previous/new-state audit,
- extension, plugin, and theme capability boundaries where manifests, host
  grants, hook invocations, rendering metadata, CSP/security policy versions,
  and simulator results are facts under `skrifheim` policy,
- import and migration planning through dry-run reports, permission-gap
  reporting, quarantine worlds, idempotent checkpoints, and schema/legal/policy
  compatibility proofs.

## Phase 8: Query Language And Context Packs

The native query model is world-aware and causal:

```text
world production
assume Service("auth-cluster-se").status = "offline"
simulate consequences depth 5
return context_pack {
    token_budget = 5000
    include evidence
    include confidence
    include policy_proof
    redact above subject_clearance
}
```

The result must include provenance, source facts, graph paths, redactions, stale artifact markers, and policy proof.

Simulation query isolation must be decided before the native query AST is
frozen. `QueryIntent::SimulateConsequences` and `WorldKind::Simulation` do not
by themselves decide whether a simulation creates a writable temporary world
fork or uses read-only counterfactual reasoning inside the query planner. The
first model behaves like a short-lived branch with cleanup, transaction, and
audit requirements. The second model behaves like a proof-producing read-only
overlay. `v0.24.1` is reserved to choose this architecture before `v0.25.0`
defines simulation AST nodes.

Confidence is not just a stored scalar in the target model. Derived facts and
query results should carry computed confidence from their evidence and
caused-by chains, weighted by source reliability and fused with mandatory
access control so policy can express rules such as redacting high-classification
facts below an accepted confidence threshold.

The confidence propagation math is a release-blocking design choice before
query execution. The project must choose and test an explicit model before
`v0.28.0`, with `v0.23.1` reserved for that decision. The initial candidate
models are:

- min-chain: derived confidence is limited by the weakest dependency,
- decay: confidence degrades by causal depth and source reliability,
- product: dependencies combine multiplicatively on the `0..=1000` scale,
- Dempster-Shafer style evidence fusion for a more formal but heavier model.

The default recommendation is to prototype a bounded integer decay model first
because it handles long causal chains without the abrupt behavior of min-chain
or the rapid collapse of naive product multiplication.

## Distinctive Security And Truth Capabilities

These capabilities are expected to make `skrifheim` stand out once the durable
engine, policy planner, and key hierarchy are in place:

- Blast-radius invalidation: use the causal DAG to walk forward from poisoned,
  revoked, invalidated, or compromised inputs and identify every downstream
  fact, decision, projection, release, and AI artifact that is epistemically
  tainted.
- Provenance-bearing declassification: write-downs across classification or
  release boundaries require signed declassification proofs with actor,
  authority, legal/policy basis, reason, source facts, target label, and policy
  epoch. The proof becomes part of the resulting fact or export signature chain.
- Capability-scoped AI derivation cones: AI workers receive bounded write
  capabilities, including classification ceilings, policy epochs, allowed
  worlds, and derivation-cone IDs. A poisoned model or worker must be traceable
  to its output cone, and that cone must support quarantine or crypto-erasure
  through key destruction where the data model permits it.
- Propagated confidence with mandatory access control: confidence for derived
  state is computed from evidence, source reliability, and causal dependencies,
  then evaluated together with mandatory access control. Low-confidence
  sensitive results can be redacted or rejected even when the requester has
  clearance.

## Phase 9: Local-First Core And Extension Compatibility

Support offline and disconnected use in the core world database while keeping
publishing, forum-style, messenger-style, forge-style, source-state, and other
application-family behavior in optional post-1.0 extension crates:

- draft worlds,
- extension-compatible metadata for future collaborative text models,
- policy-filtered changefeeds,
- encrypted sync cursors,
- device-bound local replicas,
- conflict review,
- proof-carrying source-state bundles,
- operation and event ledgers for application workflows,
- auditable explanation objects,
- bounded context packs.

Operational application deployments need secure bootstrap and public-origin
metadata primitives, but `skrifheim` must keep them generic and policy-bound:

- first-run bootstrap state with one-time token proofs, setup fingerprints,
  public-origin validation, trusted-proxy/header policy, migration readiness,
  key-hierarchy readiness, and automatic lockout after completion,
- first-owner or first-administrator challenge metadata that binds to origin,
  device context, policy epoch, expiry, and single-use consumption without
  storing setup secrets as ordinary facts or audit payloads,
- site/instance identity settings, public origins, public aliases, descriptor
  metadata, private/maintenance mode, and search visibility as audited facts
  under `skrifheim` policy,
- strict separation between public serving aliases and administrative,
  passkey/WebAuthn, bootstrap, API, and trusted internal origins,
- scheduled operations such as publish-due, descriptor rebuilds, cache purge or
  warm actions, and maintenance tasks as private authenticated operations with
  replay guards and audit binding,
- read-only configuration export that helps recovery while explicitly excluding
  credentials, sessions, bootstrap tokens, recovery codes, key material, and
  other secrets.

The collaborative text model is a post-1.0 extension-track decision before
optional publishing, forum, messenger, or collaboration crates depend on it.
When selected, it must define operation/state representation, actor and device
identity, causal clocks, tombstones, compaction, policy boundaries, and whether
releases store materialized text, operation history, compacted state, or signed
projections. Publishing extensions use world promotion, not mutable published
flags.

Source-state and compliance-forge hosting are first-class optional extension
shapes alongside publishing, forum, messenger, and collaboration support.
`skrifheim` should not own a consuming project's source object format, diff
algorithm, or local CLI semantics, but it must provide durable backend
primitives for policy-bound hosted source workflows:

- application object identity domains that include object type, algorithm tag,
  canonical format version, tenant, and world scope,
- immutable source-state records for objects, state roots, changes, revisions,
  proof reports, bundles, releases, and operation records,
- mutable aliases that point human names such as project heads, review worlds,
  and release channels to immutable state through transactions,
- proof-carrying bundle manifests and remote decision facts that allow accept,
  deny, quarantine, or more-evidence-required outcomes,
- resource-budgeted verification modes: bounded-batch, lazy-cone, and
  full-world, with explicit memory, graph, body-size, and parallelism budgets,
- append-only operation and event records where events are observations, not
  authoritative facts, until a deterministic fact compiler admits them,
- explanation objects and context packs that cite facts, objects, policy
  decisions, redactions, missing evidence, confidence, and optional AI use,
- sealed private realm support with public ciphertext storage IDs, private
  keyed plaintext IDs, protected metadata policy, recipient slots, blind remote
  storage, split-trust metadata, and leak-scan result facts.

Consuming applications own their object formats and workflows. The database
must provide generic, policy-aware, encrypted, tenant-scoped primitives so
hosted workflows can build on `skrifheim` without weakening local-first
verification, legal/compliance controls, or provenance.

## Phase 10: Rootless Podman And Production Runtime

`skrifheim` must run:

- as a compiled host binary,
- inside rootless Podman,
- with portable file I/O on all supported OS families,
- with Linux io_uring/direct-I/O only as an optional fast path.

The production baseline must support Linux, Windows, macOS, and BSD without
changing database semantics. Android and iOS remain future targets when their
sandbox, storage, and attestation models can provide equivalent guarantees.
Durable formats must be architecture-neutral and must not assume x86, native
endianness, pointer width, alignment, or page size. x86_64 and AArch64 are
first-pass production CPU targets; RISC-V and other targets must remain
possible through portable core code and optional host adapters.

## Phase 11: Hyve Cluster Fabric

Build cluster features only after the single-node database is production-ready
and the first optional extension tracks have proven the public API. The cluster
roadmap starts at `v2.0.0`.

Planned components:

- sovereign cells with local consensus and local failover,
- Hyve control plane for topology, placement, health, policy epochs, key epochs, and law-pack epochs,
- automatic encrypted tunnels with workload identity and policy-scoped streams,
- signed peer maps and node passports,
- placement planner for worlds, projections, indexes, backups, AI workers, and public releases,
- legal operation and transfer handshake before replication, query, indexing, backup, export, AI processing, or failover,
- compliance-aware failover that can promote, seal, restrict, or deny per world and data class,
- witness/notary roles for hash-only proof replication,
- compliance autopilot that detects drift and proposes lawful remediation.

The control plane may propose movement. Local compliance-law kernels can veto it.

## Test Strategy

Required layers:

- unit tests for every crate,
- property tests for policy, conflict, and storage invariants once dependencies are admitted,
- fuzz tests for parsers and storage frames,
- crash-recovery matrix,
- corruption matrix,
- deterministic simulation tests,
- concurrency model tests,
- container smoke tests,
- release-gate security review.

No release is tag-ready without tests and a passing security review for that exact commit.
