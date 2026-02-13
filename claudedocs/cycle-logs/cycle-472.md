# Cycle 472: Profile Stage 1 Execution — Identify Performance Hotspots

## Date
2025-02-12

## Scope
Profile the bootstrap compiler's Stage 1 emit-ir (2.34s → compiler.bmb) to identify
the top performance bottlenecks driving the 4.7x gap vs the Rust compiler (0.50s).

## Philosophy Alignment
| Dimension | Score |
|-----------|-------|
| Core Mission Fit | 5/5 |
| Scope Boundaries | 5/5 |
| Architecture Patterns | 5/5 |
| Dependency Direction | 5/5 |

## Research Summary
Added `time_ns()` instrumentation to `compile_program` and `lower_program_inner_sb`
in compiler.bmb. Built Stage 1 with instrumentation, ran it on compiler.bmb, collected
timing data, then reverted all instrumentation.

## Implementation — Profiling Results

### Phase-Level Breakdown (compile_program)

| Phase | Time (ms) | % of Total | Description |
|-------|-----------|-----------|-------------|
| `parse_source` | 43 | 1.9% | Lexing + parsing |
| `build_struct_reg` | 1 | ~0% | Struct registry |
| **`lower_program_sb`** | **1,774** | **78.8%** | **AST → MIR lowering** |
| `optimize_const` | 1 | ~0% | Const inlining |
| `gen_runtime_decls` | 0 | ~0% | Runtime declarations |
| `strings+globals` | 30 | 1.3% | String literal collection |
| `collect_string_fns` | 1 | ~0% | String fn collection |
| **`gen_program_sb`** | **396** | **17.6%** | **MIR → LLVM IR generation** |
| **TOTAL** | **2,250** | 100% | |

### Per-Function Lowering (>10ms threshold)

| Rank | Function | Lower (ms) | Root Cause |
|------|----------|-----------|------------|
| 1 | `llvm_gen_conc_rhs` | 202 | Largest function in codebase |
| 2 | `map_runtime_fn` | 92 | ~50+ if/else string comparisons |
| 3 | `llvm_gen_rhs_with_strings_map_and_fns` | 90 | Large codegen function |
| 4 | `llvm_gen_rhs_with_strings_map_and_fns_reg` | 89 | Register variant |
| 5 | `do_step` | 60 | Trampoline step processor |
| 6 | `step_expr` | 55 | Expression step handler |
| 7 | `llvm_gen_rhs_typed_reg` | 37 | Typed RHS codegen |
| 8 | `llvm_gen_rhs_typed` | 35 | Typed RHS codegen |
| 9 | `llvm_gen_conc_stmt` | 26 | Statement codegen |
| 10 | `get_call_arg_types` | 18 | Arg type resolution |

- **853 total functions** processed
- Top 14 functions (>10ms): ~771ms (43% of lowering)
- Remaining 839 functions: ~1,008ms (57%), avg ~1.2ms each

### Architecture Analysis — Why Lowering is 79%

The `lower_program_sb` function uses a **trampoline pattern** (`trampoline_v2`) to avoid
stack overflow. Each trampoline step involves:

1. `bmb_arena_save()` — save arena state
2. `pop_work_item()` — scan work stack string for tab separator, slice
3. `pop_work_rest()` — scan for tab, slice rest
4. `do_step()` — process work item (actual lowering work)
5. `step_temp()` — parse pipe-separated result string (1st pipe)
6. `step_block()` — parse 2nd pipe
7. `sb_new()` — create exit_sb StringBuilder
8. `step_exit_label()` — parse 3rd pipe
9. `sb_new()` — create work_sb StringBuilder
10. `step_work()` — parse 4th pipe
11. Work stack combining logic
12. `bmb_arena_restore()`
13. `sb_build(exit_sb)` + `sb_free(exit_sb)`
14. `sb_build(work_sb)` + `sb_free(work_sb)`
15. Recursive call

**17 operations per step**, estimated ~25,000+ total steps for compiler.bmb.
~70 microseconds per step, dominated by:
- 2 StringBuilder create/build/free cycles per step
- 4 pipe-finding string scans per step
- Arena save/restore per step
- String encoding/decoding of step results

### Root Cause Summary

The 4.7x gap comes from the bootstrap compiler's **string-based architecture**:

1. **All data structures are strings** — AST (S-expressions), MIR, work items,
   step results, work stacks, registries — everything is string-encoded
2. **Trampoline overhead** — 2 StringBuilders + arena save/restore per step
3. **Step result encoding** — `make_step()` packs 4 fields into pipe-separated
   string, immediately parsed back by 4 separate scans
4. **No native data structures** — Rust uses enums/structs/vectors; bootstrap
   simulates them with string encoding/decoding

## Test Results
| Item | Result |
|------|--------|
| Rust tests | 5,229 passed |
| Golden tests | 17/17 PASS |
| Stage 1 build | SUCCESS |
| IR line count | 68,993 (matches fixed point) |
| Instrumentation | Fully reverted, zero diff |

## Evaluation
| Criterion | Score | Notes |
|-----------|-------|-------|
| Correctness | 10/10 | All instrumentation reverted, tests pass, no code changes |
| Architecture | 9/10 | Thorough phase+function-level profiling |
| Philosophy Alignment | 10/10 | Direct performance investigation per BMB principles |
| Test Quality | 9/10 | Verified with all test suites after revert |
| Documentation | 10/10 | Detailed profiling data with analysis |
| Code Quality | 10/10 | Clean revert, no residual changes |
| **Average** | **9.7/10** | |

## Issues & Improvements
| # | Severity | Description | Action |
|---|----------|-------------|--------|
| I-01 | H | Trampoline creates 2 SBs per step (~25K steps) | Cycle 473: Optimize trampoline |
| I-02 | H | Step result string encoding parsed 4x per step | Cycle 473: Integer packing |
| I-03 | H | Arena save/restore per step overhead | Cycle 473: Batch arena ops |
| I-04 | M | gen_program_sb takes 396ms (18%) | Cycle 474+: Profile codegen |
| I-05 | M | map_runtime_fn: 50+ string comparisons per call | Consider hash-based lookup |
| I-06 | L | Large codegen functions dominate lowering time | Natural consequence of function size |

## Next Cycle Recommendation
- Cycle 473: Optimize `trampoline_v2` — reduce per-step overhead by eliminating
  redundant SB creation and optimizing step result encoding
- Focus on the 17 operations per trampoline step → target ≤10 operations
- Consider integer packing for step results (temp|block can be packed as single i64)
