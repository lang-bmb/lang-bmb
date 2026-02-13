# Cycle 477: Float Method Support in Bootstrap Compiler

## Date
2025-02-12

## Scope
Add float-only method support (floor, ceil, round, sqrt, is_nan, to_int, to_float)
to the bootstrap compiler. These methods are unambiguous (type-unique) and don't
require type-aware dispatch in the method lowering phase.

## Philosophy Alignment
| Dimension | Score |
|-----------|-------|
| Core Mission Fit | 5/5 |
| Scope Boundaries | 5/5 |
| Architecture Patterns | 5/5 |
| Dependency Direction | 5/5 |

## Research Summary
- Rust type checker already supports all float methods (Cycle 268, v0.90.34)
- Bootstrap had no float method support — `method_to_runtime_fn` fell through to `bmb_` prefix
- `to_int`/`to_float` can reuse existing intrinsic intercepts (`f64_to_i64`/`i64_to_f64`)
- Float-only methods (floor, ceil, round, sqrt, is_nan) are unambiguous without type info
- Shared methods (abs, min, max) need type-aware dispatch — deferred

## Implementation

### Files Modified
1. **`bmb/runtime/bmb_runtime.c`**:
   - Added `#include <math.h>`
   - Added 9 float math wrapper functions:
     `bmb_f64_floor`, `bmb_f64_ceil`, `bmb_f64_round`, `bmb_f64_sqrt`,
     `bmb_f64_abs`, `bmb_f64_is_nan`, `bmb_f64_min`, `bmb_f64_max`, `bmb_f64_to_int`

2. **`bootstrap/compiler.bmb`**:
   - `method_to_runtime_fn`: Added 7 float method mappings:
     floor→bmb_f64_floor, ceil→bmb_f64_ceil, round→bmb_f64_round,
     sqrt→bmb_f64_sqrt, is_nan→bmb_f64_is_nan, to_int→f64_to_i64, to_float→i64_to_f64
   - Added 9 extern declarations (`gen_extern_f64_*`) with optimization attributes
   - `gen_runtime_decls_ext`: Added float method declarations
   - `get_call_arg_types`: Added 9 entries (d/dd type codes)
   - `get_call_return_type`: Added double return types for floor/ceil/round/sqrt/abs/min/max

3. **`tests/bootstrap/test_golden_float_methods.bmb`** (NEW):
   - Tests: floor, ceil, round, sqrt, to_int, to_float, is_nan
   - Expected output: 176

4. **`tests/bootstrap/golden_tests.txt`**: Added float_methods test entry

### Design Decisions
- Used C runtime wrappers (math.h) instead of LLVM intrinsics — simpler, LLVM -O2
  will optimize to intrinsics anyway
- `to_int`/`to_float` reuse existing intrinsic intercepts (sitofp/fptosi) — zero overhead
- Added `nounwind willreturn memory(none) speculatable` attributes to all float math
  declarations — enables LICM, CSE, and inlining by LLVM optimizer
- Float abs/min/max deferred — needs type-aware dispatch (method_to_runtime_fn
  doesn't know receiver type)

## Test Results
| Item | Result |
|------|--------|
| Rust tests | 5,229 passed |
| Golden tests (Stage 1) | 18/18 PASS |
| Golden tests (Stage 2) | 18/18 PASS |
| Fixed point (S2==S3) | VERIFIED (70,358 lines, zero diff) |

## Evaluation
| Criterion | Score | Notes |
|-----------|-------|-------|
| Correctness | 10/10 | All tests pass, fixed point verified, new golden test passes |
| Architecture | 9/10 | Clean method→runtime function mapping pattern |
| Philosophy Alignment | 9/10 | Feature expansion for bootstrap self-hosting |
| Test Quality | 9/10 | Comprehensive golden test covering all new methods |
| Documentation | 9/10 | Clear version comments, cycle log |
| Code Quality | 9/10 | Consistent with existing int method pattern |
| **Average** | **9.2/10** | |

## Issues & Improvements
| # | Severity | Description | Action |
|---|----------|-------------|--------|
| I-01 | H | Float abs/min/max need type-aware dispatch | Future cycle: type annotations in method lowering |
| I-02 | M | to_string on float not supported (ambiguous with int.to_string) | Needs type-aware dispatch |
| I-03 | L | C runtime wrappers add indirection vs LLVM intrinsics | LLVM -O2 optimizes away |
| I-04 | L | Fixed point lines grew 70,358 (from 69,718 = +640) | New function declarations |

## Next Cycle Recommendation
- Cycle 478: String parsing + type conversion methods (parse_int, starts_with, ends_with, etc.)
- Consider type-aware method dispatch for shared methods (abs/min/max/to_string on float)
- Continue Phase B: Bootstrap Feature Expansion
