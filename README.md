<p align="center">
  <b>Skrifheim is a Rust world database for signed, versioned, policy-bound facts, branchable worlds, causal provenance, tamper-evident storage, and security-aware query planning.</b><br>
  Built for causal provenance, tamper-evident history, strict release gates, and rootless containers.
</p>

<div align="center">
  <a href="docs/IMPLEMENTATION_PLAN.md">Implementation Plan</a>
  ·
  <a href="docs/VERSION_PLAN.md">Version Plan</a>
  ·
  <a href="docs/security-controls.md">Security Controls</a>
  ·
  <a href="docs/hyve-cluster-and-compliance-roadmap.md">Compliance And Hyve Roadmap</a>
  ·
  <a href="SECURITY.md">Security</a>
</div>

<br>

<p align="center">
  <img src="./.github/images/skrifheim.webp" alt="skrifheim overview">
</p>

# skrifheim

`skrifheim` is a world database.

The 1.0 target is a serious production-ready causal world-state database for
applications that need signed, versioned, policy-bound facts; branchable worlds;
provenance; classification-aware planning; and tamper-evident storage.
CMS-style publishing, messenger, forum, forge, and other application-family
support is planned as optional compiled-in extension crates over the core world
database primitives.

The project is currently implementing `v0.18.3` production digest and AEAD
engine admission.
It is not a usable database engine.

`skrifheim` is licensed under the European Union Public Licence 1.2.

## What Works Today

### Repository Foundation

| Capability | Status | Notes |
| --- | --- | --- |
| Rust workspace | Active | Edition 2024, resolver `3`, Rust stable `1.97.1` pinned. |
| Core crate split | Active | Focused crates for core types, facts, worlds, policy, crypto envelopes, storage metadata, query planning, and CLI orchestration. |
| `no_std` core policy | Active | Library crates under `crates/` use `#![no_std]` and `#![forbid(unsafe_code)]`. |
| Dependency policy | Active | `cargo deny` policy denies wildcard external dependencies and unknown sources. |
| Security reporting | Active | Private-first vulnerability process in `SECURITY.md`. |
| Release notes | Active | `release-notes/RELEASE_NOTES_0.18.2.md` records scope, verification, and non-claims. |

### Initial Models

| Capability | Status | Notes |
| --- | --- | --- |
| Core IDs and labels | Scaffolded | Tenant, world, fact, entity, predicate, policy, transaction, actor, identity, source, timestamp, and classification types. |
| Fact builder and validation | Scaffolded | Facts carry valid time, evidence, confidence, policy, labels, causal links, and signature sets. |
| World overlays | Scaffolded | Worlds support deterministic metadata identity, parent pointers, depth, added facts, hidden facts, fork, diff, promotion preflight, rollback preflight, and conflict categories. |
| Authority-aware policy context | Scaffolded | Subject, device, and workload context constrain clearance, compartments, releasability, output classification, and aggregate proof metadata. |
| Query-result classification | Scaffolded | Allowed plans propagate output classification, exact or saturated sovereignty scope, PII-derived state, AI-processing eligibility, and confidence-threshold policy hooks. |
| Index and projection encryption policy | Scaffolded | Secondary, graph, search, vector, columnar, cache, and compaction projection surfaces require projection encryption domains and reject incompatible domain mixing. |
| Memory secrecy boundary | Scaffolded | Secret material enters crypto APIs through bounded non-clone redacted `SecretBytes` wrappers backed by admitted `sanitization` clear-on-drop storage. |
| Identity and audit events | Scaffolded | Typed identities, attestation evidence references, break-glass event shape, signed/encrypted audit-log metadata, and actor-attribution checks. |
| Crypto-agile envelopes | Scaffolded | Algorithm IDs, crypto epochs, lifecycle event sequences, bounded signature sets, key hierarchy metadata, key lifecycle metadata, encryption-domain metadata, and SHA-3/SHAKE digest policy skeletons exist without locking the database to one permanent algorithm. |
| Storage metadata | Scaffolded | Immutable segment headers and footers validate magic, version, transaction range, policy, encryption key, crypto epoch, encryption domain, body length, CRC presence, and content digest presence; fixed segment encoding, host-file staged write/read, redacted publication diagnostics, staging cleanup, CRC verification, v2 footer/header kind binding, explicit v1 legacy-unbound footer parsing rejected on normal reads, host memory caps, and verifier injection exist for opaque encrypted segment bodies; WAL frame headers validate fixed append-only encrypted-frame metadata, non-zero CRC presence, expected-domain binding, host-file append/read smoke coverage, and header-driven replay/recovery state transitions. |
| Query planning primitives | Scaffolded | Query requests become policy decision plans for early read, causality, simulation, and context intents. |

### Tooling And Verification

