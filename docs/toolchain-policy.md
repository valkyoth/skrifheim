# skrifheim Toolchain Policy

Status: policy

`skrifheim` currently pins Rust stable `1.100.0`.

This was rechecked with `rustup check`, `rustc --version --verbose`, and
`cargo --version --verbose` on November 20, 2026. The project pins
Rust `1.100.0` (`0ef1bc386`, 2026-11-09). The official
[announcement](https://blog.rust-lang.org/2026/11/12/Rust-1.100.0/) and
[release notes](https://doc.rust-lang.org/releases.html) were reviewed.
The new unsafe APIs are not needed by this project.

The review checked crates.io and upstream tags. See the
[v0.18.4 inventory](../release-notes/RELEASE_NOTES_0.18.4.md).
The normal build and CI do not need nightly; the isolated local fuzz campaign
uses `nightly-2026-11-20`. Containerfiles pin multi-platform Rust 1.100.0 and
distroless Debian 12 nonroot indexes. Local and upstream Podman are `6.2.0`.

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
