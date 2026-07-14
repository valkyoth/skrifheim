# skrifheim Supply Chain Security

Status: policy

Dependency and tooling changes require deliberate review.

## Rules

- Check the latest stable Rust before changing `rust-toolchain.toml`.
- Check latest crate versions before adding or bumping dependencies.
- Prefer `core` and `alloc` in core crates; prefer local `skrifheim` implementations over external crates.
- Do not add external crates for convenience.
- Discuss and document any external dependency before use.
- Run `cargo deny check` and `cargo audit` before release tagging.
- Git dependencies must be pinned to a revision and approved.
- GitHub Actions must use reviewed immutable SHAs with the reviewed upstream
  version documented in comments or release notes.
- Cargo-installed CI tools must use reviewed exact versions with `--locked`.
- Container base images must use reviewed immutable digests, and container
  builds must use `cargo build --locked`.
- Generated artifacts and vendored code need documented origin and update flow.
- Release gates must produce enough evidence to audit what was actually
  shipped: release notes, pentest digest, SBOM, relevant dependency tree
  snapshots, and current tool/crate review when versions change.
- External standards, laws, compliance packs, cryptographic specifications,
  file-format references, and conformance fixtures must be source-locked before
  `skrifheim` claims behavior derived from them.

## Special Rules

- Use `sanitization` instead of `zeroize` if memory cleanup is needed, after dependency admission.
- Use `base64-ng` instead of `base64` if base64 is needed, after dependency admission.

The full dependency admission process is in [Engineering Policy](engineering-policy.md).
