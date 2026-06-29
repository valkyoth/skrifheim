# skrifheim 0.16.0 Release Notes

Status: implementation stop, pentest passed, GitHub verification pending.

## Scope

`0.16.0` adds the first WAL replay and recovery-state scaffold. Replay is still
header-driven and does not decrypt WAL bodies, rebuild fact state, or perform
durable startup recovery. It establishes the transaction shape rules the future
recovery loader must obey before accepting committed WAL state.

This release also updates CI checkout tooling to the `actions/checkout` `v7`
series and updates the admitted `sanitization` dependency to `1.2.2`.

## Changes

- Updated GitHub Actions checkout from `actions/checkout@v6` to
  `actions/checkout@v7`.
- Updated admitted `sanitization` from `1.1.0` to `1.2.2` with
  `default-features = false` and only the `alloc` feature.
- Added `WalReplay`, a header-driven WAL replay state machine in the
  `no_std` storage crate.
- Added `WalRecoveryReport` for replayed frame count, checkpoint count,
  committed transaction summaries, and rolled-back transaction summaries.
- Added `WalRecoveryOutcome` for clean replay and clean-EOF uncommitted-tail
  recovery.
- Added `WalReplayStop` so callers must distinguish clean EOF from a truncated
  frame stop.
- Added committed transaction summaries with transaction ID, frame count, fact
  batch count, key ID, crypto epoch, and encryption domain accessors.
- Added rolled-back transaction summaries for abort records and clean-EOF
  uncommitted tails.
- Rejected fact batches, commits, aborts, and checkpoints outside valid
  transaction shape.
- Rejected nested transaction begins and non-advancing transaction IDs.
- Rejected mismatched transaction ID, key ID, crypto epoch, and encryption
  domain inside an active transaction.
- Rejected truncated-frame replay stops instead of treating them as clean
  recovery.
- Added crash-matrix style replay tests for committed transactions, aborts,
  uncommitted tails, truncation, transaction shape errors, and crypto epoch
  mismatch.
- Added redacted Debug output for replay state and recovery transaction
  summaries.
- Resolved the first `0.16.0` pentest pass by adding CRC64-ECMA encrypted-body
  verification on WAL write/read paths, rejecting Unix symlink WAL paths with
  `O_NOFOLLOW`, rejecting all-zero segment content digests and zero segment
  CRC sentinels, rejecting WAL crypto epoch zero, bounding replay transaction
  summaries, redacting fact structural identifiers, moving segment integrity
  metadata to `ContentDigest`, and using fixed-width domain comparison in WAL
  replay.
- Resolved the second `0.16.0` pentest pass by applying the same Unix symlink
  and regular-file guard to WAL reads, adding reader symlink rejection tests,
  and defining a no-op fallback `O_NOFOLLOW` value for unlisted Unix targets so
  they compile without weakening supported Linux, Android, macOS, iOS, BSD,
  illumos, or Solaris behavior.
- Bumped workspace and internal crate dependency versions to `0.16.0`.
- Added `scripts/release_0_16_gate.sh`.

## Verification

- `cargo info sanitization`
- `cargo test -p skrifheim-crypto`
- `cargo test -p skrifheim-fact`
- `cargo test -p skrifheim-storage`
- `cargo test -p skrifheim-storage-host`
- `scripts/checks.sh`
- `scripts/release_0_16_gate.sh`

## Non-Claims

This release does not decrypt WAL bodies, cryptographically authenticate WAL
bodies, replay fact payloads into database state, restore from immutable
segments, load manifests, perform checkpoint recovery, or execute startup
recovery. It also does not add a production digest implementation. WAL replay
reports are transaction-shape metadata for the current scaffold; CRC64 catches
accidental corruption and simple body/header mismatch but is not a security
MAC, signature, or encryption authenticity proof.

## Pentest Status

The first and second `0.16.0` pentest passes have been resolved locally. The
follow-up pentest reported no remaining findings. Root `PENTEST.md` has been
removed after findings were resolved. This release is waiting for GitHub
verification before signed tag creation.