| Capability | Status | Notes |
| --- | --- | --- |
| Local gate | Active | `scripts/checks.sh` runs formatting, shell syntax, doc links, release metadata, engineering policy, modularity, security policy, clippy, and tests. |
| Release gate | Active | `scripts/checks.sh` is the normal local gate. Version release gates run after pentest findings are resolved and the permanent pentest digest is committed. |
| Rootless Podman | Active | `Containerfile` builds with pinned reviewed image digests, `cargo build --locked`, and a non-root runtime image. |
| Pentest stop rule | Active | Every version has a clean implementation stop before tagging. Root `PENTEST.md` is temporary findings input and must be removed after resolution. |
| Modularity gate | Active | Non-generated Rust files over 500 lines fail the local gate. |
| Engineering gate | Active | Core libraries must stay `no_std`, forbid unsafe code, and avoid `std` imports. |

### Planned Or Not Yet

| Capability | Status | Target |
| --- | --- | --- |
| WAL replay and recovery | Scaffolded | `v0.16.0` adds header-driven replay reports for committed, aborted, and clean-EOF uncommitted transactions; full state recovery remains planned. |
| Quantum-aware digest policy | Scaffolded | SHA-3/SHAKE digest-strength profiles and full-width world/content/manifest digest types before compact IDs become durable storage authority. |
| Immutable segment format | Scaffolded | `v0.17.0` adds policy-scoped immutable segment headers and footers with key epoch, encryption-domain, checksum, and algorithm-agile digest metadata. |
| Immutable segment persistence | Scaffolded | `v0.18.0` adds fixed segment encoding plus host-file writer and reader scaffolds; manifests and indexes remain planned for `v0.19.0` through `v0.20.0`. |
| Production digest and AEAD storage encryption | Planned | `v0.18.3` admits production SHA-3/SHAKE and AEAD primitives; concrete block/segment encryption lands with `v0.18.11`, and WAL-v2 encryption lands with `v0.18.12` before manifests or recovery claim tamper resistance. |
| Early WAL/segment fuzzing | Planned | `v0.18.4` adds deterministic fuzz smoke for the hand-written storage byte parsers; the broader fuzz/property baseline remains later. |
| Release evidence hardening | Planned | `v0.18.5` adds SBOM validation, dependency-tree snapshots, runtime/optional-boundary policy, and release-gate tests before durable storage claims harden. |
| Cross-platform portability baseline | Planned | `v0.18.6` adds Linux, Windows, macOS, BSD, x86_64, AArch64, and future RISC-V portability checks before manifests and recovery depend on host I/O behavior. |
| Early performance and API integration smoke | Planned | `v0.20.2`, `v0.23.2`, and `v0.24.2` collect storage, recovery, transaction, policy-token, and authenticated planner-boundary evidence while the design is still cheap to change. |
| Strict serializable transactions | Planned | `v0.21.0` through `v0.23.0`. |
| Native query parser and execution | Planned | `v0.25.0` through `v0.28.0`. |
| Approval-role authority model | Planned | `v0.26.2` defines executable roles and single-maintainer fallback policy for break-glass, law-pack admission, key ceremonies, declassification, backup restore, and release operations. |
| Platform identity and product boundaries | Planned | `v0.26.3` separates identity authority, shared account/profile facts, operator/support/service identities, guardian consent, product-owned data, and minimal derived claims. |
| Optional extension crate boundary | Planned | `v0.31.2` keeps product-family support in optional `skrifheim-ext-*` crates unless a feature is mandatory generic database core. |
| Rebuildable projections and extension deferral | Planned | `v0.29.0` through `v0.32.1` adds graph/search/vector projection foundations and proves product-family extensions stay optional until after `v1.0.0`. |
| Crypto-agile manifest and threshold signatures | Planned | `v0.33.0` signs manifests; `v0.33.1` defines the threshold-signature or bounded quorum multi-signature proof model before approval-sensitive operations depend on multi-party authority. |
| Audit proofs, backup/restore, and local rollback | Planned | `v0.34.0` through `v0.36.0`, with `v0.35.1` reserved for backup-format evolution and schema compatibility, and `v0.35.2` reserved for local snapshot/rollback retention with locked archive and recovery worlds. |
| Secure bootstrap, public origins, descriptors, and scheduled operations | Planned | `v0.38.1` through `v0.38.3` define one-time bootstrap, origin/alias separation, public descriptors, scheduled operations, cache controls, and secret-free config export. |
| Extension API compatibility freeze | Planned | `v0.38.4` freezes the core APIs that optional `v1.1.0` through `v1.6.0` extension crates compile against without becoming mandatory dependencies. |
| AI artifact provenance | Planned | `v0.41.0`. |
| Distinctive security and truth features | Planned | Causal blast-radius invalidation, signed declassification proofs, AI derivation cones, and propagated confidence with mandatory access control are now tracked in the implementation and version plans. |
| Local-first worlds, mission capsules, and extension trust boundary | Planned | `v0.42.0` through `v0.43.1` adds local metadata, cross-domain export skeletons, and defers source-state/import/plugin implementation to post-1.0 extension crates. |
| Fuzz/property baseline, operations, and hardening | Planned | `v0.44.0` through `v0.51.0`. |
| Standalone legal/compliance passports and placement foundations | Planned | `v0.52.0` through `v0.55.0`, with external source-lock evidence in `v0.52.1`; privacy, private-channel, mailbox, and collaboration workflows move to `v1.6.0`. |
| Production release candidate | Planned | `v0.57.0`. |
| Optional extension crates | Planned | `v1.1.0` through `v1.6.0` add publishing/CMS, messenger/private-channel, forum/discussion, forge/source-state, relationship/feed/media, privacy, mailbox, and collaboration crates. |
| Hyve multi-cell cluster fabric | Planned | `v2.0.0` and later, after the core database and initial extension tracks are stable. |

