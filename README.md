# skrifheim

`skrifheim` is a world database.

The 1.0 target is a serious production-ready causal world-state database for applications that need signed, versioned, policy-bound facts; world branches; provenance; classification-aware planning; tamper-evident storage; and future CMS integration through typed facts, worlds, atomic releases, sanitized projections, and AI artifacts with provenance.

## Current State

Status: `v0.1.0` foundation scaffold.

This repository currently contains the workspace structure, initial crate boundaries, validation scripts, security policy, and implementation/release plans. It is not a production database yet.

## Non-Negotiable Rules

- Use Rust stable `1.96.0` until a newer stable release is checked and deliberately adopted.
- Keep crates focused. The main `skrifheim` crate orchestrates smaller crates.
- Security comes before features, performance, and convenience.
- Canonical truth is append-only, versioned, and auditable.
- Projections are rebuildable from canonical facts.
- AI output is never authoritative by default.
- Rootless Podman must remain a supported deployment path.
- Linux, Windows, BSD, and macOS are supported from day one at the code boundary; Android and iOS remain planned where realistic.

## Workspace

- `crates/skrifheim`: main crate and CLI entry point.
- `crates/skrifheim-core`: IDs, timestamps, labels, and common types.
- `crates/skrifheim-fact`: signed policy-bound fact model.
- `crates/skrifheim-world`: world branch and overlay model.
- `crates/skrifheim-policy`: classification and planner decision model.
- `crates/skrifheim-crypto`: crypto-agile algorithm and signature envelopes.
- `crates/skrifheim-storage`: storage format and tamper-evident metadata model.
- `crates/skrifheim-query`: query planning primitives.
- `tools/xtask`: project validation helper.

## Local Checks

```sh
scripts/checks.sh
```

The current gate runs formatting, clippy, tests, doc-link checks, modularity checks, release metadata checks, and policy checks.

## Documentation

- [Implementation Plan](docs/IMPLEMENTATION_PLAN.md)
- [Version Plan](docs/VERSION_PLAN.md)
- [Engineering Policy](docs/engineering-policy.md)
- [Security Controls](docs/security-controls.md)
- [Threat Model](docs/threat-model.md)
- [CMS 1.0 Target](docs/cms-1-0-target.md)
- [Toolchain Policy](docs/toolchain-policy.md)
