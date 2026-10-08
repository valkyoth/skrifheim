# skrifheim Testing Strategy

Status: development guidance; rollout owners are in the version plan.

## Scope And Workflow

This guidance adapts selected testing patterns reviewed in brynja at
`1cf227f6`: `docs/focused-assurance.md`, `docs/miri-verification-design.md`,
`docs/family-development-review.md`, the public SHA-3 consumer fixture,
checker mutation tests, and fault-injecting test support. It is not a review
or qualification of that entire project, and imports no code or dependencies.

Keep the existing workflow: implement, run local checks, call "ready for
pentest", resolve/retest, commit the permanent report, wait for green GitHub,
then create/push a signed tag only on explicit instruction. No new manual
approval tokens, branch protection, required GitHub jobs, remote workers,
nightly installations or hardware campaigns are introduced by this guidance.

Cheap deterministic tests join ordinary Cargo tests or `scripts/checks.sh` as
their milestones land. Expensive assurance belongs to the relevant local
version gate. Codex runs it and records results in existing release evidence;
the maintainer does not gain another routine command or report to maintain.

## Reusable Patterns

### Public Consumer Tests

Test usable behavior through public crate/facade APIs, not only private unit
helpers. Use integration tests before introducing a separate fixture crate;
add an unpublished consumer crate only when package or dependency isolation
requires it. Exercise the real provider path and compare with fixed,
independently sourced expected bytes, not values recomputed by the same code
under test. Round trips complement, but cannot replace, these expectations.

Cover empty, boundary, over-limit, malformed, wrong-domain and revoked inputs.
Compile-fail examples prove inaccessible constructors, forbidden secret copies
and invalid typestate transitions; pair them with a valid compiling consumer
so an unrelated build failure cannot masquerade as a protected boundary.
Provider replacement, including brynja, must pass the same consumer contract.

### Test The Checkers

For a security/release checker, first prove a valid fixture passes. Then
introduce deliberate violations in isolated temporary fixtures and assert the
specific rejection, not just any nonzero exit. Missing tools, syntax errors,
network failures and unrelated compile errors are incomplete verification,
not successful rejection tests. Never mutate the working source tree to run
these tests. Start with narrow regression fixtures, not a mutation-testing
framework or new external dependency.

### Faults And Safety Obligations

Use deterministic fault sources for short I/O, failed sync, unavailable or
partially filled entropy, clock regression, counter exhaustion, allocation
failure where injectable, cancellation, revocation and ambiguous completion.
Assert observable postconditions: no unauthorized output, no partial mutation
on rejection, no lost acknowledged state and safe retry/poison behavior.

Keep fixtures in test-only modules or unpublished dev-only support crates;
no feature combination may admit deterministic entropy or permissive authority
fixtures into production. Share a helper only when it removes real duplication.

Organize expensive cases by named safety obligation: initialization, bounds,
state transitions, publication, rollback, cleanup, cancellation and concurrency.
Boundary counters can be initialized near exhaustion in private test helpers;
do not loop billions of times merely to reach the boundary. Production behavior
must not be replaced with a no-op to make an interpreter or sanitizer pass.

### Honest Completion And Evidence

Filtered assurance runs must confirm the named tests/cases actually executed.
Zero selected tests, missing cases, unexpected ignores, timeouts and cancelled
runs never satisfy a required check. Reject an unknown selector rather than
quietly executing nothing. Do not hard-code a global workspace test count;
bind focused runners to their own stable case inventory.

Distinguish freshly passed, reused, deferred, unsupported and failed evidence.
Reused evidence retains its original commit, compiler, target, features,
dependencies, fixture and verifier identity. Documentation changes alone do
not renew expensive hardware/timing evidence. Changed relevant source, compiler,
flags, dependencies or test semantics require affected verification; a filename
such as `tests.rs` is not sufficient proof that a change is irrelevant.

Initially record this assessment in the existing release notes and pentest
digest, without importing brynja's selection/receipt infrastructure. Unknown
impact is reported as incomplete until reviewed, never silently passed.
Production candidates retain their full required qualification gates.

Native execution, cross-compilation, emulation, Miri, code-generation review
and statistical timing answer different questions. None substitutes for all
the others, and no passing local test establishes independent certification.
Unsafe/platform paths stay subject to admission and the portable fallback.

## Scheduled Adoption

| Version | Required Adoption |
| --- | --- |
| `v0.18.4` | Public crypto/storage consumer tests, differential replay and bounded fuzz coverage with real completion checks |
| `v0.18.5` | Checker positive/negative controls, production fixture exclusion and default/optional-feature build coverage |
| `v0.18.6` | Run portable public consumers on qualified targets; separate compile and native evidence |
| `v0.18.9` | Real key-provider fault and public-contract acceptance, reusable by a future brynja adapter |
| `v0.18.13` | Shared deterministic host fault seams and crash-oracle postconditions |
| `v0.20.1` | Timing/codegen evidence with harness sensitivity controls and explicit compiler/target scope |
| `v0.22.1` | Bounded concurrency obligations and deliberately broken publication/order fixtures |
| `v0.23.5` | End-to-end public world/transaction/recovery consumer acceptance |
| `v0.44.0` | Focused Miri/sanitizer/property campaigns, mutation checks and no-zero-test completion |
| `v0.51.0` | Actual caller/provider cleanup paths, including error, unwind and abort non-claims |
| `v0.55.1` | Capability matrix backed by running public consumers, not internal models alone |
| `v0.57.0` | Full candidate qualification with precise fresh/reused/incomplete evidence |

These strengthen existing releases; they do not add a second release calendar.
