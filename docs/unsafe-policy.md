# skrifheim Unsafe Policy

Status: policy

The initial scaffold forbids unsafe Rust.

`skrifheim` treats unsafe Rust as part of the trusted computing base. The military-security posture is strict: unsafe code is not admitted in core crates.

## Default Rule

Core library crates under `crates/` must use:

```rust
#![no_std]
#![forbid(unsafe_code)]
```

Unsafe is forbidden by default.

## Exception Rule

If a future feature truly cannot be implemented without unsafe code, it must be isolated before implementation:

- document the reason here first,
- create a dedicated boundary crate for the unsafe mechanism,
- keep the unsafe crate out of the core path until reviewed,
- expose only a narrow safe wrapper,
- add tests and failure-mode documentation,
- keep the safe core crates free of unsafe code.

Possible future reasons might include OS-specific direct I/O, memory-mapped storage, SIMD acceleration, or FFI to a reviewed cryptographic provider. These are not pre-approved.

## Required Documentation

Every unsafe block must include a `SAFETY:` comment covering validity, alignment, aliasing, lifetime, and concurrency assumptions. Every crate with unsafe code must have tests around the safe wrapper and a crate-level safety section.

## Current Unsafe Inventory

None.
