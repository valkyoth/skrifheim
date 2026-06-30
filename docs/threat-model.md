# skrifheim Threat Model

Status: baseline

`skrifheim` assumes hostile networks, compromised clients, stolen admin credentials, copied disks, poisoned AI workers, malicious plugins, replayed messages, stale replicas, and partial node capture.

## Protected Assets

- canonical facts,
- world history,
- policy epochs,
- crypto epochs,
- classification labels,
- signatures,
- evidence references,
- manifests,
- CMS public/private separation,
- AI artifact provenance.

## Initial Threats

- unauthorized reads across clearance boundaries,
- forged device or workload context supplied to policy evaluation,
- write-down from high classification to lower outputs,
- query inference through aggregation or repeated small queries,
- projection mixing incompatible compartments,
- stale vector/search results leaking deleted or unauthorized facts,
- AI artifact poisoning,
- manifest rollback,
- WAL or segment corruption,
- unauthorized export,
- plugin capability escape.

## Design Responses

- classify facts, projections, blobs, query results, and AI artifacts,
- reject unsafe query plans before execution,
- expose aggregate query proof state only across public API boundaries, not
  per-label planner decisions,
- bind subject, device, and workload context at the authenticated API boundary
  before policy evaluation,
- keep projections rebuildable,
- sign commits and manifests,
- scope keys by compartment and epoch,
- quarantine stale or forked replicas,
- keep public web projections separate from private worlds,
- treat AI output as untrusted until reviewed.

Break-glass access is not a current bypass capability. Existing break-glass
audit metadata records intent and attested context only. The future access model
must be scoped, time-bounded, approval-gated, and auditable before use; it must
not silently become global clearance escalation. AI-assisted identity proofing
can provide evidence, but must not be the sole authorization authority for
emergency access.
