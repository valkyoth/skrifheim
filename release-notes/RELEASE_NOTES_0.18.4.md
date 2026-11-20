# skrifheim v0.18.4

Status: implementation candidate; maintainer pentest pending. No signed tag.

## What changed

Six development-only targets cover WAL header decoding, WAL replay, segment
headers/footers, segment file layout, AEAD envelopes and canonical transcripts.
The segment reader and layout target use the same allocation-free extent check.
The replay target compares both public implementations against a separate
bounded state machine, including rejected operations, incomplete tails and
report counts. File byte/frame budgets and counter reconstruction after reopen
remain covered by the real host writer tests, not simulated disk I/O.

## Running the checks

The normal GitHub job runs `scripts/checks.sh` on stable Rust, including
5,503 deterministic cases and checker rejection tests. The local implementation
gate is `sh scripts/release_0_18_4_gate.sh`. Its coverage-guided part uses
`python3 scripts/assurance.py fuzz all` with cargo-fuzz 0.13.2, pinned nightly,
ASan, seed 184, 20,000 runs per target, 512 MiB RSS and a five-second per-input
timeout. Starting seeds are in the assurance crate; subsequent campaigns also
reuse the local corpus in `target/assurance`. Logs are written there per target.
Errors, timeouts and incomplete target counts are not passes.

The excluded fuzz workspace has its own dependency lock and license/advisory
checks. It does not ship in the CLI or require a new GitHub job.

## Update review (2026-11-20)

Rust 1.100.0; sanitization 2.2.0; cargo-deny 0.22.0; cargo-audit 0.23.0;
refreshed compatible transitive crates and both Rust builder image indexes.
Verified current without a version change: actions/checkout v7.0.1 (full SHA),
cargo-sbom 0.10.0, subtle 2.6.1, blake3 1.8.7, sha3 0.12.0, shake 0.1.0,
chacha20poly1305 0.11.0, poly1305 0.9.1 and getrandom 0.4.3.

## Limits

Local implementation gate, 2026-11-20: debug and release workspace suites,
Clippy with warnings denied, repository policies, all 5,503 corpus cases and
six 20,000-run ASan campaigns passed. Both locked graphs passed cargo-deny
and cargo-audit. The rootless CLI smoke and Alpine workspace tests passed.
The no-default-feature facade also cross-checked for FreeBSD x86_64, wasm32
and bare-metal RISC-V. Only the core crate was checked for macOS ARM64 and
Windows x86_64: local cross tools for the facade's BLAKE3 native build were
unavailable. These compile checks are not native runtime tests.

The campaign is bounded evidence, not a cryptographic audit or a proof that
parsers cannot fail. WAL v1 remains CRC-only. Neither production storage
encryption nor authoritative recovery is delivered here. v0.44.0 retains
the broader fuzz and property-testing milestone.
