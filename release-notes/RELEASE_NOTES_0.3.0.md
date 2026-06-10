# skrifheim 0.3.0 Release Notes

Status: implementation stop, pending pentest.

## Scope

`0.3.0` makes authority context explicit for security-label evaluation.
Read planning now accounts for subject, device, and workload constraints.

## Changes

- Added `DeviceId` and `WorkloadId`.
- Added `DeviceContext`, `WorkloadContext`, and `AuthorityContext`.
- Query planning now evaluates labels against the combined authority context.
- Subject, device, and workload clearance must all dominate the label
  classification.
- Subject, device, and workload context must all carry required compartments.
- Subject, device, and workload context must all be releasable to required
  release targets.
- Added negative tests for read-above-clearance, device-limited reads,
  workload-limited reads, missing compartments, and missing releasability.
- Bumped workspace and internal crate dependency versions to `0.3.0`.

## Verification

- `scripts/checks.sh`
- `scripts/release_0_3_gate.sh`
- `cargo deny check`
- `cargo audit`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `scripts/podman_smoke.sh`

## Non-Claims

This release is not a usable database engine. It does not provide durable
storage, networking, replication, cryptographic verification, or production
query execution.

`0.3.0` models subject, device, and workload authority inputs, but it does not
attest that those inputs correspond to the physical requester. That binding
belongs to the future authenticated API, transport, and attestation layers.

## Pentest Status

Pentest is required before tagging. Root `PENTEST.md` is the temporary findings
handoff file and must be removed after findings are resolved.
