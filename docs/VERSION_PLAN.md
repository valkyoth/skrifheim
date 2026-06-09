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
- `scripts/release_0_1_gate.sh`
- `cargo test --workspace`

## v0.2.0 - Core Fact Builder

Goal: make canonical facts constructible through validated APIs.

Deliverables:

- fact builder,
- required evidence checks,
- valid-time checks,
- signature-set checks,
- confidence clamping,
- negative tests for invalid facts.

## v0.3.0 - Security Labels And Authority Context

Goal: make classification, compartments, releasability, and subject context explicit.

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
- parent pointers,
- added and hidden fact sets,
- deterministic world identity rules,
- tests for branch isolation.

## v0.6.0 - World Diff And Promotion Preflight

Goal: compare worlds safely before merge or promotion.

Deliverables:

- deterministic world diff,
- conflict categories,
- promotion preflight,
- rollback preflight,
- tests for conflicting fact replacement.

## v0.7.0 - WAL Frame Format

Goal: define and validate append-only WAL frames before persistence logic.

Deliverables:

- WAL frame header,
- record kind model,
- length and checksum fields,
- frame validation,
- parser tests for malformed frames.

## v0.8.0 - WAL Writer And Reader

Goal: write and read WAL frames through portable file I/O.

Deliverables:

- append-only writer,
- sequential reader,
- fsync boundary option,
- partial-write detection,
- tests using temporary host files.

## v0.9.0 - WAL Replay And Recovery States

Goal: recover committed state and reject ambiguous corruption.

Deliverables:

- replay state machine,
- committed/uncommitted transaction handling,
- truncated-frame handling,
- recovery report,
- crash matrix tests for WAL-only storage.

## v0.10.0 - Immutable Segment Format

Goal: define policy-scoped immutable fact segments.

Deliverables:

- segment header,
- footer,
- content hash field,
- policy and encryption metadata,
- segment validation tests.

## v0.11.0 - Segment Writer And Reader

Goal: persist and read immutable segments without compaction.

Deliverables:

- segment writer,
- segment reader,
- checksum verification,
- content-hash verification,
- corruption rejection tests.

## v0.12.0 - Manifest And Checkpoint Format

Goal: record the durable storage root.

Deliverables:

- manifest structure,
- checkpoint LSN,
- segment list,
- policy epoch field,
- manifest validation tests.

## v0.13.0 - Startup Recovery Integration

Goal: combine manifests, segments, and WAL replay at startup.

Deliverables:

- recovery loader,
- manifest selection,
- WAL replay from checkpoint,
- corrupted manifest rejection,
- deterministic recovery fixtures.

## v0.14.0 - In-Memory Transaction Model

Goal: model read sets, write sets, predicate sets, and commit timestamps.

Deliverables:

- transaction state type,
- read/write/predicate set tracking,
- conflict model,
- commit timestamp allocation,
- deterministic unit tests.

## v0.15.0 - Strict Serializable Validation

Goal: enforce strict serializable single-node transaction rules in memory.

Deliverables:

- read/write conflict validation,
- predicate conflict validation,
- write-write conflict validation,
- abort reasons,
- deterministic concurrency tests.

## v0.16.0 - Durable Transaction Commit

Goal: connect transaction validation to WAL and recovery.

Deliverables:

- prepare and commit records,
- durable commit boundary,
- replay of committed transactions,
- rollback of uncommitted transactions,
- crash tests around prepare/commit.

## v0.17.0 - Fact Index And Snapshot Reads

Goal: read facts by world and snapshot from recovered state.

Deliverables:

- in-memory fact index,
- snapshot timestamp visibility,
- supersession and invalidation lookup,
- fact history tests,
- unauthorized stale-read tests.

## v0.18.0 - Native Query AST

Goal: define the first native query representation without execution.

Deliverables:

- world read AST,
- fact filter AST,
- causality explain AST,
- simulation query AST skeleton,
- AST validation tests.

## v0.19.0 - Native Query Parser

Goal: parse the minimal native query language.

Deliverables:

- parser,
- source spans,
- structured parse errors,
- parser fixtures,
- fuzz seed corpus.

## v0.20.0 - Policy-Aware Query Planning

Goal: convert query AST into a policy-checked plan.

Deliverables:

