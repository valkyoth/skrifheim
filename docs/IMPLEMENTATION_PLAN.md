# skrifheim Implementation Plan

Status: planning document

Database name: `skrifheim`

1.0 target: a production-ready causal world-state database for a CMS-style application that uses facts, worlds, policies, atomic releases, sanitized public projections, and AI artifacts with provenance.

## Core Position

`skrifheim` stores signed, versioned, policy-bound facts about worlds. It is not a SQL compatibility project and not a generic multi-model database. Relational, document, graph, vector, search, CMS render graphs, AI context packs, and analytics are projections over canonical facts.

The canonical layer must be boring, durable, testable, and auditable. The futuristic behavior comes from world overlays, causal invalidation, policy-aware planning, and derived projections that can be rebuilt from truth.

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

## Workspace Shape

- `skrifheim-core`: IDs, timestamps, labels, values, and shared errors.
- `skrifheim-fact`: causal fact records, validation, evidence, confidence, supersession, invalidation.
- `skrifheim-world`: production, draft, simulation, legal/audit, user-local, mission-capsule worlds.
- `skrifheim-policy`: clearance lattice, compartment checks, releasability, planner decisions.
- `skrifheim-crypto`: crypto-agile envelopes, epochs, algorithm identifiers, future threshold signatures.
- `skrifheim-storage`: WAL, immutable segment metadata, Merkle manifests, content-addressed blobs.
- `skrifheim-query`: query intent, policy-aware planning, context-pack planning.
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

The first correctness question is: can `skrifheim` prove what was true, who asserted it, which policy governed it, and which evidence supported it?

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

## Phase 3: Storage Kernel

Implement:

- append-only WAL,
- strict recovery state machine,
- immutable policy-scoped segments,
- manifest snapshots,
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
- what classification does the output carry.

Rejected plans must be deterministic and constant-shape where practical.

## Phase 5: Cryptographic Control Plane

Build crypto-agile metadata before hard-coding any final algorithm:

- algorithm registry,
- signature envelopes,
- per-compartment key epochs,
- per-segment encryption epochs,
- threshold approval records,
- signed manifests,
- offline verification.

Post-quantum readiness is a metadata and migration requirement from day one.

## Phase 6: Projection Compiler

Canonical facts drive projections:

- graph adjacency,
- full-text search,
- vector indexes,
- columnar analytics,
- realtime subscriptions,
- CMS render graphs,
- AI artifacts and context packs.

Every projection records its source fact range, consistency level, policy boundary, and rebuild command.

## Phase 7: Query Language And Context Packs

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

## Phase 8: Local-First And CMS Support

Support offline and collaborative worlds:

- draft worlds,
- CRDT fields for collaborative text,
- policy-filtered changefeeds,
- encrypted sync cursors,
- device-bound local replicas,
- conflict review.

CMS publishing uses world promotion, not mutable published flags.

## Phase 9: Rootless Podman And Production Runtime

`skrifheim` must run:

- as a compiled host binary,
- inside rootless Podman,
- with portable file I/O on all supported OS families,
- with Linux io_uring/direct-I/O only as an optional fast path.

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
