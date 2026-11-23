# BMB

BMB is a programming language written by AI agents and handled with deterministic tools. Its shape follows how an agent works: an agent reads code through a map and definition-sized queries instead of files, writes one definition at a time, and gets a proof or a counterexample back instead of a test run.

- **Contracts are proved, not tested.** `requires` and `ensures` go to a proof gate (Z3). A build succeeds only when every obligation is proved: contracts, arithmetic overflow, bounds, termination. An undecided obligation or a timeout is a failure.
- **Tools instead of files.** The compiler serves a map of the program (units, definitions, call edges, effects) and reads and writes at the level of a definition, over a command line and an MCP server.
- **One compiler, written from scratch.** Parsing, checking, proof encoding and LLVM IR generation are its own code. The only external programs it runs are Z3 and clang.
- **The surface inherits from Rust and Verus.** Models have not been trained on BMB, and a familiar surface makes up for that. The semantics are stricter than either.

## Status

BMB is at version 0.x and is not ready for use outside its own development. This is version 2, restarted from nothing; the earlier line (v0.100, where verification was optional) is archived at [`lang-bmb-legacy-v0`](https://github.com/lang-bmb/lang-bmb-legacy-v0) and what is written about it does not apply here.

| Area | State |
| --- | --- |
| Language surface | Frozen at `0.1.0`. One exception since the freeze: `clone` for values that own memory |
| Proof gate | No unproved definition in the corpus, the standard library or the port |
| Standard library | 11 units, all proved: `base64`, `bytes`, `collections`, `crc32`, `intern`, `json`, `num`, `sha256`, `slice`, `str`, `utf8` |
| Self-hosting | The front half of the compiler (lexer, parser, checker, normal form, identity hash, code generation: about 42%) is ported to BMB, every definition proved, output byte-identical to the Rust compiler. Built by itself it reaches a fixed point (stage 1 IR = stage 2 IR = stage 3 IR), checked on every commit |
| Performance | 7 of 7 baseline benchmarks within ±5% of safe Rust; stripped binaries about 1/20 the size |
| Claims about agents | Measured, with what was refuted: see [`docs/CLAIMS.md`](docs/CLAIMS.md) |
| Ecosystem | Closed by design (no packages, no bindings) until the claims it rests on are measured |

The plan and where the work stands on it are in [`docs/ROADMAP.md`](docs/ROADMAP.md).

## Documents

| Document | Content |
| --- | --- |
| [`docs/DESIGN.md`](docs/DESIGN.md) | The settled design: definitions, premises, principles, language surface |
| [`docs/reference/`](docs/reference/) | Language reference and the conformance tests that follow it |
| [`docs/ROADMAP.md`](docs/ROADMAP.md) | Stages, decision gates, current position, plan |
| [`docs/CLAIMS.md`](docs/CLAIMS.md) | Each claim, its state, the evidence |
| [`docs/decisions/`](docs/decisions/) | Decision records (ADRs) |
| [`docs/diagnostics.md`](docs/diagnostics.md) | Diagnostic codes |
| [`compiler/README.md`](compiler/README.md) | Using the compiler; surface rules |

Benchmarks, their harnesses, frozen task sets, result rows and measurement reports are in [`bmb-bench`](https://github.com/lang-bmb/bmb-bench).

## Build

Requires a Rust toolchain, Z3 and LLVM (clang). The versions are pinned in `scripts/toolchain.env`; `scripts/setup.sh` installs them.

```sh
scripts/setup.sh
cargo build --release
target/release/bmb doctor
target/release/bmb verify tests/run/fn_values.bmb
```

`bmb` with no arguments lists the commands.

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