- logical plan,
- security checks before execution,
- rejection and redaction reports,
- policy proof skeleton,
- tests for denied plans.

## v0.21.0 - Query Execution Prototype

Goal: execute read-only fact and causality queries on a single node.

Deliverables:

- fact scan execution,
- point lookup execution,
- causality edge traversal over fact links,
- bounded result sets,
- tests for authorized and denied reads.

## v0.22.0 - Projection Registry

Goal: register rebuildable projections without implementing every projection type.

Deliverables:

- projection metadata,
- source fact range,
- consistency mode,
- watermark tracking,
- rebuild command skeleton.

## v0.23.0 - Graph Projection

Goal: build the first projection from causal facts.

Deliverables:

- adjacency projection,
- source range tracking,
- rebuild from canonical facts,
- stale projection detection,
- graph traversal tests.

## v0.24.0 - Search Projection Skeleton

Goal: create a policy-aware full-text/search projection boundary.

Deliverables:

- tokenizer boundary,
- source fact visibility checks,
- projection watermark,
- rebuild tests,
- no cross-compartment mixing tests.

## v0.25.0 - Crypto-Agile Manifest Signatures

Goal: sign manifests without locking the project to one permanent algorithm.

Deliverables:

- algorithm registry,
- signature envelopes,
- key epoch metadata,
- manifest signature validation API,
- rejected unknown-algorithm tests.

## v0.26.0 - Audit Proof Queries

Goal: prove what was known, under which policy, and from which manifest.

Deliverables:

- fact existence proof skeleton,
- policy epoch proof skeleton,
- manifest root reference,
- audit query output type,
- tests for missing proof material.

## v0.27.0 - Backup And Restore Skeleton

Goal: export and import signed storage roots.

Deliverables:

- backup manifest,
- restore preflight,
- content hash verification,
- policy epoch verification,
- restore rejection tests.

## v0.28.0 - Rootless Podman Runtime

Goal: run `skrifheim` as a rootless container and a compiled host binary.

Deliverables:

- Containerfile,
- rootless smoke script,
- persistent volume layout,
- portable path policy,
- container release gate.

## v0.29.0 - Configuration And Admin CLI

Goal: make local operation explicit and testable.

Deliverables:

- config file format,
- config validator,
- admin CLI skeleton,
- diagnostics command,
- invalid config tests.

## v0.30.0 - CMS World And Release Primitives

Goal: support the first CMS-style atomic publishing model.

Deliverables:

- public/private world split,
- release object,
- publish preflight,
- atomic promote/rollback,
- tests for no half-published state.

## v0.31.0 - CMS Render Dependency Graph

Goal: track causal dependencies for rendered public output.

Deliverables:

- route render graph model,
- content dependency edges,
- invalidation calculation,
- public projection boundary,
- tests for precise invalidation.

## v0.32.0 - AI Artifact Provenance

Goal: store AI output as untrusted derived artifacts with provenance.

Deliverables:

- source fact lineage,
- model and prompt hash metadata,
- artifact invalidation,
- human promotion workflow,
- tests that AI artifacts are not authoritative facts.

## v0.33.0 - Local-First World Metadata

Goal: add policy-filtered local worlds and sync cursors.

Deliverables:

- device-bound world metadata,
- sync cursor model,
- encrypted sync envelope metadata,
- CRDT field metadata for future CMS editor support,
- policy-filtered sync tests.

## v0.34.0 - Mission Capsule And Cross-Domain Export Skeleton

Goal: make explicit export/import boundaries for lower-side or disconnected use.

Deliverables:

- mission capsule metadata,
- expiration and device-binding fields,
- export policy proof skeleton,
- import verification preflight,
- rejected downgrade tests.

## v0.35.0 - Fuzz And Property Test Baseline

Goal: expand verification before production hardening.

Deliverables:

- parser fuzz target,
- storage frame fuzz target,
- transaction property tests,
- policy decision property tests,
- CI wiring for non-flaky local runs.

## v0.36.0 - Production Hardening Candidate

Goal: make the single-node engine ready for release-candidate review.

Deliverables:

- no known release-blocking panics,
- recovery runbook,
- backup/restore runbook,
- security control evidence update,
- known-limits review.

## v0.37.0 - 1.0 Release Candidate

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
