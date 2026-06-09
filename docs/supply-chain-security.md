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
- Generated artifacts and vendored code need documented origin and update flow.

## Special Rules

- Use `sanitization` instead of `zeroize` if memory cleanup is needed, after dependency admission.
- Use `base64-ng` instead of `base64` if base64 is needed, after dependency admission.

The full dependency admission process is in [Engineering Policy](engineering-policy.md).
