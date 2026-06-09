# skrifheim 0.1.0 Release Notes

Status: implementation stop, pending pentest.

## Scope

`0.1.0` establishes the repository, workspace layout, security policy, release discipline, and initial placeholder models for the causal world-state database.

## Verification

- `scripts/checks.sh`
- `scripts/release_0_1_gate.sh`
- `cargo deny check`
- `cargo audit`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `scripts/podman_smoke.sh`

## Non-Claims

This release is not a usable database engine. It does not provide durable storage, networking, replication, cryptographic verification, or production query execution.

## Pentest Status

Pentest is required before tagging. Root `PENTEST.md` is the temporary findings handoff file and must be removed after findings are resolved.
