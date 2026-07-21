# skrifheim Encryption Architecture

Status: planning document

`skrifheim` must treat encryption as a database control plane, not as a single
"encrypt the files" feature.

## Core Rule

No production storage surface is complete until these are covered:

- WAL encryption,
- immutable segment encryption,
- manifest and checkpoint signature/encryption metadata,
- index and projection encryption,
- backup/export capsule encryption,
- audit-log encryption,
- query-result classification,
- key lifecycle and compromise handling.

## Key Hierarchy

`skrifheim` must not use one database-wide encryption key.

Planned hierarchy:

```text
root trust key
  -> deployment key
    -> region key
      -> tenant key
        -> compartment key
          -> segment/data key
```

Every encrypted object must record:

- encryption algorithm,
- key identifier,
- key epoch,
- policy epoch,
- tenant,
- region or sovereignty boundary,
- compartment or classification boundary,
- rotation and destruction status.

Production key release must be scoped outside the main database process. The
database process must not hold root, deployment, or unrestricted tenant keys in
production profiles. A KMS, HSM, privilege-separated key service, or equivalent
`ScopedKeyProvider` must decide key release from tenant, compartment, purpose,
policy epoch, workload identity, encryption domain, and bound policy proof.
High-assurance deployments may require separate processes or instances per
classification or compartment domain.

Storage must not treat a non-zero encryption key ID as sufficient. Once segment
I/O is wired to key metadata, segment read/write acceptance must validate the
referenced key lifecycle and reject compromised, quarantined, destroyed, or
crypto-erased keys. Old segment reads may allow explicitly rotating keys only
under a read-only migration path.

Segment headers use the typed non-zero `KeyId` metadata identifier rather than a
raw integer. That prevents zero-key sentinels and accidental mixing with
transaction, policy, tenant, or segment-scope identifiers at compile time.

Current WAL and segment body verification is not cryptographic tamper
resistance. CRC64 and unkeyed `ContentDigest` metadata catch accidental
corruption and malformed files, but an attacker with disk write access can
recompute both. No WAL or segment file may be treated as production-trusted
against local tampering until encrypted bodies are authenticated with AEAD,
signed/keyed manifests, or an equivalent admitted integrity root.

`v0.18.3` is the scheduled milestone for admitting the production SHA-3/SHAKE
digest engine, AEAD implementation, entropy provider, generic envelope
primitive, and nonce uniqueness rules. It does not define the final concrete
WAL or segment envelope. Segment and block encryption are instantiated when the
physical storage layout is frozen in `v0.18.11`; WAL-v2 encryption is
instantiated when the ordered log format lands in `v0.18.12`. Later manifests
and recovery code must not claim tamper resistance until those concrete formats
use the admitted primitives and pass wrong-key, wrong-domain, wrong-epoch,
replay, truncation, and corrupt-ciphertext rejection tests.

AEAD must not be treated as sufficient if the log can be spliced from valid
frames. WAL encryption must bind database ID, log incarnation, WAL generation,
LSN, transaction ID, transaction frame ordinal, previous-frame digest,
transaction frame count, and commit transcript root. Recovery must reject gaps,
duplicates, reordered frames, generation mismatches, unexpected ordinals,
missing transaction frames, and commit-root mismatches.

Storage metadata is sensitive. Associated data is authenticated, not normally
encrypted, so the production envelope must separate a minimal plaintext outer
header from encrypted inner metadata. Tenant, world or world-revision,
classification, compartment, policy epoch, key ID, transaction range, body
size where possible, and content identity must not be plaintext by default.
Any plaintext field needs an explicit public-header exception. Opaque filenames
and optional padding buckets remain part of the metadata-minimisation design.

The production digest model must keep separate meanings for plaintext semantic
identity, ciphertext integrity digest, and keyed equality-safe content
references. One digest over encrypted bytes cannot simultaneously be a stable
plaintext content address and a confidentiality-safe storage integrity marker.

Nonce, salt, replay nonce, randomized staging name, key generation, and future
random identifier generation require an admitted entropy/CSPRNG provider.
Production profiles must fail closed when entropy is unavailable or the provider
cannot meet the platform/security profile. Deterministic providers are allowed
only behind test fixtures and must be release-gated out of production builds.

## Key Lifecycle

The control plane must model:

- key creation,
- activation,
- rotation,
- expiration,
- compromise,
- quarantine,
- destruction,
- crypto-erasure,
- backup key recovery,
- threshold unlock,
- emergency access,
- no-escrow deployments.

Dangerous key operations require audit records and future threshold approval.
Crypto-erasure must pass through an explicit destruction checkpoint before key
metadata enters the crypto-erased state.

The crypto epoch identifies key-material/version authority for WAL frames,
segments, manifests, signatures, and recovery checks. It is not the ordering
mechanism for lifecycle metadata. A separate lifecycle event sequence records
compromise, quarantine, destruction, and erasure events when those operations do
not introduce new key material. Rotation and replacement-key activation still
require strictly advancing crypto epochs.

Any future encrypt, decrypt, manifest, recovery, or key-use path must inspect
the full key lifecycle state and fail closed for compromised, quarantined,
destroyed, or crypto-erased keys. A crypto epoch comparison alone is not a trust
decision because metadata-only lifecycle events can share the same crypto epoch
as the key material they affect.

