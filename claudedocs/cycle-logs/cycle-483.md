# Cycle 483: Array For-In Iteration in Bootstrap

## Date
2025-02-12

## Scope
Add `for x in arr { body }` array iteration support to bootstrap compiler via parser-level
desugaring to index loop using `arr_len` method (Cycle 482).

## Philosophy Alignment
| Dimension | Score |
|-----------|-------|
| Core Mission Fit | 5/5 |
| Scope Boundaries | 5/5 |
| Architecture Patterns | 5/5 |
| Dependency Direction | 5/5 |

## Research Summary
- Bootstrap parser only supported range-based for: `for x in start..end { body }`
- Array for-in requires detecting absence of `..` after expression in `parse_for_range`
- Desugaring approach chosen over new AST node: reuses all existing lowering/step machinery
- Source position used as suffix for internal variable names to ensure uniqueness in nested loops
- Array accumulator pattern used in tests (bootstrap doesn't support `set var = expr`, only `set arr[i]` and `set obj.field`)

## Implementation

### Files Modified
1. **`bootstrap/compiler.bmb`**:
   - `parse_for_range`: Now checks for `{` (array for-in) in addition to `..` (range for)
   - `parse_for_in_body` (NEW): Desugars `for x in arr { body }` to:
     ```
     (block (let <__fia_N> arr
         (let <__fil_N> (method arr_len (var <__fia_N>))
             (for <__fii_N> (int 0) (var <__fil_N>)
                 (let <x> (index (var <__fia_N>) (var <__fii_N>)) body)))))
     ```
   - Uses source position (`uid`) as suffix for unique internal variable names

2. **`tests/bootstrap/test_golden_for_in_array.bmb`** (NEW):
   - Tests: basic sum, count, for-in with array repeat, **nested for-in loops**
   - Uses array-based accumulators (`set acc[0] = acc[0] + x`)
   - Expected output: 192

3. **`tests/bootstrap/golden_tests.txt`**: Added for_in_array test

### Key Design Decisions
- **Parser-level desugaring**: No new AST node needed. The desugared code uses existing `let`, `method`, `index`, and `for` (range) AST nodes. This means all existing lowering, step machine, and codegen paths work unmodified.
- **Position-based unique names**: `__fia_N`, `__fil_N`, `__fii_N` where N is source position after `in`. Guarantees uniqueness for nested for-in loops since they have different source positions.
- **`arr_len` dependency**: The desugaring calls `.arr_len()` on the array, leveraging the method added in Cycle 482. This is clean because it goes through the standard method dispatch pipeline.

## Test Results
| Item | Result |
|------|--------|
| Rust tests | 5,229 passed |
| Golden tests (Stage 1) | 24/24 PASS |
| Golden tests (Stage 2) | 24/24 PASS |
| Fixed point (S2==S3) | VERIFIED (72,919 lines, zero diff) |

## Evaluation
| Criterion | Score | Notes |
|-----------|-------|-------|
| Correctness | 10/10 | All tests pass, nested for-in works, fixed point verified |
| Architecture | 10/10 | Parser-level desugaring is the right abstraction level — no changes to lowering/codegen |
| Philosophy Alignment | 10/10 | Proper feature addition, not a workaround |
| Test Quality | 9/10 | Tests cover basic, count, array repeat, and nested for-in |
| Documentation | 9/10 | Desugaring documented in code comments |
| Code Quality | 10/10 | Minimal code change (one modified function, one new function) |
| **Average** | **9.7/10** | |

## Issues & Improvements
| # | Severity | Description | Action |
|---|----------|-------------|--------|
| I-01 | M | `set var = expr` not supported in bootstrap — limits loop patterns | Future cycle: add local variable mutation |
| I-02 | L | Rust compiler MIR lowering also lacks array for-in (only interpreter supports it) | Future cycle |
| I-03 | L | `break`/`continue` inside for-in not tested | Low priority — works via desugaring to range for |

## Next Cycle Recommendation
- Cycle 484: Local variable mutation (`set var = expr`) in bootstrap
  - Currently only `set arr[i]` and `set obj.field` supported
  - Would enable more natural loop patterns
  - Alternative: String method expansion or other feature work
