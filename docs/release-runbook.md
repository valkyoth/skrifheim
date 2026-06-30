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
2. Commit that release-prep state.
3. Write the permanent report at `security/pentest/<tag>.md`.
4. Commit only that report as the final tag-candidate commit.
5. Run final local gates. The release gate must verify the report's
   `Reviewed-Commit:` against the final commit's first parent.
6. Tag only when explicitly instructed.
7. Push only normal commits unless explicitly instructed to push tags.
