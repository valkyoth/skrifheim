# skrifheim 0.16.0 Release Notes

Status: implementation stop, pending pentest.

## Scope

`0.16.0` adds the first WAL replay and recovery-state scaffold. Replay is still
header-driven and does not decrypt WAL bodies, verify body CRCs, rebuild fact
state, or perform durable startup recovery. It establishes the transaction
shape rules the future recovery loader must obey before accepting committed WAL
state.

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
- Bumped workspace and internal crate dependency versions to `0.16.0`.
- Added `scripts/release_0_16_gate.sh`.

## Verification

- `cargo info sanitization`
- `cargo test -p skrifheim-storage`
- `scripts/checks.sh`
- `scripts/release_0_16_gate.sh`

## Non-Claims

This release does not decrypt WAL bodies, verify encrypted-body CRCs against
body bytes, replay fact payloads into database state, restore from immutable
segments, load manifests, perform checkpoint recovery, or execute startup
recovery. It also does not add a production digest implementation. WAL replay
reports are transaction-shape metadata for the current scaffold, not proof that
the encrypted bodies are authentic or semantically valid.

## Pentest Status

This is the `0.16.0` implementation stop and is ready for pentest. Root
`PENTEST.md` remains the temporary findings handoff file and must be removed
after findings are resolved.
