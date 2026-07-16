# skrifheim Release Runbook

Status: policy

## Normal Development Commits

Regular implementation commits are allowed locally. Pushes are left to the
maintainer. Tags are never created or pushed unless explicitly requested.

## Version Stop

1. Confirm the version scope in `docs/VERSION_PLAN.md`.
2. Finish the version deliverables.
3. Run `scripts/checks.sh`.
4. Run `cargo deny check`.
5. Run `cargo audit`.
6. Run the version gate script when one exists, such as `scripts/release_0_7_gate.sh`.
7. Run rootless Podman smoke if the milestone includes container support and it is not already covered by the version gate.
8. Stop and call out the pentest handoff for the exact commit.

Use this handoff text:

```text
vX.Y.Z implementation stop reached. Run pentest for this exact commit.
```

Do not tag at this stop.

For final release-candidate qualification, this stop is the **reviewed
implementation commit**: frozen source, configuration, dependency lockfiles,
toolchain metadata, and qualification harnesses. Fuzzing, benchmarks,
crash/recovery tests, platform qualification, backup/restore qualification, and
pentesting bind to this commit. Producing or archiving evidence must not modify
the reviewed source tree.

## Pentest Resolution

1. The maintainer writes temporary findings to root `PENTEST.md`.
2. Read `PENTEST.md` completely.
3. Fix release-scope findings.
4. Update tests, docs, and release notes as needed.
5. Remove `PENTEST.md`.
6. Run `scripts/checks.sh`, `cargo deny check`, and `cargo audit` again.
7. Repeat if a new `PENTEST.md` is provided.

## Tag Readiness

1. Finish all release metadata, docs, and gates first.
2. Commit that release-prep state. This is the reviewed implementation commit
   for final release-candidate evidence.
3. Run or collect required external qualification against that reviewed commit.
   External archives must name the reviewed commit and record content digests,
   authenticated producer identity, trusted timestamp, storage location,
   toolchain versions, harness versions, and PASS/FAIL result.
4. Compute the executable-input digest over source, configuration, lockfiles,
   toolchain specification, build scripts, enabled features, container build
   inputs, and every other input that can affect executable artifacts. Exclude
   only the permitted evidence report and qualification manifest.
5. Build in a hermetic profile for reproducibility claims: network disabled,
   pinned builder/container/compiler/linker digests, fixed locale, timezone,
   timestamps, environment variables and filesystem ordering, explicit target
   triple, and explicit CPU feature baseline. Bit-for-bit claims require a
   clean-room or independent second build.
6. Decide and record whether release binaries, containers, reviewed-commit
   source bundles, and packages are built from the reviewed implementation
   commit. Executable artifacts are not built from the later evidence-only
   commit. If a future release profile permits evidence-only-commit artifacts,
   their hashes and any mechanically verified permitted differences must be
   recorded in a signed post-commit attestation, not inside the evidence-only
   commit.
7. Smoke test the exact binaries, containers, reviewed-commit source bundles,
   and packages built from the reviewed commit that will be published. Record
   their hashes. Tag-generated source archives cannot be smoked before the tag
   exists and must be handled by the post-tag gate.
8. Write the permanent report at `security/pentest/<tag>.md` and, for final RC
   releases with external evidence, the machine-readable qualification
   manifest. The report/manifest must name the reviewed commit,
   executable-input digest, fuzz archive digest, performance/endurance report
   digest, crash-matrix report digest, platform-qualification report digest,
   backup/restore qualification digest, toolchain and harness versions,
   artifact hashes for reviewed-commit artifacts, and PASS status for every
   mandatory qualification class. The manifest must not declare its own future
   commit hash.
9. Commit only the permitted permanent evidence report and qualification
   manifest as the final evidence-only commit. This evidence-only commit must
   be a direct child of the reviewed implementation commit.
10. Run final local gates. The release gate must verify the report's
   `Reviewed-Commit:` against the final commit's first parent and reject
   missing evidence digests, non-PASS results, toolchain/harness mismatches,
   untrusted or expired attestations, retrieval or digest failures,
   executable-input mismatches, forbidden evidence-only commit contents,
   self-referential evidence-commit hash declarations, and unverified
   qualified-versus-published artifact differences.
11. If source, configuration, test harnesses, dependencies, or executable-input
    files change after the reviewed implementation commit, rerun affected
    qualification and create a new reviewed implementation commit.
    Documentation-only corrections may reuse evidence only when the
    executable-input digest proves no qualified build input changed and a
    signed no-impact decision is recorded.
12. Stop until tagging is explicitly authorized. Creating any version tag is
    tagging and must not happen before that authorization. The authorization
    proof must use the release quorum/threshold policy, distinct roles,
    validity windows, and self-approval rejection.
13. After authorization, create and validate the local immutable version tag.
    Authentication is either a signed annotated tag or a lightweight tag plus a
    detached signed attestation binding tag name and target commit. The local
    validation checks the selected authentication mode, evidence-only commit,
    reviewed commit, qualification manifest digest, reviewed-commit artifact
    hashes, reviewed source-bundle hash where used, and PASS status.
14. Obtain separate authorization before pushing the tag. Remote tag push is an
    externally visible irreversible publication event.
15. Push the immutable remote tag only after that authorization. If
    provider-generated archives are used, retrieve, smoke, and hash them after
    the remote tag exists, then bind those hashes in a separate signed post-tag
    attestation.
16. Run the post-tag pre-publication gate. It verifies selected tag
    authentication mode and target, post-tag attestation, tag-generated archive
    hash and smoke result where used, final downloadable objects from their
    actual distribution endpoints, registry/package-index integrity, artifact
    signatures, SBOM/provenance links, evidence freshness windows, release
    quorum proofs, and no mismatch between signed release evidence and
    downloadable bytes.
17. Publish transactionally per channel. For each channel, follow its
    prepare/publish/verify/compensate state machine, record whether hidden
    staging is supported, identify the first irreversible operation, use retry
    and idempotency keys where available, download and verify public endpoint
    bytes, then publish registry indexes, release pages, and `latest` pointers
    last. Never overwrite artifacts or move an existing version tag. Handle
    partial publication by retry or signed yank/revocation metadata.
18. Write a signed release-completion receipt listing every channel, final
    public object digest, immutable URL or registry identity, verification
    time, and PASS/FAIL result.
19. Push only normal commits unless explicitly instructed to push tags.

Tag-generated source archives are handled after the tag exists: either publish
the qualified reviewed-commit source bundle, or retrieve, smoke, hash, and bind
the generated tag archive in a separate signed post-tag attestation.

Evidence freshness windows are checked before tag push and again before public
publication. If a blocking advisory appears after the reviewed commit, abandon
the release, publish signed yank/revocation metadata where applicable, or
restart from a new reviewed commit. Never silently reuse an already-pushed
version tag for a changed release.
