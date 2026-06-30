# skrifheim Implementation Plan

Status: planning document

Database name: `skrifheim`

1.0 target: a production-ready causal world-state database for a CMS-style application that uses facts, worlds, policies, atomic releases, sanitized public projections, and AI artifacts with provenance.

## Core Position

`skrifheim` stores signed, versioned, policy-bound facts about worlds. It is not a SQL compatibility project and not a generic multi-model database. Relational, document, graph, vector, search, CMS render graphs, AI context packs, and analytics are projections over canonical facts.

The canonical layer must be boring, durable, testable, and auditable. The
futuristic behavior comes from world overlays, causal invalidation,
policy-aware planning, cryptographic declassification, capability-scoped AI
derivation, confidence propagation, and derived projections that can be rebuilt
from truth.

## Non-Negotiable Engineering Rules

- Rust stable `1.96.0`, edition 2024, workspace resolver `3`.
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

## Workspace Shape

- `skrifheim-core`: IDs, timestamps, labels, values, and shared errors.
- `skrifheim-fact`: causal fact records, validation, evidence, confidence, supersession, invalidation.
- `skrifheim-world`: production, draft, simulation, legal/audit, user-local, mission-capsule worlds.
- `skrifheim-policy`: clearance lattice, compartment checks, releasability, planner decisions.
- `skrifheim-crypto`: crypto-agile envelopes, epochs, algorithm identifiers, future threshold signatures.
- `skrifheim-audit`: identities, attestation evidence references, audit events, and audit-log protection metadata.
- `skrifheim-storage`: WAL, immutable segment metadata, Merkle manifests, content-addressed blobs.
- `skrifheim-storage-host`: host-file WAL helpers that use `std` outside the
  `no_std` core crates.
- `skrifheim-query`: query intent, policy-aware planning, context-pack planning.
- `skrifheim-compliance`: future legal/compliance passports, law-pack metadata, and legal transfer decisions.
- `skrifheim-cluster`: future cell, control-plane, tunnel, placement, and failover planning primitives.
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

## Phase 3: Storage Kernel

Implement:

- append-only WAL,
- strict recovery state machine,
- immutable policy-scoped segments,
- manifest snapshots,
- admitted SHA-3/SHAKE digest strength policy before durable storage keys,
- full-width `WorldIdentityDigest`, `ContentDigest`, and `ManifestDigest`
  primitives,
- checksums,
- content-addressed blocks,
- policy and encryption boundary metadata,
- corruption rejection instead of silent repair.

Compaction must preserve tenant, policy, region, encryption, and MVCC boundaries.

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

Before key hierarchy work proceeds beyond metadata, the policy planner must
close known scaffold timing leaks: policy-token comparison needs admitted
constant-time evidence, policy-token storage must use fixed-slot authorization
sets, and label evaluation must avoid exposing compartment or releasability set
size through variable loop shape.

Future causal-DAG, query, and policy traversals must avoid recursive call paths
that clone or stack-nest `AuthorityContext`, `SecurityLabel`, or policy-token
sets. These structures intentionally carry fixed-size token slots for
constant-shape evaluation; traversal code should use iterative work queues or
explicit heap ownership when evaluating many graph hops.

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

See [Hyve Cluster And Compliance Roadmap](hyve-cluster-and-compliance-roadmap.md).

## Phase 7: Projection Compiler

Canonical facts drive projections:

- graph adjacency,
- full-text search,
- vector indexes,
- columnar analytics,
- realtime subscriptions,
- CMS render graphs,
- AI artifacts and context packs.

Every projection records its source fact range, consistency level, policy boundary, and rebuild command.

Projection metadata must also support taint propagation. If a source fact,
worker, model, or key is compromised, `skrifheim` must be able to find the
downstream projection and artifact cone and mark it stale, quarantine it into a
separate world, or make it eligible for crypto-erasure where the key hierarchy
allows that response.

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

## Phase 9: Local-First And CMS Support

Support offline and collaborative worlds:

- draft worlds,
- CRDT fields for collaborative text,
- policy-filtered changefeeds,
- encrypted sync cursors,
- device-bound local replicas,
- conflict review.

CMS publishing uses world promotion, not mutable published flags.

## Phase 10: Rootless Podman And Production Runtime

`skrifheim` must run:

- as a compiled host binary,
- inside rootless Podman,
- with portable file I/O on all supported OS families,
- with Linux io_uring/direct-I/O only as an optional fast path.

## Phase 11: Hyve Cluster Fabric

Build cluster features only after the single-node database and legal planning
foundations are strong enough to test in isolation.

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
