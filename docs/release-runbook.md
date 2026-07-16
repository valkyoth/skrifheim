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
5. Decide and record whether release binaries, containers, source archives, and
   packages are built from the reviewed implementation commit. Executable
   artifacts are not built from the later evidence-only commit. If a future
   release profile permits evidence-only-commit artifacts, their hashes and any
   mechanically verified permitted differences must be recorded in a signed
   post-commit attestation, not inside the evidence-only commit.
6. Smoke test the exact binaries, containers, source archives, and packages
   built from the reviewed commit that will be published. Record their hashes.
7. Write the permanent report at `security/pentest/<tag>.md` and, for final RC
   releases with external evidence, the machine-readable qualification
   manifest. The report/manifest must name the reviewed commit,
   executable-input digest, fuzz archive digest, performance/endurance report
   digest, crash-matrix report digest, platform-qualification report digest,
   backup/restore qualification digest, toolchain and harness versions,
   artifact hashes for reviewed-commit artifacts, and PASS status for every
   mandatory qualification class. The manifest must not declare its own future
   commit hash.
8. Commit only the permitted permanent evidence report and qualification
   manifest as the final evidence-only commit. This evidence-only commit must
   be a direct child of the reviewed implementation commit.
9. Run final local gates. The release gate must verify the report's
   `Reviewed-Commit:` against the final commit's first parent and reject
   missing evidence digests, non-PASS results, toolchain/harness mismatches,
   untrusted or expired attestations, retrieval or digest failures,
   executable-input mismatches, forbidden evidence-only commit contents,
   self-referential evidence-commit hash declarations, and unverified
   qualified-versus-published artifact differences.
10. If source, configuration, test harnesses, dependencies, or executable-input
    files change after the reviewed implementation commit, rerun affected
    qualification and create a new reviewed implementation commit.
    Documentation-only corrections may reuse evidence only when the
    executable-input digest proves no qualified build input changed and a
    signed no-impact decision is recorded.
11. Create the signed annotated tag or external signed release attestation only
    after the evidence-only commit exists. It binds the release tag/version,
    evidence-only commit, reviewed commit, qualification manifest digest,
    artifact hashes, source archive hash, and PASS status.
12. Tag only when explicitly instructed.
13. Push only normal commits unless explicitly instructed to push tags.

Tag-generated source archives are handled after the tag exists: either publish
the qualified reviewed-commit source bundle, or hash and attest the generated
tag archive in the signed annotated tag or external release attestation.
