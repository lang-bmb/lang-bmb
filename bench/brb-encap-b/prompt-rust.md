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
- The library is `src/lib.rs`. Only the standard library is available to the
  library code.
- For tests, `proptest` is available as a dev-dependency (property-based
  testing: `use proptest::prelude::*;` with `proptest! { .. }`), when the
  crate's `Cargo.toml` lists it. Unit tests (`#[cfg(test)] mod tests`) and
  integration tests (`tests/*.rs`) both work.

## How to work
The crate holds a data structure: a type with private fields, an invariant
stated in its documentation, its existing methods and their tests. Add the
method the task asks for to the type in `src/lib.rs`. Do not change the
type, its fields, its invariant or the existing methods, and keep the type
at the crate root. After the session your method is run in hidden
sequences of operations on the structure, together with the existing ones,
starting from an empty structure: every result must be the one the task
describes, and the type's invariant must hold after every operation (the
hidden judge reads the fields). A panic (including integer overflow or
out-of-bounds indexing) counts as a failure, and the crate must build. Write
tests that check the behaviour (property-based tests are encouraged), run
them with `cargo_test`, and fix the code until they pass. Finish when you are
confident the method is correct and the tests pass.
