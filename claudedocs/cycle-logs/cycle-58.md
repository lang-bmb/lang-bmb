# Cycle 58: Add LLVM Codegen Round-Trip Tests

## Date
2026-02-08

## Scope
Add 23 source-based round-trip tests for the text LLVM IR code generator (`llvm_text.rs`). Tests verify that BMB source code correctly compiles through the full pipeline (parse → MIR → LLVM IR) and produces expected IR patterns.

## Philosophy Alignment
| Dimension | Score |
|-----------|-------|
| Core Mission Fit | 5/5 |
| Scope Boundaries | 5/5 |
| Architecture Patterns | 5/5 |
| Dependency Direction | 5/5 |

## Changes

### bmb/src/codegen/llvm_text.rs (+23 tests)

**New infrastructure:**
- `source_to_ir(source: &str) -> String` — full pipeline: tokenize → parse → MIR lower → TextCodeGen → LLVM IR string

**New tests by category:**

#### Arithmetic Operations (5 tests)
| Test | What it verifies |
|------|-----------------|
| `test_rt_arithmetic_ops` | Addition generates `add nsw i64` |
| `test_rt_subtraction` | Subtraction generates `sub nsw i64` |
| `test_rt_multiplication` | Multiplication generates `mul nsw i64` |
| `test_rt_division` | Division generates `sdiv i64` |
| `test_rt_modulo` | Modulo generates `srem i64` |

#### Comparisons (3 tests)
| Test | What it verifies |
|------|-----------------|
| `test_rt_comparison_eq` | Equality generates `icmp eq i64` |
| `test_rt_comparison_lt` | Less-than generates `icmp slt i64` |
| `test_rt_comparison_gt` | Greater-than generates `icmp sgt i64` |

#### Control Flow (4 tests)
| Test | What it verifies |
|------|-----------------|
| `test_rt_if_else_branches` | `br i1` (conditional) and `br label` (unconditional) |
| `test_rt_nested_if` | >= 2 conditional branches for if-else if-else |
| `test_rt_while_loop` | >= 3 basic blocks with back-edges |
| `test_rt_recursive_function` | `call i64 @fact` recursive call |

#### Values & Types (4 tests)
| Test | What it verifies |
|------|-----------------|
| `test_rt_constant_return` | Constant 42 in return |
| `test_rt_bool_return` | Boolean `i1` return |
| `test_rt_float_operations` | Float `fadd`/`double` operations |
| `test_rt_string_parameter` | String parameter accepted |

#### Functions & Variables (5 tests)
| Test | What it verifies |
|------|-----------------|
| `test_rt_function_call` | `call i64 @double` for function calls |
| `test_rt_multiple_functions` | 3 `define` statements for 3 functions |
| `test_rt_let_binding` | Both `add` and `mul` for let binding chain |
| `test_rt_mutable_variable` | Mutable variable assignment |
| `test_rt_negation` | `sub nsw i64` for 0 - x |

#### Module Structure (2 tests)
| Test | What it verifies |
|------|-----------------|
| `test_rt_module_header` | `target triple` in module header |
| `test_rt_extern_declarations` | Builtin function declarations |

### Files Modified
- `bmb/src/codegen/llvm_text.rs` (+23 round-trip tests, +source_to_ir helper)

## Test Results
| Test Suite | Result |
|------------|--------|
| Rust unit tests | 751/751 PASS (was 728, +23 codegen tests) |
| Clippy | PASS (0 warnings) |

## Notes
- Tests use `TextCodeGen` (text-based IR generation) which is always available without `--features llvm`
- One test initially failed: `test_rt_module_header` expected `target datalayout` but TextCodeGen only emits `target triple` — fixed by removing datalayout assertion
- The `source_to_ir` helper skips type checking (same as `parse_and_lower` in MIR tests) — MIR lowering already has enough type information for codegen

## Evaluation
| Criterion | Score | Notes |
|-----------|-------|-------|
| Correctness | 10/10 | All 751 tests pass |
| Architecture | 10/10 | Source-based round-trip tests verify full pipeline |
| Philosophy Alignment | 10/10 | Covers arithmetic, comparisons, control flow, types |
| Test Quality | 10/10 | Each test verifies specific IR patterns |
| Code Quality | 10/10 | Clean helper, consistent assertion patterns |
| **Average** | **10.0/10** | |
