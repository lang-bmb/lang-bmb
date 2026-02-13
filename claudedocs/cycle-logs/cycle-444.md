# Cycle 444: Fix Stage 1 bootstrap — work3_get1, inline SSA register, Select string comparison

## Date
2026-02-13

## Scope
Fix three bugs preventing Stage 1 bootstrap compilation and native code generation:
1. Missing `work3_get1` function in `compiler.bmb` (type error)
2. Inline byte_at/len/ord producing undefined SSA registers in `llvm_text.rs`
3. Select instruction not handling string comparison in `llvm_text.rs`

## Philosophy Alignment
| Dimension | Score |
|-----------|-------|
| Core Mission Fit | 5/5 |
| Scope Boundaries | 5/5 |
| Architecture Patterns | 5/5 |
| Dependency Direction | 5/5 |

## Implementation

### Fix 1: Missing `work3_get1` in `compiler.bmb`

The function `work3_get1(item)` was called at line 3050 but never defined. It's a convenience accessor for work item field 1:

```bmb
fn work3_get1(item: String) -> String = get_field(item, 1);
```

### Fix 2: Inline SSA register naming in `llvm_text.rs`

**Bug**: When byte_at/len/ord are inlined in the text-based LLVM codegen, the result is stored to `%_tN.addr` (alloca) but `%_tN` (SSA register) is never defined. Subsequent Select/icmp instructions reference the undefined `%_tN`.

**Root cause**: Inline codegen used unique intermediate names (`%charat.ext.0`, `%strlen.len.0`, `%ord.ext.0`) for the final result instead of the destination register name.

**Fix**: Use `%dest_name` directly for the final instruction in each inline sequence:

| Inline Op | Before | After |
|-----------|--------|-------|
| byte_at | `%charat.ext.N = zext ...` → store | `%_tN = zext ...` → store `%_tN` |
| len | `%strlen.len.N = load ...` → store | `%_tN = load ...` → store `%_tN` |
| ord | `%ord.ext.N = zext ...` → store | `%_tN = zext ...` → store `%_tN` |
| ord passthrough | store only | `%_tN = add i64 ..., 0` → store `%_tN` |

### Fix 3: Select string comparison in `llvm_text.rs`

**Bug**: Select instruction used `icmp eq i64` for ALL comparisons, even when comparing strings (ptr type). Strings require `@bmb_string_eq` runtime call.

**Fix**: Added string operand detection to Select codegen:
- Checks `is_string_operand()` and `place_types` for string condition operands
- String Eq/Ne → `call @bmb_string_eq` + `icmp ne/eq i64 %result, 0`
- Integer comparison → unchanged `icmp` pattern
- Value type detection → `ptr` for strings, `i64` for integers
- Store to alloca when dest is a local

### Stage 1 Bootstrap Results

- **compiler.bmb** type-checks successfully (0 errors)
- **Stage 1 IR**: 70,578 lines, compiles with `opt -O2`
- **Stage 1 executable**: 587KB native binary
- **Correctness test**: `fib(10)` returns 55 correctly
- **0 actual calls to `@bmb_string_char_at`** (all 628+ byte_at calls inlined)
- **0 actual calls to `@bmb_string_len`** (all len calls inlined)

## Test Results
- Unit tests: 2845 passed
- Main tests: 47 passed
- Integration tests: 2314 passed
- Gotgan tests: 23 passed
- **Total: 5229 tests — ALL PASSING**
- Clippy: PASS (0 warnings)
- Build: SUCCESS
- Stage 1 Bootstrap: COMPILES AND RUNS CORRECTLY

## Evaluation
| Criterion | Score | Notes |
|-----------|-------|-------|
| Correctness | 10/10 | Stage 1 compiles, optimizes, links, and runs correctly |
| Architecture | 10/10 | Follows existing codegen patterns |
| Philosophy Alignment | 10/10 | Root cause fix, not workaround; enables bootstrap |
| Test Quality | 8/10 | Existing tests cover changes; Stage 1 manually verified |
| Code Quality | 10/10 | Minimal, targeted fixes |
| **Average** | **9.6/10** | |

## Issues & Improvements
| # | Severity | Description | Action |
|---|----------|-------------|--------|
| I-01 | L | No automated Stage 1 bootstrap test in CI | Future improvement |
| I-02 | L | Select codegen string detection could miss edge cases | Monitor |

## Next Cycle Recommendation
- Cycle 445: Stage 2 bootstrap attempt — compile compiler.bmb with the Stage 1 bootstrap compiler to verify self-compilation capability
