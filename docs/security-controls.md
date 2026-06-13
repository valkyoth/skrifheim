# skrifheim Security Controls

Status: baseline control map

| Area | Control | Current Status | Evidence |
| --- | --- | --- | --- |
| Toolchain | Rust stable `1.96.0` pinned | Active | `rust-toolchain.toml` |
| Core runtime | Core library crates are `no_std` | Active | `scripts/validate-engineering-policy.sh` |
| Dependency policy | License, source, advisory, and duplicate checks | Configured | `deny.toml` |
| Security reporting | Private-first vulnerability handling | Configured | `SECURITY.md` |
| Unsafe code | Forbidden in scaffold | Active | `scripts/validate-security-policy.sh` |
| Modularity | Focused crates and file-size gate | Active | `docs/modularity-policy.md` |
| Canonical truth | Facts are versioned and evidence-bound | Scaffolded | `skrifheim-fact` |
| World promotion safety | Promotion and rollback preflight expose deterministic conflict categories before merge/promotion logic exists | Scaffolded | `skrifheim-world` |
| Classification | Labels and clearance checks | Scaffolded | `skrifheim-policy` |
| External error shape | `SkrifheimError::public_message()` provides generic messages for trust-boundary responses | Scaffolded | `skrifheim-core` |
| Signature-set bounds | Maximum signature-envelope count per set before durable ingest accepts untrusted commits | Planned for v0.7.0 | `docs/VERSION_PLAN.md` |
| Fact-builder complexity | O(n log n) deduplication for evidence and causal-link builder inputs | Planned for v0.7.0 | `docs/VERSION_PLAN.md` |
| Blast-radius invalidation | Forward causal DAG traversal identifies tainted downstream facts, projections, releases, and AI artifacts | Planned | `docs/IMPLEMENTATION_PLAN.md` |
| Declassification | Write-downs require signed provenance-bearing declassification proofs | Planned | `docs/IMPLEMENTATION_PLAN.md` |
| AI derivation cones | AI writes are capability-scoped with classification ceilings and traceable derivation cones | Planned | `docs/IMPLEMENTATION_PLAN.md` |
| Confidence and mandatory access control | Propagated confidence is evaluated together with mandatory access control | Planned | `docs/IMPLEMENTATION_PLAN.md` |
| Timing side channels | Policy-token comparison and label evaluation need constant-time evidence and fixed-shape checks before production claims | Planned for v0.7.0 | `docs/engineering-policy.md` |
| Crypto agility | Algorithm and signature envelopes | Scaffolded | `skrifheim-crypto` |
| Encryption control plane | Key hierarchy, lifecycle, domains, memory secrecy, encrypted projections, and compromise handling | Planned | `docs/encryption-architecture.md` |
| Legal/compliance planning | Instance/node, data, and operation passports with signed law-pack metadata and legal operation/transfer decisions | Planned | `docs/hyve-cluster-and-compliance-roadmap.md` |
| Sovereign placement | Placement and failover decisions constrained by jurisdiction, compliance, data category, and legal basis | Planned | `docs/hyve-cluster-and-compliance-roadmap.md` |
| Cluster control plane | Future Hyve cells, control plane, policy-scoped tunnels, witness nodes, and compliance autopilot | Planned | `docs/hyve-cluster-and-compliance-roadmap.md` |
| Tamper evidence | Segment metadata and validation | Scaffolded | `skrifheim-storage` |
| AI authority | AI artifacts are derived, not truth | Planned | `docs/IMPLEMENTATION_PLAN.md` |
| CMS isolation | Public/private world split | Planned | `docs/cms-1-0-target.md` |
| Rootless container | Podman smoke path | Scaffolded | `scripts/podman_smoke.sh` |

## Admission Rule

Security-sensitive features do not graduate from planned to active until they have tests, documentation, failure-mode analysis, and release-gate coverage.
