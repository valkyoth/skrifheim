# skrifheim 0.13.0 Release Notes

Status: implementation stop, first pentest pass resolved locally, pending
retest.

## Scope

`0.13.0` adds the first identity and audit-event scaffold. Security-relevant
actions can now be represented with typed actor attribution, optional device
and workload posture evidence references, audit-log encryption domain metadata,
signature metadata, and break-glass event shape.

## Changes

- Added the `skrifheim-audit` crate.
- Added typed core IDs for users, services, nodes, replicas, plugins, AI
  workers, backup agents, admin tools, attestation evidence, and audit events.
- Added `AuditIdentity` for user, device, workload, service, node, replica,
  plugin, AI worker, backup-agent, admin-tool, and generic actor attribution.
- Added `AttestationEvidenceRef` for device and workload posture references.
- Added `DeviceAuditContext` and `WorkloadAuditContext`.
- Added `AuditEvent`, `AuditEventInput`, and `AuditEventKind`.
- Added a break-glass audit-event skeleton with explicit justification codes.
- Added `AuditLogProtection` for audit-log encryption-domain and signature-set
  metadata.
- Added `AuditRecord` tenant-consistency validation between event and audit-log
  protection metadata.
- Added redacted debug output for audit identities, attestation references,
  device/workload contexts, events, protection metadata, and records.
- Added tests for required actor attribution, attestation time bounds,
  break-glass context requirements, audit-log domain validation, tenant
  consistency, signed audit-log metadata, and redacted diagnostics.
- Replaced derived `Debug` output for segment headers, segment-header inputs,
  world metadata, and worlds with redacted diagnostics.
- Redacted fact actor attribution and policy identifiers in `Fact` diagnostics.
- Redacted audit event kind in `AuditEvent` diagnostics so break-glass
  justifications do not leak through logs.
- Required break-glass audit events to carry attested device and workload
  contexts.
- Rejected audit events with stale or future-dated attestation evidence.
- Extended security validation to gate storage, world, fact, and audit
  diagnostic redaction.
- Confirmed the `MissingAuditActor` diagnostic specificity note is already
  covered by `docs/engineering-policy.md`: trust-boundary paths must use
  `SkrifheimError::public_message()` rather than `Display`, `to_string()`, or
  generic error serialization.
- Bumped workspace and internal crate dependency versions to `0.13.0`.
- Added `scripts/release_0_13_gate.sh`.

## Verification

- `cargo test -p skrifheim-audit`
- `scripts/checks.sh`
- `scripts/release_0_13_gate.sh`

## Non-Claims

This release does not persist audit logs, encrypt audit bytes, verify
signatures cryptographically, attest a physical device, bind workload identity
to mTLS or hardware evidence, implement threshold approvals, or execute
break-glass policy. It only creates the typed metadata boundary future storage,
transport, and policy layers must use.

## Pentest Status

The first `0.13.0` pentest pass has been resolved locally. Root `PENTEST.md`
remains the temporary findings handoff file and must be removed after findings
are resolved.
