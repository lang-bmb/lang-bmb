# Changelog

BMB has no releases yet. This file records what changes for a user of the compiler and the language; the reasons are in the decision records ([`docs/adr/`](docs/adr/)) and the state of the work is in [`docs/ROADMAP.md`](docs/ROADMAP.md).

The language surface carries its own version, apart from the compiler crate's.

## Unreleased

### Language surface `0.1.0` (frozen)

- The surface is frozen until the compiler builds itself. A change after the freeze is listed here with its reason.
- Since the freeze: `clone` is accepted for values that own memory (the self-hosting port needs it).

### Compiler

- The proof gate, the checker, the normal form, the identity hash and LLVM IR generation are the compiler's own code; it runs Z3 and clang as external programs.
- The front half of the compiler (lexer, parser, checker, normal form, identity hash, code generation) is ported to BMB in `selfhost/`, every definition proved, with output byte-identical to the Rust compiler; built by itself it reaches a fixed point.
- Diagnostic texts no longer carry internal task identifiers. Six retired diagnostics and two help texts changed wording; the codes are unchanged.
- An integer literal of a specification above the largest `i128` keeps its value in a proof query. It was read as a negative number (`2^128 - 1` as `-1`), so a false assertion about it could be proved.
- A sub-proof `assert(e) by (bit_vector)` that passes a plain variable to a spec fn parameter of a narrower type is proved; it ended in a solver error.

### Standard library

- 11 units, all proved: `base64`, `bytes`, `collections`, `crc32`, `intern`, `json`, `num`, `sha256`, `slice`, `str`, `utf8`.

### Repository

- The working records of the project (task lists, reports, measurement logs) moved out; this repository holds the product and its documents. Decision records are in `docs/adr/`.
- The benchmarks (harnesses, task sets, results, reports) moved to [`bmb-bench`](https://github.com/lang-bmb/bmb-bench). The programs the compiler's tests read stay as `tests/corpus/`; the gate no longer runs the harness self-checks, which are that repository's own check.
