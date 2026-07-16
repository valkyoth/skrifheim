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
- Final release candidates must follow the release runbook's evidence model:
  reviewed implementation commit, evidence-only commit, executable-input
  digest, canonical qualification manifest, hermetic build evidence, artifact
  hashes, exact-artifact smoke tests, and machine-readable release provenance.
- Reproducibility claims require network-disabled builds with pinned
  builder/container/compiler/linker digests, fixed locale/time/environment and
  filesystem ordering, explicit target triples and CPU baselines, and
  independent rebuild evidence for bit-for-bit claims.
- Every release needs an immutable version tag authenticated either by a signed
  annotated tag or by a lightweight tag plus detached signed attestation
  binding tag name and target commit. Remote tag push is an externally visible
  publication event requiring explicit authorization.
- Provider-generated tag archives must be handled after the remote tag exists:
  retrieve, smoke, hash, and bind them through a separate signed post-tag
  attestation, or publish a reviewed-commit source bundle instead.
- Release publication must be channel-specific and transactional where the
  channel allows it: upload immutable artifacts first, verify downloads from
  public endpoints, publish indexes/pages/latest pointers last, never overwrite
  versioned artifacts or move version tags, and use signed yank/revocation
  metadata for partial or compromised releases.
- Evidence freshness windows are required for advisory scans, container/base
  image scans, pentest and fuzz evidence, signing certificates, attestations,
  source-lock availability, toolchain review, and build-image review at tag and
  publication time.
- External standards, laws, compliance packs, cryptographic specifications,
  file-format references, and conformance fixtures must be source-locked before
  `skrifheim` claims behavior derived from them.

## Special Rules

- Use `sanitization` instead of `zeroize` if memory cleanup is needed, after dependency admission.
- Use `base64-ng` instead of `base64` if base64 is needed, after dependency admission.

The full dependency admission process is in [Engineering Policy](engineering-policy.md).
