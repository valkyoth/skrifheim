# skrifheim 0.1.0 Release Notes

Status: implementation stop, pending pentest.

## Scope

`0.1.0` establishes the repository, workspace layout, security policy, release discipline, and initial placeholder models for the causal world-state database.

## Pentest Remediation

- Redaction decisions now block query plan executability until a redaction path exists.
- Signature envelopes reject empty signatures and known fixed-size algorithms with malformed signature lengths.
- Policy denial reasons no longer disclose compartment or releasability names.
- Confidence uses a deterministic fixed-point representation with private invariants.
- Segment header validation rejects unknown versions and missing integrity/encryption metadata.
- Facts reject self-referential causal links.
- Core IDs are non-zero and no longer expose public tuple fields.
- World diffs require a direct parent/child relationship and return deltas.
- `xtask` resolves scripts from the workspace root and maps non-zero child exits to failure without truncation.
- CLI startup output no longer discloses the Rust toolchain baseline.
- Policy tokens are restricted to a bounded ASCII character set before canonicalization.
- Policy read checks no longer use `BTreeSet::contains()` for compartment or releasability membership in the hot path.
- Unicode homograph compartment and releasability tokens are rejected at label/context construction.
- Signature envelopes reject hash-only algorithms such as Blake3 in signing contexts.

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
