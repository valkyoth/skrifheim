# skrifheim Hyve Cluster And Compliance Roadmap

Status: planned architecture

Database name: `skrifheim`

This document records the cluster and compliance direction for `skrifheim`.
The core idea is that topology, jurisdiction, compliance posture, and legal
movement rules become database planning inputs, not external deployment notes.

## Design Position

The cluster model is a set of sovereign cells coordinated by a control plane.
The control plane can propose placement, tunnels, failover, and repair, but it
does not sit on the hot data path for every user write.

The legal and compliance model is stricter: a node may not move, replicate,
index, summarize, back up, fail over, or export data until the operation passes
the relevant local and destination policy checks.

Important boundaries:

- human-approved law and compliance packs are inputs,
- the database does not invent legal interpretation,
- AI may help draft explanations but must not become legal authority,
- every legal interpretation change is versioned, signed, reviewed, and tested,
- a local compliance kernel can veto a control-plane placement decision.

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

Every node that participates in non-public data movement eventually needs a
local compliance-law kernel.

Responsibilities:

- resolve jurisdiction and applicable policy packs,
- evaluate node passports, data passports, and operation passports,
- enforce classification and data-category rules,
- evaluate legal basis, purpose limitation, retention, and export rules,
- detect legal or policy conflicts,
- produce signed allow, constrained-allow, approval-required, or deny decisions,
- explain decisions without leaking protected facts,
- log immutable audit events for every security-relevant decision.

If the local compliance-law kernel is unavailable, the node may continue
serving already-approved local reads according to its last safe state, but it
must not accept new cross-border movement, new sensitive replication, or new
legal interpretation changes.

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

## Legal Transfer Handshake

Before data crosses a node, cell, region, jurisdiction, tunnel, projection,
AI worker, backup target, or failover boundary, `skrifheim` must evaluate a
legal transfer handshake.

The handshake checks:

- source node passport,
- destination node passport,
- data passport,
- operation passport,
- source law and policy packs,
- destination law and policy packs,
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
- can this data leave this node,
- can derived data leave this node,
- can this index be built in that jurisdiction,
- can this AI worker process this data,
- can this backup be stored there,
- can this failover happen automatically,
- can this query result be shown to this actor,
- what classification and legal basis does the output inherit.

Only after legal approval can performance planning run.

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

The 1.0 release must include the foundations that make legal and compliance
awareness possible: passports, law-pack metadata, legal planner skeletons,
placement intent, and legal-basis proofs.

Full multi-cell clustering, automatic tunnel management, multi-region failover,
and compliance autopilot are post-1.0 roadmap items unless their prerequisites
are deliberately pulled forward.
