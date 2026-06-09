# skrifheim 0.1.0 Release Notes

Status: planned foundation release.

## Scope

`0.1.0` establishes the repository, workspace layout, security policy, release discipline, and initial placeholder models for the causal world-state database.

## Verification

- `scripts/checks.sh`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`

## Non-Claims

This release is not a usable database engine. It does not provide durable storage, networking, replication, cryptographic verification, or production query execution.
