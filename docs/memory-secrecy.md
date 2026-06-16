# skrifheim Memory Secrecy

Status: scaffolded in `v0.12.0`

`skrifheim` treats secret memory as an explicit boundary. Key material, bearer
tokens, seed material, recovery secrets, and future hardware-provider handles
must not be stored in ordinary `Vec<u8>` or `String` fields once they enter the
crypto control plane.

## Current Boundary

`skrifheim-crypto` exposes `SecretBytes` as the first secret-owning wrapper.
It is intentionally narrow:

- non-empty and bounded by `SECRET_VALUE_MAX_BYTES`,
- no `Clone`, `PartialEq`, or `Eq`,
- redacted `Debug`,
- closure-only read access through `with_secret`,
- explicit `clear_secret` and `into_cleared`,
- clear-on-drop storage through the admitted `sanitization` crate.

`SecretBytes::from_slice` copies from a borrowed source slice and cannot clear
that source memory. Callers that own secret bytes in a `Vec<u8>` should prefer
`SecretBytes::from_vec`, which transfers the allocation into clear-on-drop
storage instead of creating a duplicate plaintext allocation.

The wrapper is not a key manager, KMS client, HSM handle, memory-locking
system, or cryptographic verifier. It is the in-process ownership rule that
future key and secret APIs must use before durable encryption paths exist.

## Dependency Admission

`sanitization` `1.1.0` is admitted for this boundary with:

```toml
sanitization = { version = "1.1.0", default-features = false, features = ["alloc"] }
```

The selected feature set keeps `skrifheim` on no-std library crates, avoids
derive macros, avoids serde, avoids zeroize/subtle interop, and avoids
platform memory-locking features. It gives `skrifheim` a safe wrapper around a
documented volatile wipe boundary without adding unsafe Rust to the core
crates.

## Required Rules

- Do not derive or implement `Clone`, `PartialEq`, or `Eq` for secret-owning
  wrappers.
- Do not expose `as_slice`, `to_vec`, `into_vec`, or raw byte accessors from
  secret-owning wrappers.
- Debug output for secret-owning wrappers must redact contents and exact byte
  length.
- Error messages must be static and must never include secret bytes, key IDs
  that are themselves secret, or caller-provided secret labels.
- Rejected owned secret buffers must still be cleared before drop.

## Non-Claims

This scaffold does not protect against process dumps, privileged memory reads,
swap, hibernation, CPU caches, crash aborts, optimizer-visible historical stack
copies, or host runtime snapshots. Platform memory locking, guard pages,
hardware-backed handles, and statistical/codegen evidence belong to later
hardening milestones.
