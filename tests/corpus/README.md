# Compiler test corpus from the benchmarks

Benchmark seeds and reference programs, kept here as a test corpus of the compiler: the tests prove them, check that no proof is vacuous, write them through the JSON syntax tree, build and run them, and the differential tests of the port read them as inputs. The benchmarks themselves (harnesses, task sets, results and reports) live in the [`bmb-bench`](https://github.com/lang-bmb/bmb-bench) repository.

The files are copies, byte for byte, and are not edited here. Their summaries and comments are in the language they were written in.

| Directory | What | Read by |
| --- | --- | --- |
| `baseline/` | The seven programs of the performance baseline (BMB side) | `compiler/tests/verify.rs` |
| `seed/m1/src/`, `seed/m1b/src/`, `seed/bvb/src/` | The seed workspaces of three writing benchmarks: each `src` directory is one program | `compiler/tests/verify.rs`, `compiler/tests/ast_json.rs` |
| `seed/bvb/tasks/` | One task of the third, whose hidden acceptance check a proof test rebuilds | `compiler/tests/verify.rs` |
| `encap/` | The seed unit of each task of the encapsulation benchmark, one directory per task | `compiler/tests/verify.rs` |
| `game/` | Four Benchmarks Game programs, the output each prints (`*.out`), and single-threaded Rust references for two of them (`ref/`) | `compiler/tests/verify.rs`, `compiler/tests/codegen.rs` |

`scripts/cross_solver.py` and `scripts/solver_stability.py` take their programs from here as well.
