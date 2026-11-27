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
- The port's `emit-llvm` and `build` run the proof gate as the Rust compiler does: without `--all-checks` they prove the program (through `--cache` when given), drop the runtime checks whose proofs are usable and keep the others, take the IR of unchanged definitions from the incremental cache and store what they lower, with the same `--stats`; `build` refuses a program that is not fully proved and builds the executable with clang.
- The port has `bmb map` and `bmb query`: the standard maps L0, L1 and L2 and every read query, with `--cache` and the window flags, byte-identical to the Rust compiler's.
- The port has `bmb import` and `bmb apply`: the workspace and its transactions (`unit`, `put`, `remove`, `remove_unit`, `rename`, `expect`, `move`), with the same answer and the same files as the Rust compiler's, and its commands read a workspace directory among their files.
- Diagnostic texts no longer carry internal task identifiers. Six retired diagnostics and two help texts changed wording; the codes are unchanged.
- An `assume` inside an expression (in an argument of a call, an operand, the value of a `return`) is counted as an escape: the row of `bmb verify` carries it in `escapes`, and the maps and queries show its rationale. It was marked `"trust":"assumed"` when the proof needed it, but not counted.
- An integer literal of a specification above the largest `i128` keeps its value in a proof query. It was read as a negative number (`2^128 - 1` as `-1`), so a false assertion about it could be proved.
- A sub-proof `assert(e) by (bit_vector)` that passes a plain variable to a spec fn parameter of a narrower type is proved; it ended in a solver error.
- `bmb dump smt-queries` prints, without running the solver, the queries `bmb verify --emit-smt` writes of each definition before the first answer, with their legends.
- `BMB_Z3_RLIMIT`, `BMB_Z3_MEMORY_MB` and `BMB_Z3_BACKSTOP_MS` set to 0 give the smallest limit, 1. Z3 reads 0 as no limit, so a variable that may only lower a limit removed it: `bmb verify` under `BMB_Z3_RLIMIT=0` proved with no resource limit. The probe's share of a resource limit lowered below 100 is 1 for the same reason.
- A query that is asked again alone after its process ended at the memory limit, and then ends its own process there at a later command, is undecided at once. It was asked a third time, in another process under the same limit.
- The incremental cache (`--cache` without `--emit-smt`) places the proved obligations of a definition that reuses another definition's entry (the same proof key: a rename, or the same text calling by a bare name inside its unit and by a path from another) on that definition's own expressions. A path counted one position per segment, so the proofs landed on other expressions: the IR kept a check that an uncached run drops (`bmb build --cache` refused a fully proved program), and could drop one whose obligation was not proved there.
- An answer of the solver that has not ended after 1,000,000 lines is undecided (E0811, reason `answer limit reached, 1000000 lines`), and the solver process that wrote it is ended; what a process writes after `(exit)` is read for as many lines at most. A solver that wrote without end kept `bmb verify` reading forever.
- The keys of the incremental cache no longer hold the compiler's identity, which the file's header names: a cache file of another build is still not read at all, and the key `bmb reads` prints of a definition no longer changes with each build of the compiler.
- A query the wall-clock safeguard ends spends no resource units in the output: its units enter no `rlimit` or `rlimit_max` of a `bmb verify` status line, no cost of `lint` or `examples` and no row of `stability`, and a proof with such a query is not stored in the proof cache. The units it had spent by then depend on the clock, so the line differed from run to run (`"backstop"` still counts these queries).
- `bmb fmt --check` holds each file to the units it declares: a file passes when its text is the normal form of those units in name order, with one blank line between, and the check passes when every file does. It compared the files joined in the order given with all the units joined in name order, so two files in normal form passed in one order and failed in the other, and an empty file among the files made the check fail. `bmb fmt` without `--check` prints the same text as before.
- A parameter may have the name of one of the code generator's own labels or temporaries (`entry`, `t1`, `then1`): such a program passed the proof and then failed to build ("multiple definition of local value"). Parameter registers now carry a suffix no source name can have.
- Round trips of integers and their bytes prove: `T::from_le_bytes(x.to_le_bytes()) == x` and `T::from_le_bytes(b).to_le_bytes() == b` (and the big-endian forms) for every integer type, directly, through a local or a field, and in code also through an element-wise copy, a `Vec` and a slice at a variable offset (ADR-0157). They were undecided at the resource limit for a 64-bit value. A byte read by `from_*_bytes` is its own image without `mod` when it is known to be a byte, so a word a specification reads from a byte slice costs what the sum of the products costs. Code assumes a fact with each `to_*_bytes` and `from_*_bytes`, so the cost of a proof that takes bytes without reading them back grows by a few hundred units.
- A status line of `bmb verify` says what a verdict on a package needs in one run: `"escapes":n` for a definition with escape hatches (also when the proof did not need its `assume`, which left a plain `verified` line), and `"termination":"unproved"` for a definition with a `loop` without `decreases` or that calls one. They appear only where they apply; the line of any other definition is unchanged (ADR-0160).
- `#[trigger]` on a bound variable alone (`p(#[trigger] i)`, or `#[trigger] i < n`, which marks `i`) is a check error, E0517, with a hint to mark a term that applies a function or an index to the variable. `bmb check` accepted it and the solver then rejected the query (E0812 "invalid function application").
- A counterexample that rests on a spec fn the proof sees only by name says so and shows the values it assumed: `counterexample with `off` unknown: x = [] (len 0), n = 38, assuming off(x, n) = -1`. The inputs alone read as a counterexample and often were none under the function's definition; verdicts are unchanged (ADR-0161).
- A call of a `proof fn` asks the range of each argument in one obligation. A struct argument was asked once per integer field, each query with the whole context: a lemma that passed a struct of nine integers seven times had 63 such obligations, two thirds of its proof cost. The obligation count and the resource units of such a definition go down; a failing argument gives one diagnostic instead of one per field. Verdicts are unchanged.

### Standard library

- 11 units, all proved: `base64`, `bytes`, `collections`, `crc32`, `intern`, `json`, `num`, `sha256`, `slice`, `str`, `utf8`.
- The units are in the normal form, and the explanations that were comments are the `#[summary]` of the definitions they explain (ADR-0158). A map, `bmb query` and `bmb dump std` show these summaries for library definitions; `bmb fmt` of a program that holds a library file leaves it as it is. Definitions, identity hashes, proof obligations and their costs are unchanged; positions in the library's units (diagnostics, the offsets in query comments) moved with the text, and the identity digest of the compiler, which covers the embedded text, changed.

### Repository

- The comparisons of the port with the Rust compiler too long for the per-push gate run every night in CI (`tests/port/nightly/`, `scripts/port_nightly.sh` locally): the port verifying itself cold and warm, dumping its queries (3.7 GB) and emitting itself through the cache, and `verify`, the proof cache, `lint`, `examples` and `stability` over every file of the test and library sets. Until now they were each compared once by hand.
- The working records of the project (task lists, reports, measurement logs) moved out; this repository holds the product and its documents. Decision records are in `docs/adr/`.
- The benchmarks (harnesses, task sets, results, reports) moved to [`bmb-bench`](https://github.com/lang-bmb/bmb-bench). The programs the compiler's tests read stay as `tests/corpus/`; the gate no longer runs the harness self-checks, which are that repository's own check.
