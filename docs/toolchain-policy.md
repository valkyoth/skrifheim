# skrifheim Toolchain Policy

Status: policy

`skrifheim` currently pins Rust stable `1.97.0`.

This was checked against the official Rust release announcements on July 14,
2026. The Rust project announced Rust 1.97.0 on July 9, 2026 as the current
stable release.

## Update Rule

Before changing the toolchain:

1. Check the official Rust release announcements.
2. Read the release notes for compatibility and security changes.
3. Run `scripts/checks.sh`.
4. Update this document and release notes.

## Crate Rule

Before adding a third-party crate:

1. Check crates.io for the latest stable version.
2. Review license compatibility with EUPL-1.2.
3. Review maintenance and advisory status.
4. Add tests that cover behavior introduced by the crate.
