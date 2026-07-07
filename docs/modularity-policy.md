# skrifheim Modularity Policy

Status: policy

`skrifheim` must never grow into one huge source file or one unreviewable crate.

## Rules

- The main `skrifheim` crate is orchestration only.
- Major subsystems live in focused crates.
- Application-family primitives live in optional extension crates when they are
  not required by the mandatory database core.
- Core crates must not depend on extension crates; extension crates prove the
  core APIs are generic enough by depending on core crates.
- Every extension primitive must be classified before implementation as
  mandatory core, generic extension helper, or product-owned schema.
- `lib.rs` wires modules and exports APIs; it does not hold subsystem implementation.
- `main.rs` starts the process and delegates behavior.
- Parsing, validation, state mutation, I/O, policy checks, and tests stay separate.
- Pure logic should be host-testable without a server.

## File Size

Target:

- normal implementation files: 300 lines or less,
- hard limit: 500 lines for non-generated `.rs` files,
- generated files must be clearly marked and isolated.

Current exceptions: none.

The gate is `scripts/validate-modularity-policy.sh`.
