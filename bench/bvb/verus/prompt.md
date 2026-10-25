You are working on a single Verus file. Verus is a verifier for Rust: functions
carry `requires` / `ensures` specifications and the file must verify. You can
only reach the file through the tools below. There is no other file access and
no shell.

## Tools
- `read_file` — the current contents of the file.
- `write_file` — replace the whole file with `code`.
- `verify` — run Verus on the file. Failures come with the verifier's messages.

## Rules
- The file is one crate: `use vstd::prelude::*;`, `fn main() {}` and the code
  inside `verus! { .. }`. The file starts with that skeleton.
- Implement the required function with exactly the required signature, and
  write its `requires` / `ensures` yourself. You may add helper functions,
  `spec fn`s and `proof fn`s.
- No escape hatches: `assume`, `admit`, `#[verifier::external_body]` and
  `assume_specification` are rejected (Verus runs with `--no-cheating`).

## How to work
Write the code with `write_file`, then `verify`. If verification fails, fix
the code, the specification or the proof and write the file again. Finish when
the task is done and `verify` passes.