Key hierarchy validation must reject children of compromised, quarantined,
destroyed, or crypto-erased parent keys. Future storage-backed ancestor walks
must propagate those states through the subtree before segment access is
allowed.

Crypto-erasure granularity must be fixed before real storage encryption is
trusted. The design must choose per-object or erasure-group data encryption
keys, wrapped-key indexes, backup key-slot deletion, compaction and
re-encryption protocol, snapshot/legal-hold exceptions, and proof that every
readable key slot was removed. Immutable audit can retain non-secret deletion
evidence, but must not retain decryptable erased payloads unless a legal hold
or explicit override policy authorizes it.

The lifecycle audit layer must preserve full transition history. Repeated
active/rotating movement is not a privilege escalation by itself, but audit
records must make repeated cycles visible so compromise timelines cannot be
obscured by state churn.

## Encryption Domains

Encryption domains are separate blast-radius boundaries.

Required domains:

- tenant,
- region,
- classification level,
- compartment,
- world or branch where needed,
- WAL,
- immutable segment,
- index/projection,
- backup,
- export capsule,
- AI artifact,
- WASM/plugin secret,
- audit log.

The current scaffold models these as metadata-only blast-radius boundaries in
`skrifheim-crypto`. Domain merge compatibility is intentionally exact:
purpose, tenant, region, classification, compartment, world or branch, and
segment identity must match before two encrypted surfaces can be treated as the
same domain. Durable encryption and key derivation are still planned work.
`v0.18.3` turns those domains into associated-data and key-derivation inputs
for WAL and segment bodies; projection, backup, export, AI artifact, and audit
body encryption must reuse the same domain-separation contract in their later
milestones.

## Query-Result Classification

Encryption at rest does not prevent derived-result leaks.

The planner must classify query results:

- Secret fact plus Public fact produces at least Secret output.
- EU-only fact plus Global fact produces EU-constrained output.
- PII fact plus AI summary produces a PII-derived artifact.
- Classified source facts cannot be embedded, summarized, cached, exported, or
  synced into a lower domain without an explicit release path.

The `0.10.0` scaffold records this as `ResultClassification` metadata on
allowed query plans: output classification, sovereignty tokens, PII-derived
state, AI-processing eligibility, and a future confidence-threshold hook.
Rejected and redacted plans deliberately expose only public sentinel metadata.

## Index And Projection Encryption

Indexes can leak nearly as much as the original facts.

These must be encrypted and policy-scoped:

- secondary indexes,
- graph indexes,
- vector indexes,
- full-text indexes,
- columnar projections,
- AI summaries,
- cache files,
- WAL files,
- compaction temporary files,
- debug and diagnostic dumps.

Projection builders must refuse to mix incompatible compartments or encryption
domains.

The `0.11.0` scaffold records this as `ProjectionEncryptionPolicy` metadata for
secondary indexes, graph indexes, search indexes, vector indexes, columnar
projections, cache files, and compaction temporary files. These policies require
`EncryptionDomainPurpose::Projection`, require encryption at rest, disallow
plaintext temporary files, and expose domain-compatibility checks before any
durable projection builder exists.

## Memory Secrecy

Rust memory safety does not automatically protect secrets in RAM.

The `0.12.0` scaffold admitted `sanitization`; the active dependency admission
now pins `sanitization` `2.0.2` with only the `alloc` feature and exposes
`SecretBytes` in `skrifheim-crypto`. `SecretBytes` is a bounded, non-clone,
redacted, clear-on-drop wrapper for future key material, bearer tokens, and
other crypto-control-plane secrets. See
[Memory Secrecy](memory-secrecy.md).

Current controls:

- no secrets in logs or panic text,
- secret buffers cleaned with the project-approved `sanitization` crate,
- raw secret byte access only through closure-scoped APIs,
- rejected owned secret buffers clear before drop,
- secret wrappers must not derive `Debug`, `Clone`, `PartialEq`, or `Eq`.

Planned controls:

- classification-sensitive metadata types such as `QueryResultInput` and
  `ResultClassification` must be included in the reviewed cleanup-on-drop plan
  before untrusted query results are durable or cross a trust boundary,
- crypto-erasure metadata such as `KeyErasureMetadata` must be prioritised for
  reviewed cleanup-on-drop with key-material types,
- `zeroize` is not admitted for this project; if `sanitization` cannot satisfy
  a future key-material requirement, the unsafe-boundary exception process must
  be completed before key bytes land,
- optional locked memory for key material where the OS supports it,
- no swapping of long-lived key material where practical,
- separated secure arenas for key material,
- encrypted or disabled crash dumps,
- constant-time cryptographic verification paths.

## Identity And Audit

Every security-relevant action must be attributable:

- user login,
- service request,
- node/replica action,
- query,
- export/import,
- policy change,
- key rotation,
- declassification,
- backup/restore,
- plugin install,
- AI processing,
- break-glass access.

Audit logs must be append-only, encrypted, signed, and tamper-evident.

## Recovery And Compromise

`skrifheim` must have explicit responses for:

- lost key,
- compromised key,
- compromised tenant,
- compromised compartment,
- captured node,
- rollback attempt,
- stale replica,
- leaked backup,
- failed restore,
- poisoned AI worker,
- malicious plugin.

Compromise handling is a release requirement, not an operations afterthought.
