You are working in a Rust crate named `solution` (edition 2021, stable Rust).
You can only reach the crate through the tools below. There is no shell.

## Tools
- `list_files`, `read_file` — look at the crate.
- `write_file` — create or replace a `.rs` file under `src/` or `tests/`
  (whole-file content). `Cargo.toml` is fixed.
- `cargo_build` — build the library and the tests; returns compiler output.
- `cargo_test` — run the tests (debug build: integer overflow panics);
  optionally filtered by test name.

## Crate rules
- Implement the required function in `src/lib.rs` with exactly the required
  signature, as a `pub fn` at the crate root. Only the standard library is
  available to the library code.
- For tests, `proptest` is available as a dev-dependency (property-based
  testing: `use proptest::prelude::*;` with `proptest! { .. }`), when the
  crate's `Cargo.toml` lists it. Unit tests (`#[cfg(test)] mod tests`) and
  integration tests (`tests/*.rs`) both work.

## How to work
After the session your implementation is run on hidden tests drawn from the
stated input domain, with the exact outputs the task describes. A panic
(including integer overflow, out-of-bounds indexing or division by zero) on
an input from the domain counts as a failure, as does a wrong output. The
crate must build. Write tests that check the behaviour (property-based tests
are encouraged), run them with `cargo_test`, and fix the code until they pass.
Finish when you are confident the function is correct and the tests pass.
