# skrifheim 0.18.0 Release Notes

Status: release-prep stop, local release gate pending, GitHub verification pending.

## Scope

`0.18.0` adds the first immutable segment persistence scaffold. The storage
crate now owns fixed-size segment header/footer byte encoding and parsing, while
the host storage crate can create, write, open, and read a single immutable
segment file with rootless host-file safety checks.

The release keeps the full mirrored 256-byte footer. The overhead is accepted
for this stage because the reader can bind both ends of the file and reject
footer/header mismatch, body corruption, truncation, unexpected domains, and
trailing bytes before exposing encrypted segment bytes.

## Changes

- Bumped workspace and internal crate dependency versions to `0.18.0`.
- Added fixed-size `SegmentHeader::encode`, `SegmentHeader::parse`,
  `SegmentHeader::parse_for_domain`, `SegmentFooter::encode`, and
  `SegmentFooter::parse`.
- Split segment byte encoding and decoding into a dedicated
  `skrifheim-storage` segment encoding module to keep validation and wire
  layout separate.
- Kept the mirrored fixed-size footer layout and documented the overhead
  decision in the implementation and version plans.
- Added header/footer round-trip tests for fixed-size segment bytes.
- Added parser rejection tests for malformed segment magic, version, kind,
  reserved bytes, identifiers, classification tags, digest tags, body CRCs, and
  header/footer mismatches.
- Added `SegmentFileWriter` in `skrifheim-storage-host` for immutable segment
  file creation with Unix `O_NOFOLLOW`, owner-only `0600` permissions, regular
  file checks, optional `sync_all`, body length checks, mandatory body CRC
  verification, and content-digest verifier injection.
- Added `SegmentFileReader` in `skrifheim-storage-host` for expected-domain
  parsing, exact file-length checks, encrypted body reads, body CRC
  verification, footer parsing, footer/header binding, and content-digest
  verifier injection.
- Added `SegmentContentVerifier` as the explicit trust-boundary hook for the
  admitted production digest engine.
- Added host-file tests for segment round trips, body length mismatch, CRC
  mismatch, footer/header mismatch, unexpected domain, partial/trailing files,
  writer and reader verifier rejection, Unix file permissions, and Unix symlink
  rejection.
- Resolved the first `0.18.0` pentest pass by documenting that current
  WAL/segment CRC and unkeyed digest checks are structural corruption checks
  rather than keyed tamper resistance, blocking in-repo production
  `SegmentContentVerifier` implementations until the admitted digest engine
  lands, syncing parent directories after Unix WAL/segment file creation, and
  adding batch world fact-list merge APIs to avoid future one-by-one insertion
  pressure on high-volume write paths.

## Verification

- `cargo test -p skrifheim-storage`
- `cargo test -p skrifheim-storage-host`
- `scripts/validate-modularity-policy.sh`
- `scripts/checks.sh`
- `scripts/release_0_18_gate.sh`

## Non-Claims

This release does not add compaction, manifests, segment indexes, fact
deserialization, signature verification for stored facts, production SHA-3/SHAKE
hash computation, authenticated encryption, multi-segment reading, or recovery
from immutable segments. Segment bodies remain encrypted opaque bytes at this
layer. The host reader and writer require an injected content verifier, but the
production digest implementation is still a planned storage-kernel milestone.

## Pentest Status

The first `0.18.0` pentest pass has been resolved locally. Root `PENTEST.md`
has been removed after findings were resolved.

Residual planned boundary: production tamper resistance for WAL and segment
files still requires AEAD authentication, signed/keyed manifests, or an
equivalent admitted integrity root before local disk-write attackers are in
scope.

This release is waiting for the permanent pentest digest commit, local release
gate, and GitHub verification before signed tag creation.
