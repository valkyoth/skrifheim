# skrifheim Security Controls

Status: baseline control map

| Area | Control | Current Status | Evidence |
| --- | --- | --- | --- |
| Toolchain | Rust stable `1.96.0` pinned | Active | `rust-toolchain.toml` |
| Release arithmetic | Release profile keeps overflow checks enabled | Active | `Cargo.toml` |
| Core runtime | Core library crates are `no_std` | Active | `scripts/validate-engineering-policy.sh` |
| Dependency policy | License, source, advisory, and duplicate checks | Configured | `deny.toml` |
| Security reporting | Private-first vulnerability handling | Configured | `SECURITY.md` |
| Unsafe code | Forbidden in scaffold | Active | `scripts/validate-security-policy.sh` |
| Modularity | Focused crates and file-size gate | Active | `docs/modularity-policy.md` |
| Canonical truth | Facts are versioned and evidence-bound | Scaffolded | `skrifheim-fact` |
| World promotion safety | Promotion and rollback preflight expose deterministic conflict categories before merge/promotion logic exists, using sorted-slice scans to avoid transient set allocation | Scaffolded | `skrifheim-world` |
| Classification | Labels and clearance checks | Scaffolded | `skrifheim-policy` |
| External error shape | `SkrifheimError::public_message()` provides generic messages for trust-boundary responses; `Display` is internal diagnostic output only | Scaffolded | `skrifheim-core` |
| Diagnostic redaction | Sensitive fact payloads, labels, policy tokens, signatures, authority contexts, query-result inputs, result classifications, planner decisions, policy proofs, query requests, query plans, and exact query/proof input counts use redacted `Debug`; direct sensitive `Debug` derives are release-gate blocked | Scaffolded | `scripts/validate-security-policy.sh` |
| Sensitive equality | Security labels, policy-token sets, policy-token slots, signature envelopes, signature sets, planner decisions, policy proofs, query-result inputs, result classifications, query requests, and query plans do not expose derived or manual `PartialEq`/`Eq`; structural comparison is explicit and not for policy or crypto decisions | Scaffolded | `scripts/validate-security-policy.sh` |
| Signature-set bounds | Maximum signature-envelope count and bounded signature key identifiers before durable ingest accepts untrusted commits | Scaffolded | `skrifheim-crypto` |
| Signature algorithm admission | Named and hybrid signature algorithm identifiers require a closed approved list | Scaffolded | `skrifheim-crypto` |
| Fact-builder complexity | O(n log n) deduplication, fail-fast builder bounds, and bounded text/byte fact payloads | Scaffolded | `skrifheim-fact` |
| Blast-radius invalidation | Forward causal DAG traversal identifies tainted downstream facts, projections, releases, and AI artifacts | Planned | `docs/IMPLEMENTATION_PLAN.md` |
| Declassification | Write-downs require signed provenance-bearing declassification proofs | Planned | `docs/IMPLEMENTATION_PLAN.md` |
| AI derivation cones | AI writes are capability-scoped with classification ceilings and traceable derivation cones | Planned | `docs/IMPLEMENTATION_PLAN.md` |
| Confidence and mandatory access control | Propagated confidence is evaluated together with mandatory access control | Planned | `docs/IMPLEMENTATION_PLAN.md` |
| Timing side channels | Policy-token checks use fixed-slot byte sets, local compiler barriers, bounded scans, and fail-closed oversize guards; production constant-time evidence remains required | Scaffolded | `docs/engineering-policy.md` |
| Query inference | Public query plans expose aggregate proof/decision state, not per-label decisions; query requests do not expose raw result-input slices after construction, and query-result inputs do not expose raw metadata accessors publicly | Scaffolded | `skrifheim-query`, `skrifheim-policy` |
| Query request memory | Query requests and aggregate policy helpers fail closed for empty or oversized input sets, use constructor-enforced input limits, store one exact-capacity result-input vector, and keep an explicit fixed result-input memory budget | Scaffolded | `skrifheim-query`, `skrifheim-policy` |
| Query-result classification | Allowed query plans propagate output classification, sovereignty, PII-derived output, AI-processing eligibility, and confidence-threshold policy metadata; non-allow plans expose only public sentinel metadata | Scaffolded | `skrifheim-policy`, `skrifheim-query` |
| Query proof construction | Planner decisions and policy proofs are constructed inside `skrifheim-policy`; public callers inspect aggregate state but cannot forge non-allow proofs with sensitive result metadata | Scaffolded | `scripts/validate-security-policy.sh` |
| Crypto agility | Algorithm and signature envelopes | Scaffolded | `skrifheim-crypto` |
| Key hierarchy | Root trust, deployment, region, tenant, compartment, segment, and data key parent metadata with tenant deployment/region and segment/data compartment binding | Scaffolded | `skrifheim-crypto` |
| Key lifecycle | Creation, activation, rotation preflight, compromise, quarantine, unsafe-parent rejection, destruction, and destruction-gated crypto-erasure metadata | Scaffolded | `skrifheim-crypto` |
| Encryption domains | Tenant, region, classification, compartment, world, WAL, segment, projection, backup, export, AI artifact, WASM/plugin secret, and audit-log blast-radius boundaries with exact merge compatibility checks | Scaffolded | `skrifheim-crypto` |
| Encryption control plane | Key hierarchy, lifecycle, domains, memory secrecy, encrypted projections, and compromise handling | Planned | `docs/encryption-architecture.md` |
| Legal/compliance planning | Instance/node, data, and operation passports with signed law-pack metadata and legal operation/transfer decisions | Planned | `docs/hyve-cluster-and-compliance-roadmap.md` |
| Sovereign placement | Placement and failover decisions constrained by jurisdiction, compliance, data category, and legal basis | Planned | `docs/hyve-cluster-and-compliance-roadmap.md` |
| Cluster control plane | Future Hyve cells, control plane, policy-scoped tunnels, witness nodes, and compliance autopilot | Planned | `docs/hyve-cluster-and-compliance-roadmap.md` |
| Tamper evidence | Segment header metadata and structural validation, including typed non-zero encryption key identifiers and explicit checksum presence representation; body CRC/hash recomputation is required before accepting segment bytes | Scaffolded | `skrifheim-storage` |
| Segment size bounds | Segment headers reject empty and oversized bodies before future segment readers allocate | Scaffolded | `skrifheim-storage` |
| World identity collision resistance | Deterministic world IDs are non-secret scaffold identifiers; storage must enforce uniqueness on `(tenant_id, kind, depth, parent, name)` and collision-resistant derivation is required before storage-key authority | Planned | `docs/VERSION_PLAN.md` |
| Tenant world quotas | Tenant-level world count and aggregate tracked fact-reference budgets are required before storage-backed world creation is exposed | Planned | `docs/IMPLEMENTATION_PLAN.md` |
| AI authority | AI artifacts are derived, not truth | Planned | `docs/IMPLEMENTATION_PLAN.md` |
| CMS isolation | Public/private world split | Planned | `docs/cms-1-0-target.md` |
| Rootless container | Podman smoke path | Scaffolded | `scripts/podman_smoke.sh` |

## Admission Rule

Security-sensitive features do not graduate from planned to active until they have tests, documentation, failure-mode analysis, and release-gate coverage.
