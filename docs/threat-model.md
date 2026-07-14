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
- optional publishing-extension public/private separation,
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
- rollback to a self-consistent old database directory containing old
  manifests, WAL, policies, credentials, key states, deleted facts, missing
  audit events, or reusable emergency approvals,
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
- require a non-rollbackable freshness anchor for production profiles; signed
  manifests and AEAD authenticate a stored state but do not prove it is the
  newest valid state by themselves,
- fail closed on active startup when local storage is older than the freshness
  anchor, and open historical roots only through explicit recovery workflows,
- scope keys by compartment and epoch,
- release keys through scoped key-provider boundaries rather than giving the
  main database process unrestricted root or tenant key authority,
- quarantine stale or forked replicas,
- chain audit records and include audit roots in manifests and freshness
  anchors so valid individual audit records cannot be silently deleted or
  reordered,
- bind query plans to authenticated identity, attestation evidence, snapshot,
  policy epoch, query digest, purpose, expiry, and replay nonce; public query
  requests must not provide the labels or result security metadata of stored
  facts,
- keep public web projections separate from private worlds,
- treat AI output as untrusted until reviewed.

Break-glass access is not a current bypass capability. Existing break-glass
audit metadata records intent and attested context only. The future access model
must be scoped, time-bounded, approval-gated, and auditable before use; it must
not silently become global clearance escalation. AI-assisted identity proofing
can provide evidence, but must not be the sole authorization authority for
emergency access.
