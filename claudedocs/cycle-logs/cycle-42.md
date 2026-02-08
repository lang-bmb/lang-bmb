# Cycle 42: Fix 4 Compiler/Interpreter Issues

## Date
2026-02-08

## Scope
Fix 4 bugs discovered during Cycles 37-41 dogfooding: float/int `==` mismatch, `free()` returns Unit, codegen `%tmp` undefined, `let` scope in blocks.

## Philosophy Alignment
| Dimension | Score |
|-----------|-------|
| Core Mission Fit | 5/5 |
| Scope Boundaries | 5/5 |
| Architecture Patterns | 5/5 |
| Dependency Direction | 5/5 |

Per CLAUDE.md Principle 2: "Workaround는 존재하지 않는다" - these are compiler bugs that must be fixed at the proper level.

## Research Summary
- Analyzed existing `compare_values()` function for coercion pattern (eval.rs)
- Reviewed `store_i64`/`store_u8`/`store_f64` codegen handlers for `free()` handler pattern (llvm.rs)
- Studied SSA naming in `Expr::Let` handler to understand scope restoration (lower.rs)
- Examined ecosystem packages for workaround patterns to clean up

## Implementation

### 1. Float/int `==` mismatch (eval.rs)
- `BinOp::Eq` and `BinOp::Ne` now coerce `Int↔Float` to f64 before comparing
- Matches existing behavior of `Lt/Gt/Le/Ge` via `compare_values()`
- `PartialEq for Value` left strict (struct/array/tuple comparison unaffected)

### 2. `free()` returns i64 (types/mod.rs, eval.rs, llvm.rs)
- Type system: `free`, `store_i64`, `store_f64`, `store_u8`, `store_i32`, `box_set_i64`, `box_free_i64`, generic `free<T>` all changed from `Unit` to `I64`
- Interpreter: `builtin_free`, `builtin_store_i64`, `builtin_store_f64` return `Value::Int(0)`
- Codegen: Added specialized `free` handler (calls void C free(), stores i64 0 to dest)

### 3. `let` scope in blocks + codegen `%tmp` (lower.rs, mod.rs, llvm.rs)
- Added `last_let_binding` field to `LoweringContext` for Block↔Let communication
- `Expr::Let` records binding before restoring old mapping
- `Expr::Block` re-inserts let bindings after each Let expression; restores at block end
- `Expr::Assign` resolves through `var_name_map` for SSA-unique names
- Codegen `load_from_place`: fallback tries `{name}_v*` SSA variants

### 4. Ecosystem cleanup
- bmb-math: Simplified `test_sqrt()` from `>=`/`<=` workaround to direct `==`
- bmb-sha256: Simplified `drop()`/`st()` wrappers to direct `free()`/`store_i64()` calls
- bmb-hashmap: Simplified `free_and_zero()` to direct `free()` call

### Files Modified
- `bmb/src/interp/eval.rs` (Steps 1, 2b)
- `bmb/src/types/mod.rs` (Step 2a)
- `bmb/src/codegen/llvm.rs` (Steps 2c, 3e)
- `bmb/src/mir/lower.rs` (Steps 3b, 3c, 3d)
- `bmb/src/mir/mod.rs` (Step 3a)
- `ecosystem/gotgan-packages/packages/bmb-math/src/lib.bmb`
- `ecosystem/gotgan-packages/packages/bmb-sha256/src/lib.bmb`
- `ecosystem/gotgan-packages/packages/bmb-hashmap/src/lib.bmb`
- `claudedocs/issues/ISSUE-20260207-interp-float-int-eq.md` (RESOLVED)
- `claudedocs/issues/ISSUE-20260207-free-returns-unit.md` (RESOLVED)
- `claudedocs/issues/ISSUE-20260207-let-in-while-block.md` (PARTIALLY RESOLVED)

## Test Results
| Test Suite | Result |
|------------|--------|
| Rust unit tests | 694/694 PASS |
| Clippy | PASS (0 warnings) |
| bmb-math | 12/12 PASS |
| bmb-sha256 | 9/9 PASS |
| bmb-hashmap | 9/9 PASS |
| bmb-hash | 8/8 PASS |
| bmb-tree | 8/8 PASS |
| All ecosystem | 18/18 packages PASS |

## Evaluation
| Criterion | Score | Notes |
|-----------|-------|-------|
| Correctness | 9/10 | All 4 bugs fixed, all tests pass. Parser grammar for let-in-while still pending. |
| Architecture | 9/10 | Fixes follow existing patterns (compare_values, store handlers, SSA naming) |
| Philosophy Alignment | 10/10 | Root cause fixes, no workarounds, proper level in Decision Framework |
| Test Quality | 7/10 | No new Rust unit tests for the specific fixes; verified via ecosystem tests |
| Documentation | 8/10 | Issue files updated, ecosystem comments cleaned up |
| Code Quality | 9/10 | Clean, minimal changes, well-commented with version tags |
| **Average** | **8.7/10** | |

## Issues & Improvements
| # | Severity | Description | Action |
|---|----------|-------------|--------|
| I-01 | H | Parser grammar still restricts `let` in while/for/loop bodies | Cycle 43 |
| I-02 | M | No dedicated Rust unit tests for float/int Eq coercion | Add in future cycle |
| I-03 | L | Codegen `load_from_place` fallback uses linear scan over ssa_values | Acceptable for correctness; optimize if profiled |
| I-04 | M | bmb-sort/bmb-ptr still cannot run in interpreter (*i64 typed pointers) | Cycle 45 |

## Next Cycle Recommendation
**Cycle 43**: Fix grammar.lalrpop to allow `let` bindings inside while/for/loop blocks. This is the parser-level root cause of Issue ISSUE-20260207-let-in-while-block. The MIR scope fix from this cycle ensures the bindings will work correctly once the parser accepts them.
