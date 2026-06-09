# skrifheim Hyve Cluster And Compliance Roadmap

Status: planned architecture

Database name: `skrifheim`

This document records the standalone compliance and future cluster direction
for `skrifheim`. The core idea is that jurisdiction, compliance posture, legal
processing rules, topology, and movement rules become database planning inputs,
not external deployment notes.

## Design Position

Compliance-aware planning is a core database capability. It must work in a
single standalone `skrifheim` instance before any multi-node cluster exists.

The standalone model decides whether local reads, writes, CMS document access,
exports, backups, search indexes, vector embeddings, AI summaries, admin
queries, and public projections are lawful for the actor, purpose, data class,
request context, and configured policy packs.

The cluster model extends the same planner to a set of sovereign cells
coordinated by a control plane. The control plane can propose placement,
tunnels, failover, and repair, but it does not sit on the hot data path for
every user write.

The legal and compliance model is stricter than normal access control: an
operation may not read, reveal, derive, move, replicate, index, summarize,
back up, fail over, or export protected data until the operation passes the
relevant local policy checks. In a cluster, destination checks are added.

Important boundaries:

- human-approved law and compliance packs are inputs,
- the database does not invent legal interpretation,
- AI may help draft explanations but must not become legal authority,
- every legal interpretation change is versioned, signed, reviewed, and tested,
- a local compliance kernel can veto a local query, export, AI/index operation,
  backup operation, or control-plane placement decision.

## Standalone Compliance Is Core

A standalone database in Sweden, Norway, Germany, France, or any other
jurisdiction must carry the same compliance-aware planning model as a future
cluster node.

Examples:

- a CMS public gateway asks for a private draft as public content: deny,
- a request context is outside the approved region for an EEA-only tenant
  policy: deny, redact, or require stronger proof according to the policy pack,
- an AI worker asks to summarize personal data without an allowed purpose:
  deny,
- a search projection tries to index data above its legal or classification
  domain: deny,
- an admin export asks for personal data without the required legal basis:
  deny or require approval,
- a backup target is outside the allowed region or contract scope: deny.

Network origin, IP country, device posture, actor identity, tenant policy,
purpose, lawful basis, consent state, retention state, and data category are
policy signals. They are not individually sufficient as legal truth, but the
planner must be able to combine them into a deterministic decision.

The safe default is that unlabeled non-public data cannot be read outside its
declared local purpose, cannot be exported, cannot be indexed, cannot be
processed by AI, and cannot be backed up to a new boundary.

## Sovereign Cells

A cell is the local operational unit. It can be a single machine in early
development, then a local cluster later.

Planned cell roles:

- gateway nodes,
- transaction/shard nodes,
- storage nodes,
- projection and index workers,
- AI artifact workers,
- replication workers,
- local key guardians,
- audit-log writers,
- compliance-law kernels.

The first production database target stays single-node. Multi-node local cells
and multi-region cells are post-1.0 work unless explicitly moved earlier.

## Hyve Control Plane

The Hyve control plane stores and reconciles operational intent:

- node and cell registry,
- region and jurisdiction labels,
- world and tenant placement intent,
- projection and index placement intent,
- tunnel topology,
- health and lag observations,
- failover eligibility,
- key and policy epochs,
- law-pack epochs,
- audit proof references.

The control plane may propose:

- create a secondary replica,
- move a projection closer to readers,
- open or close an encrypted tunnel,
- quarantine a node,
- promote a hot secondary,
- rebuild stale search or vector projections,
- rotate affected key epochs,
- pre-warm public release projections,
- stop movement after a compliance drift event.

The control plane must not override a local compliance-law veto.

## Compliance-Law Kernel

Every standalone instance and every future cluster node that participates in
non-public access or movement needs a local compliance-law kernel.

Responsibilities:

- resolve jurisdiction and applicable policy packs,
- evaluate local instance/node passports, data passports, operation passports,
  actor context, request context, and workload context,
- enforce classification and data-category rules,
- evaluate legal basis, purpose limitation, retention, and export rules,
- detect legal or policy conflicts,
- produce signed allow, constrained-allow, approval-required, or deny decisions,
- explain decisions without leaking protected facts,
- log immutable audit events for every security-relevant decision.

If the local compliance-law kernel is unavailable, the instance may continue
serving already-approved local reads according to its last safe state, but it
must not accept new sensitive reads, exports, AI/search/index operations,
backups, cross-boundary movement, sensitive replication, or legal
interpretation changes.

## Passports

### Node Passport

A node passport states what the node claims and proves about itself:

- node and cell identity,
- physical country and jurisdiction,
- operating legal entity,
- ownership and provider profile,
- certifications and authority approvals,
- allowed classifications and data categories,
- allowed processing modes,
- hardware or runtime attestation references,
- crypto profile,
- law-pack versions,
- policy epoch,
- validity window,
- signature.

A node without a valid passport can only receive public data.

### Data Passport

A data passport follows canonical data and derived data:

- origin country and controller,
- data-subject region when applicable,
- data categories,
- classification and compartments,
- legal basis,
- purpose limits,
- retention rule,
- allowed and denied regions,
- allowed processing modes,
- AI, search, vector, backup, and export policy,
- required approvals,
- transfer rules,
- source law-pack version,
- policy epoch,
- signature.