## Why skrifheim

- **Worlds instead of databases**: production, draft, simulation, audit,
  user-local, and mission worlds are first-class branchable states.
- **Facts instead of rows**: canonical state is signed, versioned, timed,
  evidence-bound, and policy-bound.
- **Security-aware planning**: classification, compartments, releasability,
  redaction, and rejection are database planning concerns, not application-side
  decoration.
- **Compliance-aware direction**: future instance, data, and operation
  passports let standalone reads, optional extension access, exports, indexing,
  backup, AI processing, placement, replication, and failover respect signed law
  and compliance packs.
- **Tamper-evident direction**: WAL, immutable segments, manifests, signatures,
  and audit proofs are planned as the storage foundation.
- **AI is not truth**: AI output is planned as derived artifact state with
  provenance and review, never silent authoritative mutation.
- **Truth has blast radius**: causal links, declassification proofs, AI
  derivation cones, and propagated confidence are planned as first-class
  security controls.
- **Strict engineering posture**: core crates are `no_std`, unsafe code is
  forbidden, external crates require admission, and release stops require
  pentest review.
- **Optional verticals**: CMS, messenger, forum, forge, collaboration, and
  other product-family support belongs in extension crates that depend on the
  secure core, not in the mandatory database runtime.

## Quick Start

Build the workspace:

```bash
cargo build --workspace
```

Run the current CLI:

```bash
cargo run -p skrifheim
```

Expected output:

```text
skrifheim 0.18.2
```

Run the normal local checks:

```bash
scripts/checks.sh
```

Release gates are run after the pentest stop has passed and the permanent
pentest digest has been committed.

## Rootless Podman

Build and run the local container:

```bash
scripts/podman_smoke.sh
```

The current container only starts the CLI and prints build identity. Durable
database operation begins in later storage and runtime milestones.

## Workspace

| Crate | Purpose |
| --- | --- |
| `skrifheim` | Main crate and CLI entry point. |
| `skrifheim-core` | IDs, timestamps, labels, values, and shared errors. |
| `skrifheim-fact` | Signed policy-bound fact model. |
| `skrifheim-world` | World branch and overlay model. |
| `skrifheim-policy` | Classification and planner decision model. |
| `skrifheim-crypto` | Crypto-agile algorithm and signature envelopes. |
| `skrifheim-audit` | Identity, attestation evidence reference, and audit-event metadata. |
| `skrifheim-storage` | Storage format and tamper-evident metadata model. |
| `skrifheim-query` | Query planning primitives. |
| `xtask` | Project automation helper. |

## Security Posture

`skrifheim` is designed around military-security constraints:

- no god-mode database assumption,
- no unsafe code in core crates,
- no external dependencies without admission,
- no `std` in core library crates,
- no AI output as authoritative truth,
- no release tag without a clean stop and pentest resolution,
- no legal/compliance-sensitive access, derivation, backup, export, or movement
  without signed policy inputs and audit proof,
- no root `PENTEST.md` committed.

See [Engineering Policy](docs/engineering-policy.md), [Unsafe Policy](docs/unsafe-policy.md),
[Threat Model](docs/threat-model.md), and [Security Controls](docs/security-controls.md).

## Release Process

Each version has a clean implementation stop. When the version criteria are
done, the maintainer runs a pentest for the exact commit and writes temporary
findings to root `PENTEST.md`. Findings are fixed, `PENTEST.md` is removed, and
the gates are rerun before any permanent pentest report or tag.

Tags are created only when explicitly requested.

## Documentation

- [Implementation Plan](docs/IMPLEMENTATION_PLAN.md)
- [Version Plan](docs/VERSION_PLAN.md)
- [Engineering Policy](docs/engineering-policy.md)
- [Encryption Architecture](docs/encryption-architecture.md)
- [Memory Secrecy](docs/memory-secrecy.md)
- [Hyve Cluster And Compliance Roadmap](docs/hyve-cluster-and-compliance-roadmap.md)
- [Security Controls](docs/security-controls.md)
- [Threat Model](docs/threat-model.md)
- [Optional Publishing Extension Target](docs/publishing-extension-target.md)
- [Toolchain Policy](docs/toolchain-policy.md)
- [Release Runbook](docs/release-runbook.md)
