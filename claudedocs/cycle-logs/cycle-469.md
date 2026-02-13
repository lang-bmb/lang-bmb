# Cycle 469: Bootstrap Integer Method Support (abs, min, max)

## Date
2025-02-12

## Scope
Add integer method support (.abs(), .min(), .max()) to the bootstrap compiler,
enabling the bootstrap to compile programs using these methods. Fix existing
extern declarations that had wrong function names.

## Philosophy Alignment
| Dimension | Score |
|-----------|-------|
| Core Mission Fit | 5/5 |
| Scope Boundaries | 5/5 |
| Architecture Patterns | 5/5 |
| Dependency Direction | 5/5 |

## Research Summary
- Bootstrap's `method_to_runtime_fn` fallback already maps `.abs()` → `bmb_abs` correctly
- Existing `gen_extern_min/max` used wrong names: `@min`/`@max` instead of `@bmb_min`/`@bmb_max`
- Duplicate function definitions caused LLVM "invalid redefinition" error
- Free function calls `min(x, y)` need `map_runtime_fn` mapping since they generate `@min`
- Method calls `.min(y)` use the `"bmb_" + method` fallback correctly

## Implementation

### Files Modified
1. **`bootstrap/compiler.bmb`**:
   - Fixed `gen_extern_min`: `@min` → `@bmb_min` (line 5582)
   - Fixed `gen_extern_max`: `@max` → `@bmb_max` (line 5583)
   - Added `map_runtime_fn` entries: `@abs` → `@bmb_abs`, `@min` → `@bmb_min`, `@max` → `@bmb_max`
   - Removed duplicate `gen_extern_abs/min/max` that I initially added
   - Removed duplicate inclusion in `gen_runtime_decls_rest`

2. **`tests/bootstrap/test_golden_int_methods.bmb`** — NEW
   - Tests: `.abs()`, `.min()`, `.max()` on integers
   - 5 test functions: abs, min, max, chained calls, expressions

3. **`tests/bootstrap/golden_tests.txt`** — Added new test entry

### Key Fix: Function Name Resolution
The bootstrap compiler has two paths for integer methods:
- **Method calls** (`.abs()`, `.min()`): `method_to_runtime_fn` fallback → `bmb_abs`/`bmb_min`
- **Free function calls** (`abs(x)`, `min(x, y)`): `map_runtime_fn` needed explicit mapping

Both paths now correctly resolve to the runtime functions `bmb_abs`/`bmb_min`/`bmb_max`.

## Test Results
| Item | Result |
|------|--------|
| Rust tests | 5,229 passed |
| Golden tests | 17/17 PASS (was 16/16) |
| Stage 1 build | SUCCESS |
| Fixed point | VERIFIED (S2==S3, 68,993 lines) |

## Evaluation
| Criterion | Score | Notes |
|-----------|-------|-------|
| Correctness | 10/10 | All tests pass, fixed point verified, bug fix correct |
| Architecture | 9/10 | Clean fix leveraging existing fallback mechanism |
| Philosophy Alignment | 9/10 | Advances self-hosting capability |
| Test Quality | 9/10 | Golden test covers abs/min/max with various patterns |
| Documentation | 8/10 | Code comments + cycle log |
| Code Quality | 9/10 | Proper fix, removed duplicates |
| **Average** | **9.0/10** | |

## Issues & Improvements
| # | Severity | Description | Action |
|---|----------|-------------|--------|
| I-01 | M | clamp, pow not in C runtime — would need runtime additions | Future: add to bmb_runtime.c |
| I-02 | M | Float methods (.floor, .ceil, .sqrt, .round) still missing | Need runtime functions + extern declarations |
| I-03 | L | to_float, to_int conversions may need special MIR handling | Investigate MIR lowering for type conversions |

## Next Cycle Recommendation
- Cycle 470: Full regression check and golden binary update with new capabilities
- Consider adding float math methods to runtime + bootstrap
- Update golden binary with latest integer method support
