# Cycle 482: Array Methods — push, pop, arr_len, arr_concat, arr_slice

## Date
2025-02-12

## Scope
Enable array methods (push, pop, arr_len, arr_concat, arr_slice) in bootstrap compiler,
now that array representation uses [capacity, length, data...] header (Cycle 481).

## Philosophy Alignment
| Dimension | Score |
|-----------|-------|
| Core Mission Fit | 5/5 |
| Scope Boundaries | 5/5 |
| Architecture Patterns | 5/5 |
| Dependency Direction | 5/5 |

## Research Summary
- Array methods were added to C runtime in Cycle 479 but used `int64_t` for pointer params
- Cycle 481 fixed array representation to [capacity, length, data...], enabling method compatibility
- Bootstrap method dispatch pipeline: `method_to_runtime_fn` → extern decl → `get_call_arg_types` → `get_call_return_type`
- Type-aware dispatch NOT needed for array methods since they use unique names (`arr_len`, `arr_concat`, `arr_slice`)
- `push`/`pop` names don't conflict with other types currently

## Implementation

### Files Modified
1. **`bootstrap/compiler.bmb`** (from previous session):
   - `method_to_runtime_fn`: Added push→bmb_array_push, pop→bmb_array_pop, arr_len→bmb_array_len, arr_concat→bmb_array_concat, arr_slice→bmb_array_slice
   - Extern declarations: Changed from i64 to ptr for array parameters
   - `get_call_arg_types`: Added push→"pi", pop→"p", concat→"pp", slice→"pii", len→"p"
   - `get_call_return_type`: Added push/pop/concat/slice→"ptr" (arr_len returns i64, default)

2. **`bmb/runtime/bmb_runtime.c`**:
   - Updated all 5 array functions from `int64_t` params/returns to `int64_t*` pointer types
   - Removed unnecessary int64_t↔pointer casts
   - Version bump to v0.90.98

3. **`tests/bootstrap/test_golden_array_methods.bmb`** (NEW):
   - Tests: arr_len, push, pop, arr_concat, arr_slice, chained push operations
   - Expected output: 175

4. **`tests/bootstrap/golden_tests.txt`**: Added array_methods test

### Key Design Decisions
- **Unique method names**: `arr_len`, `arr_concat`, `arr_slice` avoid conflicts with string methods (`len`, `concat`, `slice`) since bootstrap has no type-aware dispatch
- **`push`/`pop` use short names**: No conflict since no other type currently has push/pop methods
- **Pointer types in C runtime**: Changed from int64_t casts to proper int64_t* types for type safety and ABI correctness with LLVM ptr declarations
- **Functional style**: All array methods return NEW arrays (immutable semantics), matching the existing C runtime implementations

## Test Results
| Item | Result |
|------|--------|
| Rust tests | 5,229 passed |
| Golden tests (Stage 1) | 23/23 PASS |
| Golden tests (Stage 2) | 23/23 PASS |
| Fixed point (S2==S3) | VERIFIED (72,676 lines, zero diff) |

## Evaluation
| Criterion | Score | Notes |
|-----------|-------|-------|
| Correctness | 10/10 | All tests pass, fixed point verified, array methods work correctly |
| Architecture | 9/10 | Clean integration into existing method dispatch pipeline |
| Philosophy Alignment | 10/10 | Resolves I-02 from Cycle 480, builds on Cycle 481 array fix |
| Test Quality | 9/10 | Tests cover all 5 methods + chaining + edge cases |
| Documentation | 9/10 | Version comments, design decisions documented |
| Code Quality | 9/10 | Proper pointer types, no unnecessary casts |
| **Average** | **9.3/10** | |

## Issues & Improvements
| # | Severity | Description | Action |
|---|----------|-------------|--------|
| I-01 | H | Type-aware method dispatch needed for shared names (len, abs, min, max) | Future cycle - requires major architecture change |
| I-02 | M | `arr_len`/`arr_concat`/`arr_slice` naming is workaround for lack of type dispatch | Accept until type dispatch is implemented |
| I-03 | L | Array join method not yet implemented in bootstrap | Low priority |
| I-04 | L | push/pop could conflict if future types add same methods | Monitor |

## Next Cycle Recommendation
- Cycle 483: Array `for-in` iteration in bootstrap OR additional method coverage
  - For-in iteration: `for x in arr { ... }` desugars to index loop
  - Alternative: Continue expanding bootstrap feature coverage (closures, nullable types)
- Longer term: Type-aware method dispatch to unify `len`/`arr_len`, `concat`/`arr_concat` etc.
