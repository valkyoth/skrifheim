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
7. A permanent report is written at `security/pentest/<tag>.md` only when the
   exact commit is ready to tag and the result is `Status: PASS`.
8. Tagging and pushing tags happen only when explicitly requested.

Root `PENTEST.md` is temporary scratch input. It must not be committed, and the
release metadata validator fails while it exists.

## v0.1.0 - Repository Foundation

Goal: initialize the serious Rust workspace and policy baseline.

Deliverables:

- Rust stable `1.96.0` pinned.
- Focused workspace crates.
- `scripts/checks.sh`.
- CI, dependency policy, security policy, release notes.
- Implementation, version, modularity, threat-model, toolchain, and CMS target docs.

Verification:

- `scripts/checks.sh`
- `cargo test --workspace`

## v0.2.0 - Fact Model And Policy Labels

Goal: make canonical facts executable and well-tested.

Deliverables:

- fact builder,
- validation errors,
- classification labels,
- evidence requirements,
- signature envelope requirements,
- negative tests for missing evidence, bad time ranges, and missing signatures.

## v0.3.0 - World Branch Engine

Goal: implement world create, fork, diff, and promote for in-memory fact sets.

Deliverables:

- world DAG metadata,
- merge preflight,
- conflict classes,
- deterministic world diff output,
- tests for branch isolation and rollback.

## v0.4.0 - WAL And Recovery Skeleton

Goal: add durable append-only WAL frames and startup recovery.

Deliverables:

- WAL record headers,
- checksum validation,
- recovery replay,
- committed/uncommitted transaction handling,
- crash matrix tests.

## v0.5.0 - Immutable Segment Storage

Goal: flush facts into immutable policy-scoped segments.

Deliverables:

- segment format,
- manifest format,
- content hashes,
- corruption rejection,
- segment reader/writer tests.

## v0.6.0 - Strict Transaction Kernel

Goal: strict serializable single-node transactions.

Deliverables:

- read/write sets,
- predicate set model,
- conflict validation,
- commit timestamp ordering,
- deterministic concurrency tests.

## v0.7.0 - Policy-Aware Query Planner

Goal: enforce classification, compartments, releasability, and output labels before execution.

Deliverables:

- subject/device/workload context,
- planner rejection model,
- redaction model,
- constant-shape denial tests.

## v0.8.0 - Native Query Prototype

Goal: parse and plan minimal world queries.

Deliverables:

- query AST,
- `world ... at ...` reads,
- causality explain query,
- policy proof stub,
- parser fuzz seed corpus.

## v0.9.0 - Projection Engine

Goal: rebuildable projections from canonical facts.

Deliverables:

- projection registry,
- source range tracking,
- graph adjacency projection,
- full-text placeholder projection,
- rebuild tests.

## v0.10.0 - CMS Release Primitive

Goal: support the first CMS-style atomic publishing model.

Deliverables:

- public/private world split,
- release object,
- route render graph model,
- dependency graph,
- atomic promote/rollback tests.

## v0.11.0 - Rootless Podman Runtime

Goal: run `skrifheim` as a rootless container and a compiled host binary.

Deliverables:

- Containerfile,
- rootless smoke script,
- persistent volume layout,
- portable path policy.

## v0.12.0 - Crypto Control Plane Metadata

Goal: implement crypto-agile manifests and key epochs without locking to one algorithm.

Deliverables:

- algorithm registry,
- manifest signatures,
- key epoch transitions,
- offline verification CLI stub.

## v0.13.0 - AI Artifact Model

Goal: store AI output as untrusted derived artifacts with provenance.

Deliverables:

- source fact lineage,
- model and prompt hash metadata,
- artifact invalidation,
- human promotion workflow.

## v0.14.0 - Local-First Worlds

Goal: add policy-filtered local worlds and sync cursors.

Deliverables:

- device-bound world metadata,
- encrypted sync envelope metadata,
- CRDT field model for future CMS editor support.

## v0.15.0 - Backup, Restore, And Audit Proofs

Goal: prove and restore history.

Deliverables:

- signed backup manifests,
- restore verification,
- audit proof queries,
- rollback detection tests.

## v1.0.0 - Production World Database For CMS Integration

Goal: first serious production-ready `skrifheim`.

Deliverables:

- durable single-node fact/world engine,
- strict transaction semantics,
- policy-aware query planning,
- tamper-evident manifests,
- rootless Podman deployment,
- backup/restore,
- CMS release primitives,
- public/private world split,
- render dependency tracking,
- AI artifact provenance,
- complete release runbook,
- security review PASS for exact commit.

Non-goals for 1.0:

- replacing every SQL database,
- distributed consensus as default,
- unsandboxed plugins,
- AI as authoritative truth,
- exotic hardware requirement.
