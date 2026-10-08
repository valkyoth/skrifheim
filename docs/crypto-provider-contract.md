# skrifheim Crypto Provider Contract

Status: v0.18.3 primitive implementation; pending pentest. Concrete encrypted
storage, key persistence, authenticated recovery and production qualification
are not delivered by this milestone.

## Composition And Replacement

`skrifheim-crypto` owns `DigestProvider`, `AeadProvider`, `EntropySource`,
transcripts, context and sealed `AuthenticatedPlaintext`. It has no dependency
on a software algorithm provider or the operating system.
`skrifheim-crypto-rustcrypto` implements the optional software boundary;
`skrifheim-entropy-host` supplies OS entropy. The facade enables them only with
`rustcrypto-provider` and `os-entropy`. Providers are trusted compiled code,
not plugins supplied by a tenant. A dishonest provider can violate its contract.

The future `skrifheim-crypto-brynja` adapter will implement the same interfaces.
It must pass the same five digest known-answer tests, XChaCha draft vector,
envelope golden fixture, negative-context tests, failure cleanup, counter
exhaustion and portability checks. The current brynja SHA-3 workspace was
inspected; its ordinary/hardened states are distinct and its own documentation
does not claim independent qualification. No sibling path dependency is
introduced. Replacement requires admission evidence, not just API compatibility.
Provider-specific key/plaintext owners may change at composition sites; core
callers and envelope bytes remain provider-independent. An algorithm change,
unlike a provider change, requires a new suite and explicit format migration.

## Digest And Transcript Semantics

Five profiles compute SHA3-256, SHA3-384, SHA3-512, SHAKE256 with 32-byte output,
and SHAKE256 with 64-byte output. Output size is not a blanket quantum-security
claim for signatures, encryption, or the entire database.
`ContentDigest::compute`, `ManifestDigest::compute`, and
`WorldIdentityDigest::compute` require their matching transcript kind and exact
requested algorithm/width. Existing byte constructors remain structural input
types, not cryptographic verification proofs. Compact BLAKE3-derived `WorldId`
values remain non-authoritative scaffold handles; they cannot become durable
trust roots. Core code must not add a direct production hash dependency.

Canonical transcripts contain `skrifheim/transcript/v1\0`, a one-byte closed
purpose tag, then strictly increasing nonzero u16 field tags, u32 byte lengths,
and field bytes. Integers are little-endian. Empty fields differ from absent
fields. Limits are 256 fields and 1 MiB total. Rejected appends leave the
transcript unchanged. Sensitive transcript allocations wipe before growth
and on drop. Concrete mandatory fields are frozen by their owning storage
milestone, not inferred from an arbitrary caller-built transcript.

Plaintext semantic identity, ciphertext integrity and keyed equality-safe
references are separate concepts. `ContentDigest` is an unkeyed typed digest,
not a confidentiality-safe deduplication key. Plaintext hashes must not appear
in public filenames, logs or outer headers. A future keyed reference includes
tenant, region, compartment, purpose and erasure group; equality leaks stay
within that admitted domain. KDF and wrapping registries currently admit no
implementation. No key is derived using a plain hash of a secret.

## Generic Envelope V1

The current sole AEAD suite is full-round XChaCha20-Poly1305, numeric ID 1.
It is not a FIPS-validated implementation. The envelope encodes:

| Field | Encoding |
| --- | --- |
| Magic | 8 bytes `SKRIFENC` |
| Version | u16 LE, 1 |
| Suite | u16 LE, 1 |
| Nonce | 24 bytes |
| Ciphertext length | u64 LE, includes the 16-byte tag |
| Ciphertext and tag | At most 64 KiB plaintext plus 16 bytes |

Empty plaintext is supported. Unknown versions/suites, oversized values,
truncation and trailing bytes fail before body allocation. The format is a
generic primitive fixture, not WAL v2 or the final block/segment envelope.

Associated data is exactly `skrifheim/aead/v1\0`, the 44-byte outer header,
key ID (u128 LE), crypto epoch (u64 LE), canonical encryption domain (86
bytes), operation identity (32 bytes), plaintext length (u64 LE), transcript
length (u64 LE), then the transcript. Expected context comes from trusted
resolution, never from the supplied envelope. Epoch and operation identity
must be nonzero. Changing any bound context invalidates authentication.

AEAD authenticates bytes; it does not independently enforce access policy,
current key lifecycle, placement, authorization, or freshness. Reopening the
same envelope under the same expected context remains cryptographically valid.
The operation identity must bind the expected object/revision/placement and
be checked against current anchored state when durable formats land.
Rejecting replay in another context is tested; global replay prevention is
not claimed before manifests and the freshness anchor.

Sensitive tenant/world identifiers, classification, compartment, policy epoch,
key identifiers, transaction ranges and content digests go inside encrypted
inner storage metadata in v0.18.11/v0.18.12. Associated data alone does not hide
them. Only framing, suite, opaque key lookup slot, nonce and length may be
public. This primitive receives key identity out of band; final key-slot lookup
is a later storage responsibility. Filenames must be opaque. High-assurance
profiles use explicit authenticated padding buckets, with bounded unpadding;
padding is not implemented by this unpadded primitive. Size and activity leaks
remain explicit until that format milestone.

## Entropy And Secret Ownership

`getrandom` uses the supported OS CSPRNG with errors propagated. There is no
time/PID/counter fallback and no bundled deterministic production RNG feature.
Test entropy implementations exist only in test modules. Applications must
compose admitted providers; a user-written deterministic `EntropySource` is
not an approved production provider. Alternate getrandom backends, RUSTFLAGS
overrides and custom targets require admission.

