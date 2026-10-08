# skrifheim Toolchain Policy

Status: policy

`skrifheim` currently pins Rust stable `1.99.0`.

This was rechecked with `rustup check`, `rustc --version --verbose`, and
`cargo --version --verbose` on October 8, 2026. The project pins
Rust `1.99.0` (`b940084d7`, 2026-09-28). The official
[announcement](https://blog.rust-lang.org/2026/10/01/Rust-1.99.0/) and
[release notes](https://doc.rust-lang.org/releases.html) were reviewed.
The new unsafe APIs are not needed by this project.

The same review checked crates.io and upstream tags: `sanitization` `2.1.0`,
`blake3` `1.8.7`, and `subtle` `2.6.1` are the current admitted direct
dependencies. `actions/checkout` `7.0.1`, `cargo-deny` `0.20.2`,
`cargo-audit` `0.22.2`, and `cargo-sbom` `0.10.0` remain current.
Containerfiles pin refreshed multi-platform Rust 1.99.0 and distroless
Debian 12 nonroot image indexes. Local Podman is `6.1.2`; host package
management remains outside the repository. Upstream Podman `6.1.3` fixes
[CVE-2026-94603](https://github.com/podman-container-tools/podman/releases/tag/v6.1.3);
upgrade the host package when available. Repository smoke tests build normal
images and never consume checkpoint images.

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
