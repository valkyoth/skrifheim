# skrifheim v0.18.3

Status: implementation candidate; first pentest and resource-exhaustion retest remediated, awaiting maintainer retest.

## Scope

- Optional RustCrypto provider and separate OS-entropy crate, behind
  skrifheim-owned interfaces designed for future brynja replacement.
- Actual SHA3-256/384/512 and SHAKE256-256/512 computation for typed content,
  manifest and world digests; sealed verified-digest results.
- Generic XChaCha20-Poly1305 envelope with bounded parsing, canonical AAD,
  scope/epoch/key/operation binding, scrubbed secret owners and sealed
  authenticated plaintext. No generic secret-retention callbacks on provider
  buffers. Internal upstream cleanup uses the maintainer-approved temporary
  transitive zeroize exception; our own buffers use sanitization.
- Deployment/region ancestry retained below tenant key scope.
- WAL-v1 existing-tail scans, explicit durability outcomes, poisoned writers
  after write/flush/sync errors, local byte-range receipts, single-batch
  idempotency/status scaffolding and diagnostic golden records.
- First pentest fixes: canonical replay validation before transactional append
  and status, domain-bound WAL writers, fixed-size redacted key-control/WAL-body/
  world-preflight diagnostics and release-gate regression coverage. See the
  [pending-retest digest](../security/pentest/v0.18.3.md).
- Retest fix: fixed-memory replay validation sharing the report transition
  engine, EOF checks after candidate preflight, and hard 128 MiB/8,192-frame
  per-file scan limits. Whole-transaction reservation fails before any write;
  `RotationRequired` is backpressure, not corruption or permission to discard
  data. Automatic checkpoint/rotation remains later storage work.
- [Provider contract](../docs/crypto-provider-contract.md) records suite
  lifecycle, resumable migration, metadata privacy, erasure granularity,
  domain-separated KDF requirements and production non-claims.

## Tooling Review

Checked on 2026-10-08: Rust 1.99.0, sanitization 2.1.0, blake3 1.8.7,
subtle 2.6.1, current compatible lockfile dependencies and refreshed
multi-platform Rust/distroless container indexes. Checkout 7.0.1,
cargo-deny 0.20.2, cargo-audit 0.22.2, cargo-sbom 0.10.0 and rustup 1.29.1
remain current. The provider admission pins current sha3 0.12.0, shake 0.1.0,
chacha20poly1305 0.11.0, poly1305 0.9.1 and getrandom 0.4.3.
Local Podman is 6.1.2; the configured OS repository offers that version.
Upstream 6.1.3 includes a checkpoint-image security fix; host upgrade remains
outstanding. Smoke tests build normal images and do not run checkpoint images.

## Verification

The implementation gate is `sh scripts/release_0_18_3_gate.sh`.
The post-pentest release gate adds `--release` and requires the permanent
pentest report through the existing release-readiness check.

Coverage includes five FIPS 202 known-answer profiles, the XChaCha draft
vector, a generic envelope fixture independently reproduced with libsodium,
context isolation, every-byte envelope mutation/truncation, allocation bounds,
entropy failure, key-use exhaustion, tail corruption, append failures at every
byte/flush/sync boundary, ambiguous outcomes and duplicate/conflicting retries.
Pentest regressions cover unrelated incomplete transactions, regressing IDs
and epochs, invalid/interleaved logs, reader/writer domain agreement, and
constant-size debug formatting at maximum WAL-body/world-list sizes.
Resource regression tests compare generated validator/report histories, cover
one million closed transactions plus an incomplete tail, checked-counter
overflow, sparse oversized files, and exact/over-budget writes and reopen.

Native Linux tests and core/provider checks cover formatting, clippy,
dependency/advisory policy and all features. Cross-target compile evidence
covers Windows MSVC, macOS/AArch64, FreeBSD, RISC-V Linux, big-endian PowerPC
Linux and bare-metal Thumb no_std. Cross-compilation is not runtime
qualification. Rootless normal and Alpine/musl container checks are included
in the local verification pass.

## Limits

The database is still a scaffold. Existing WAL/segment files are not encrypted
or authenticated by the new generic engine. Concrete encrypted block/segment
formats remain v0.18.11; WAL-v2 ordering/authentication remains v0.18.12.
CRC64 is only corruption detection. BLAKE3 WorldId values remain scaffold
handles, not durable authority. No FIPS validation, complete residue erasure,
statistical timing qualification, persisted-key nonce accounting, global replay
protection, KDF/wrapping implementation, signature verification, authoritative
commit lookup, or non-rollbackable freshness is claimed here.

Random-nonce uniqueness is probabilistic. Restored/cloned VM entropy state is
outside this primitive's guarantees and requires the later anchored recovery
workflow. WAL receipts are local observations, not signed commit proofs.

## Release Stop

Run the maintainer pentest on the committed implementation. Resolve findings
and retest until green, then commit the permanent digest and wait for GitHub
green. A signed tag is created and pushed only on explicit instruction.