Each seal draws a fresh random 192-bit nonce. Each generated key is non-clone
and raw nonce tokens can only be constructed inside the engine. Keys are
capped at 2^32 seal attempts, including failed operations. The birthday
collision bound for independent draws at this cap is approximately 2^-129;
this is probabilistic uniqueness, not proof of impossibility. The current API
generates fresh keys and does not persist/import them. Restart under a new key
does not reuse its nonce domain. Future persisted keys need durable allocation
or usage accounting before storage encryption is enabled. Fork/VM rollback
can invalidate independence: production restoration must obtain fresh entropy
and advance externally anchored incarnation/key state before writing.

`SecretKeyBuffer` and `SensitivePlaintextBuffer` use sanitization clearing
owners. There are no public generic secret-retention closures or byte exports
on these types. Provider operations are synchronous, take owned plaintext,
and return ciphertext or sealed authenticated plaintext; no borrowed secret
crosses await, callbacks supplied by callers, thread spawning or trait-object
escape. Authenticated plaintext can be resealed through the engine. Future
parsing must add a reviewed purpose-specific operation, not expose the buffer.
Caller-owned original inputs remain the caller's cleanup responsibility.

RustCrypto's private key, MAC and sponge states enable its cleanup features,
including the separately enabled Poly1305 feature. The temporary transitive
zeroize exception is maintainer-approved; project-owned code still uses
sanitization. This clears owned source-declared storage, not all stack copies,
registers, optimizer spills, swap, dumps, abort paths or privileged reads.
Timing/codegen and fault-residue qualification remain required before production.

## Suite Lifecycle And Erasure Decisions

Digest, AEAD, KDF, wrapping, signing and quorum suites have separate registries.
No implementation is admitted for KDF/wrapping/quorum yet; signature identities
are structural only. The old `AlgorithmId` is a legacy signature-scaffold
type and still rejects BLAKE3 in signature contexts. It is not used by AEAD.

Every protocol context must configure an exact active write suite and an
explicit accepted read set; numeric IDs are not security rankings. Migration
is read-old/write-new. Mixed-suite manifests enumerate and authenticate each
object's suite, key domain and digest profile. Unknown IDs never fall back.
Write retirement stops new objects; active-state retirement prevents selection
as an active root; read/verification retirement removes historical access.
Emergency rejection, quarantine and intentional crypto-erasure are distinct
audited decisions. Deprecation schedules identify a deadline and exact context.

Resumable migration stages new encrypted objects, records bounded progress,
checks old/new digests and commitments, then atomically publishes an anchored
manifest. Old roots remain reachable until readers and protected roots move.
Multi-digest transitions explicitly map old identities to new roots without
treating truncation or changed algorithms as equal. Retiring an old reader/key
requires evidence that no accessible object needs it, every readable protected
root migrated, or remaining roots were explicitly quarantined, emergency
rejected or crypto-erased. Legal holds, backups, rollback roots and readable
audit history keep their providers until this condition is satisfied.

Erasure uses per-object or bounded erasure-group DEKs, never an accidental
whole-tenant segment key for unrelated deletion subjects. Each readable key
slot must appear in a wrapped-key reachability index covering live objects,
projections, compaction copies, backups and rollback roots. Shared segments
require rewrite/re-encryption of retained objects before deleting old slots.
Legal holds block erasure explicitly; deletion evidence in immutable audit
must retain non-secret identifiers/proofs rather than erased payloads.
Immutable backup key slots cannot claim selective erasure until their external
wrapping authority is revoked or the readable backup is replaced/quarantined.

Future KDF transcripts bind deployment, region, tenant, classification,
compartment, world/revision where applicable, object/erasure group, purpose,
suite and crypto epoch. WAL, segment, projection, backup, export, AI and audit
keys are distinct. Compartment/segment/data metadata now retains deployment
and region ancestry. The main process must not hold root keys; scoped external
key-release enforcement remains the scheduled production authority boundary.

## WAL V1 Stopgap

Opening a writer binds it to one expected encryption domain, holds the file
lock and scans every header/body CRC with 8 KiB scratch before append. Every
existing and incoming frame must match that domain. It does not truncate
evidence. Appends distinguish
buffered, durable, pre-write rejection and ambiguous I/O failure. Partial
writes, flush or sync failure poison the writer until explicit reopen/recovery.
Offsets are checked for exhaustion. Receipts are local byte ranges, not global
LSNs, manifest generation proofs or proof against copied-disk rollback.

Single-batch `append_transaction_once` retains (domain, TxId) as its retry key.
A full-file canonical `WalReplay` pass checks global transaction ordering,
nesting, key/epoch consistency and replay limits for both status and retry.
New begin/batch/commit headers must pass that same replay state before any
bytes are written; incomplete unrelated tails, non-advancing transaction IDs
and regressing crypto epochs fail without changing the WAL. Invalid existing
replay state poisons the writer. Domain rejection and invalid new candidates
do not poison otherwise valid state.
A completed retry must match every batch/marker byte and syncs before reporting
AlreadyDurable. Incomplete/conflicting attempts cannot automatically append.
`transaction_status` inspects only the locked local WAL; absence is not proof
against rollback. Raw `append_frame` remains low-level and does not implement
transaction idempotency or global replay validation on individual frame writes;
it must not be used as a transaction commit API. Multi-batch transactions, ordering commitments,
authoritative LSN/incarnation and authenticated status belong to WAL v2.

Versioned diagnostic outcome bytes are version 1, outcome byte (0 buffered,
1 durable, 2 ambiguous), start/end u64 LE and TxId u128 LE. They are sensitive
local evidence, never parsed as a durable authorization claim. Golden tests
freeze transcript, digest, envelope and outcome encoding. CRC64 remains only a
structural corruption check. Production integrity needs concrete AEAD formats,
authenticated manifests and the non-rollbackable freshness anchor.
