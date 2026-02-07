# Cycle 25: Codegen Round-Trip Integration Tests

## Date
2026-02-07

## Scope
Add integration tests that verify the full compilation pipeline (parse → type-check → MIR → optimize → LLVM IR text generation) produces correct output patterns.

## Philosophy Alignment
| Dimension | Score |
|-----------|-------|
| Core Mission Fit | 5/5 |
| Scope Boundaries | 5/5 |
| Architecture Patterns | 5/5 |
| Dependency Direction | 5/5 |

End-to-end codegen tests directly verify Performance > Everything — they catch optimization regressions at the IR level.

## Research Summary
- **TextCodeGen::generate()** takes `MirProgram` → `String` of LLVM IR
- **IR format**: Functions are `define private` with attributes (`alwaysinline nounwind willreturn mustprogress memory(none)`)
- **ConstantPropagationNarrowing**: Narrows i64 constants to i32 + `sext` in IR
- **TailRecursiveToLoop**: Completely eliminates tail calls, replacing with `bb_loop_header` + `phi` + backedge
- **Contract optimization**: Stores constant `i1 1` for eliminated checks
- **DCE**: Removes unused computations, function body reduces to direct `ret`

## Implementation
- **integration.rs**: Added 2 helpers + 12 tests
  - `source_to_ir()`: Full optimized pipeline (Release level)
  - `source_to_ir_unopt()`: Unoptimized pipeline (for testing raw codegen)
  - Tests: function signature, bool return type, constant folding, string constants, branch structure, recursive call, tail-recursion-to-loop, multiple functions, f64 operations, contract elimination, dead code elimination, module header

## Test Results
- Rust tests: 413/413 passed (up from 401, +12 new codegen tests)
  - 260 unit tests (lib)
  - 130 integration tests (up from 118)
  - 23 gotgan tests
- Bootstrap: Stage 1 verified (from Cycle 24)

## Evaluation
| Criterion | Score | Notes |
|-----------|-------|-------|
| Correctness | 10/10 | All 413 tests pass |
| Architecture | 9/10 | Clean helper functions, tests use both optimized and unoptimized pipelines |
| Philosophy Alignment | 10/10 | IR-level verification catches optimization regressions |
| Test Quality | 9/10 | Tests verify optimization effects (constant folding, TCO→loop, contract elimination, DCE) |
| Documentation | 9/10 | Clear test names and assertions |
| Code Quality | 9/10 | Consistent structure, focused assertions |
| **Average** | **9.3/10** | |

## Issues
- I-01 (L): Tests use string matching on IR text which is fragile to formatting changes. Acceptable for now since TextCodeGen output is stable.

## Next Cycle Recommendation
Update ROADMAP.md and VERSION for v0.89 quality gate progress.
