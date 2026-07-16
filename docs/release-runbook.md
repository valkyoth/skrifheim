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
8. When the maintainer reports the pentest is green, write or update the
   permanent digest at `security/pentest/<tag>.md`.
9. Commit the implementation fixes, release metadata, and permanent pentest
   digest together.
10. Wait for GitHub Actions.

## GitHub Result

1. If GitHub Actions are green, the maintainer tells Codex.
2. Codex verifies the local branch is clean and tags only when explicitly
   instructed.
3. Tags are signed when signing is available in the local Git configuration.
4. Codex pushes only the tag when explicitly instructed.
5. If GitHub Actions fail, the maintainer shares the failure. Codex fixes the
   issue, updates the permanent pentest digest if the fix changes the release
   evidence, commits again, and the project waits for GitHub Actions again.

No release tag is created before both pentest and GitHub Actions are green.
Normal development commits may continue between versions, but a version tag is
only a clean stop after the above loop completes.
