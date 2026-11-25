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
- `bmb dump smt-queries` prints, without running the solver, the queries `bmb verify --emit-smt` writes of each definition before the first answer, with their legends.
- `BMB_Z3_RLIMIT`, `BMB_Z3_MEMORY_MB` and `BMB_Z3_BACKSTOP_MS` set to 0 give the smallest limit, 1. Z3 reads 0 as no limit, so a variable that may only lower a limit removed it: `bmb verify` under `BMB_Z3_RLIMIT=0` proved with no resource limit. The probe's share of a resource limit lowered below 100 is 1 for the same reason.
- A query that is asked again alone after its process ended at the memory limit, and then ends its own process there at a later command, is undecided at once. It was asked a third time, in another process under the same limit.
- The incremental cache (`--cache` without `--emit-smt`) places the proved obligations of a definition that reuses another definition's entry (the same proof key: a rename, or the same text calling by a bare name inside its unit and by a path from another) on that definition's own expressions. A path counted one position per segment, so the proofs landed on other expressions: the IR kept a check that an uncached run drops (`bmb build --cache` refused a fully proved program), and could drop one whose obligation was not proved there.
- The keys of the incremental cache no longer hold the compiler's identity, which the file's header names: a cache file of another build is still not read at all, and the key `bmb reads` prints of a definition no longer changes with each build of the compiler.

### Standard library

- 11 units, all proved: `base64`, `bytes`, `collections`, `crc32`, `intern`, `json`, `num`, `sha256`, `slice`, `str`, `utf8`.

### Repository

- The working records of the project (task lists, reports, measurement logs) moved out; this repository holds the product and its documents. Decision records are in `docs/adr/`.
- The benchmarks (harnesses, task sets, results, reports) moved to [`bmb-bench`](https://github.com/lang-bmb/bmb-bench). The programs the compiler's tests read stay as `tests/corpus/`; the gate no longer runs the harness self-checks, which are that repository's own check.