No unlabeled data may move. Derived state inherits a passport from its sources:
search indexes, vector embeddings, summaries, caches, analytics projections,
logs, and backups are treated as legally relevant data.

### Operation Passport

Every action that can expose, move, derive, or destroy data gets an operation
passport:

- operation identity and type,
- source and destination node,
- actor and workload,
- purpose,
- requested processing mode,
- data-set descriptor,
- approvals,
- emergency or break-glass flag,
- audit context.

Examples include replication, backup, restore, query, join, index, embed,
summarize, train, sync-to-device, export, declassify, failover, move-range,
open-tunnel, and create-projection.

## Law And Compliance Packs

Law and compliance packs are signed, versioned, machine-readable policy inputs.
They may represent statutory law, regulation, sector rules, customer contracts,
organization policy, classification rules, or emergency procedures.

Each pack must include:

- issuer identity,
- jurisdiction or authority scope,
- semantic version,
- validity window,
- ontology and concepts,
- compiled deterministic rules,
- exceptions,
- required approvals,
- evidence requirements,
- test cases,
- signature.

Packs are not accepted only because they parse. Admission requires review,
tests, rollback handling, and signed approval by the correct authority.

## Legal Operation And Transfer Handshake

Before protected data is read, written, derived, indexed, summarized, exported,
backed up, or moved across a node, cell, region, jurisdiction, tunnel,
projection, AI worker, backup target, or failover boundary, `skrifheim` must
evaluate a legal operation handshake. Cross-boundary movement adds the transfer
handshake checks.

The handshake checks:

- local instance or source node passport,
- destination node passport when data crosses a boundary,
- data passport,
- operation passport,
- actor and workload context,
- request context such as source network region and device posture when known,
- source law and policy packs,
- destination law and policy packs when data crosses a boundary,
- tenant and organization policy,
- crypto profile,
- key epoch,
- approvals,
- audit requirements.

Possible decisions:

- allow full movement,
- allow encrypted-only movement,
- allow public projection only,
- allow redacted result only,
- allow aggregate only,
- allow hash-witness only,
- allow remote sealed query only,
- allow compute-to-data only,
- require human or authority approval,
- require declassification,
- deny.

A transfer is allowed only when all required authorities agree. A single
required veto denies the operation.

## Legal Planner

The planning stack eventually becomes:

1. Legal planner.
2. Security planner.
3. Sovereignty planner.
4. Data-minimisation planner.
5. Query planner.
6. Cost planner.
7. Execution planner.

The legal planner answers:

- can this operation legally exist,
- can this actor access this document for this purpose,
- can this request context receive this result,
- can this data leave this node,
- can derived data leave this node,
- can this index be built in that jurisdiction,
- can this AI worker process this data,
- can this backup be stored there,
- can this failover happen automatically,
- can this query result be shown to this actor,
- what classification and legal basis does the output inherit.

Only after legal approval can security, query, and performance planning run.

## Compliance-Aware Tunnels

Hyve tunnels are not only encrypted network links. They carry policy scope:

- source and destination node,
- source and destination jurisdiction,
- allowed classifications,
- allowed data categories,
- denied data categories,
- allowed operations,
- crypto profile,
- law-pack epoch,
- policy epoch,
- audit requirement,
- expiry.

Replication or query streams must carry operation identity, data passport hash,
transfer decision identity, and policy epoch. The tunnel refuses streams outside
its allowed scope.

## Failover With Law Awareness

Failover is per world, data class, and legal basis.

Examples of planned behavior:

- public CMS projections may fail over broadly when export policy permits,
- ordinary personal data may fail over only to approved regional secondaries,
- authentication secrets may require sealed regional mirrors,
- protected infrastructure data may be limited to redacted or hash-witness
  failover,
- classified data may deny automatic cross-border failover entirely.

The control plane must explain which worlds can move, which can become
read-only, which must remain sealed, and which require manual recovery.

## Compute-To-Data

When raw data cannot move, the planner should prefer compute-to-data patterns:

- remote sealed query,
- federated aggregation,
- redacted answer,
- minimum-group-size aggregate,
- hash witness proof,
- zero-row-leak denial shape.

The result inherits an output passport and policy proof.

## Compliance Autopilot

The compliance autopilot reconciles desired legal state with actual state:

- actual node passports,
- actual law-pack versions,
- actual data placement,
- actual tunnel state,
- actual replicas,
- actual key epochs,
- actual projection watermarks,
- actual backup locations.

When drift is detected, planned actions include:

- stop new movement,
- freeze affected tunnel classes,
- seal or quarantine replicas,
- rotate keys,
- mark projections stale,
- rebuild derived data in lawful locations,
- create compliance incident records,
- propose replacement placement.

Autopilot actions that can destroy availability or change legal exposure require
explicit release-gated policies and audit proofs.

## Versioning Intent

The 1.0 release must include standalone legal and compliance foundations:
passports, law-pack metadata, legal planner skeletons, local operation
decisions, placement intent, and legal-basis proofs.

Full multi-cell clustering, automatic tunnel management, multi-region failover,
and compliance autopilot are post-1.0 roadmap items unless their prerequisites
are deliberately pulled forward.
