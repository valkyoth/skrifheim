# Security Policy

Security is the first design constraint for `skrifheim`.

## Supported Versions

Only the latest unreleased `main` branch is supported until the first tagged release.

## Reporting A Vulnerability

Do not open a public issue for a vulnerability.

Use private maintainer contact or GitHub private vulnerability reporting once the repository is hosted. Include:

- affected commit or tag,
- operating system and deployment mode,
- reproduction steps,
- impact,
- whether secrets, signatures, policies, or world history can be exposed or modified.

## Security Baseline

- No god-mode operational role is assumed in the design.
- Administrative actions must become threshold-approved where they can expose, downgrade, or rewrite protected truth.
- AI artifacts are untrusted until policy promotes them.
- Storage and query metadata must be crypto-agile.
- Release tags require completed security review evidence.
