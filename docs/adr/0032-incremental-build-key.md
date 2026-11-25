# ADR-0032: The key of the contract firewall incremental build

- Status: Accepted (autonomous). The "query header" row of the proof key and its limit are superseded by ADR-0033. The "contract hash" of callee executable functions is superseded by ADR-0043 (the caller hash without `#[uses]`). The name dependence of the key rows (callees and source text written by path, byte offset positions) is superseded by ADR-0059 §4. The compiler identity inside the proof key and the IR key is superseded by ADR-0150 (it stays in the file header)
- Date: 2026-10-24
- Related: DESIGN.md §8.1, ADR-0017 §4, ADR-0019 §2, backlog 2D-01·2D-02·2D-10

## Context
The proof cache (2D-01) is keyed by the whole solver input text and skips only the solver call. Every run still encodes every definition by symbolic execution and rebuilds all the IR. DESIGN.md §8.1 requires "if the body changes but the contract stays the same, callers are not verified or compiled again". ADR-0019 §2 set only the starting point of the key (body hash, callee contract hashes, body hashes of the `spec fn`s that are unfolded).

## Decision
For each definition, two results are stored in the `defs` file of the cache directory (`--cache DIR`, MCP `.bmb-cache/`) (`src/incremental.rs`).

- **Proof result**: only for fully proved definitions. The status line figures (number of obligations, `rlimit`, `rlimit_max`) and the list of proved obligations (positions relative to the start of the definition). A failed definition is not stored and is always encoded and proved again (to emit the diagnostics as they are).
- **IR**: the `define` fragment of an executable function and the number of remaining checks.

The proof key is everything the encoding reads.

| Element | Reason |
| --- | --- |
| Compiler identity (`CARGO_PKG_VERSION` + the SHA-256 that `build.rs` makes from `src/**` and `toolchain.env`), Z3 version | If the encoding or the tools change, everything is invalid |
| Hash of the query header (options, seeds, `rlimit`, prelude, every `spec fn` declaration) | It goes into every query as is |
| The source text of the definition (from `fn` to the closing brace) and its qualified normal form (including the body) | The source text pins the obligation positions (byte offsets), and the qualified normal form pins name resolution |
| For each definition the qualified normal form calls by name: the contract hash for an executable function, the body hash for a `spec fn` that is unfolded by `#[uses]` or is in the same recursive component, and only the name for the remaining `spec fn`s (the declaration is in the header) | The same as the `calls`·`uses`·`names` edges of the `impact` query |
| The members of the recursive component | Calls inside the component get a `decreases` obligation |

The IR key is the compiler identity, the source text, the qualified normal form, the contract hashes of the executable functions it calls (argument and return types), and the proved obligations that code generation can use inside that definition (the result of `usable_proofs` of 1B-31, relative positions).

The file has the format version and the compiler identity in its header, and writes one latest entry per (kind, definition) on one line, with the SHA-256 of the fields attached to each line. A line that does not parse or whose hash does not match, and a file of another version or identity, are ignored and rebuilt. If a stored position is outside the definition's range or a diagnostic code is not in the list, it is not used as a hit either.

## Rationale
- The elements of the key point one by one at the inputs `verify_impl` reads while encoding one definition. The same key gives the same query text, so the verdict is the same as the proof cache's. `incremental::with_audit` recomputes even on a hit and compares the script hash, status and proof list, and `tests/incremental.rs` confirms 0 mismatches over the whole corpus and under edits (shifting positions with a comment, adding a `spec fn`, adding a sub-unit, changing a measure).
- The IR key includes the list of usable proofs, so if another definition of the same call component fails and checks come back (1B-31), the IR of that component is rebuilt.
- Alternative: a key made only from body and contract hashes hits even when the source positions differ, so obligation positions drift, and it misses a change of the header (all `spec fn` declarations). Rejected.
- Alternative: always encode and reuse only the IR. Safe, but the encoding cost remains. The sufficiency of the key can be tested in audit mode, so rejected.

## Consequences
- On a generated program of 1200 definitions (750 executable functions), a warm `verify` is about 4 times faster than with the proof cache alone (the measurement is the evidence of backlog 2D-10).
- Changing the signature of a `spec fn`, or adding or removing one, changes the query header and every definition is encoded again. `impact` picks only the definitions that call it by name, but the query text really does change for all (the proof cache also misses all). Reducing the header to the declarations a query reaches is an encoding change and a separate task.
- The source text is in the key, so a whitespace or comment change inside a definition also encodes that definition again (the proof is caught by the proof cache). An edit outside the definition only shifts positions and is reused.
- `--emit-smt` needs the scripts of all definitions, so it turns reuse off.
- The file is trusted as much as the source tree, like the proof cache. Corruption is guarded against, but a deliberately edited entry is not.
