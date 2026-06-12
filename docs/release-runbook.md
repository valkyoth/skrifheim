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
6. Run the version gate script when one exists, such as `scripts/release_0_5_gate.sh`.
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

1. Write the permanent report at `security/pentest/<tag>.md`.
2. Confirm the report names the exact commit and has `Status: PASS`.
3. Run final local gates.
4. Tag only when explicitly instructed.
5. Push only normal commits unless explicitly instructed to push tags.
