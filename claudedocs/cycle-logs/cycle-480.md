# Cycle 480: Additional String Methods — to_upper, to_lower, repeat, is_empty

## Date
2025-02-12

## Scope
Add 4 additional string methods to bootstrap: to_upper, to_lower, repeat, is_empty.
Expands the string method library for self-hosted compilation support.

## Philosophy Alignment
| Dimension | Score |
|-----------|-------|
| Core Mission Fit | 5/5 |
| Scope Boundaries | 5/5 |
| Architecture Patterns | 5/5 |
| Dependency Direction | 5/5 |

## Research Summary
- Followed established pattern from Cycle 478 (string methods)
- 4 methods selected: to_upper, to_lower (case conversion), repeat (string repetition), is_empty (emptiness check)
- All follow the existing method dispatch pipeline: method_to_runtime_fn → extern decl → get_call_arg_types → get_call_return_type
- Type signatures: to_upper/to_lower/is_empty take ptr (string), repeat takes ptr+i64
- Return types: to_upper/to_lower/repeat return ptr (new string), is_empty returns i64 (0/1)

## Implementation

### Files Modified
1. **`bmb/runtime/bmb_runtime.c`**:
   - `bmb_string_to_upper(BmbString*)`: ASCII uppercase conversion
   - `bmb_string_to_lower(BmbString*)`: ASCII lowercase conversion
   - `bmb_string_repeat(BmbString*, i64)`: String repetition via memcpy loop
   - `bmb_string_is_empty(BmbString*)`: Null/empty check returning 0/1

2. **`bootstrap/compiler.bmb`**:
   - `method_to_runtime_fn`: Added to_upper→bmb_string_to_upper, to_lower→bmb_string_to_lower, repeat→bmb_string_repeat, is_empty→bmb_string_is_empty
   - Added 4 extern declarations with optimization attributes
   - `gen_runtime_decls_string`: Added 4 new declarations
   - `get_call_arg_types`: Added to_upper→"p", to_lower→"p", repeat→"pi", is_empty→"p"
   - `get_call_return_type`: Added to_upper/to_lower/repeat→"ptr" (is_empty returns i64, default)

3. **`tests/bootstrap/test_golden_string_ops.bmb`** (NEW):
   - Tests: to_upper, to_lower, round-trip, repeat (3/1/0 times), is_empty, combined with starts_with/ends_with/contains
   - Expected output: 166

4. **`tests/bootstrap/golden_tests.txt`**: Added string_ops test

## Test Results
| Item | Result |
|------|--------|
| Rust tests | 5,229 passed |
| Golden tests (Stage 1) | 21/21 PASS |
| Golden tests (Stage 2) | 21/21 PASS |
| Fixed point (S2==S3) | VERIFIED (71,364 lines, zero diff) |

## Evaluation
| Criterion | Score | Notes |
|-----------|-------|-------|
| Correctness | 10/10 | All tests pass, fixed point verified |
| Architecture | 9/10 | Follows established method dispatch pattern |
| Philosophy Alignment | 9/10 | Feature expansion aligned with self-hosting goals |
| Test Quality | 9/10 | Tests cover each method + combinations + edge cases (empty, repeat 0) |
| Documentation | 9/10 | Clear version comments in compiler code |
| Code Quality | 9/10 | ASCII-only case conversion is appropriate for bootstrap |
| **Average** | **9.2/10** | |

## Issues & Improvements
| # | Severity | Description | Action |
|---|----------|-------------|--------|
| I-01 | H | Array flat representation lacks length header | Architecture change needed (from Cycle 479) |
| I-02 | H | Array methods (push/pop/concat/slice) blocked by I-01 | Future cycle |
| I-03 | M | method_to_runtime_fn doesn't know receiver type | Limits shared method support |
| I-04 | L | Case conversion is ASCII-only (no Unicode) | Sufficient for bootstrap |

## Next Cycle Recommendation
- Cycle 481: Enum/variant basic support OR array representation fix
- The array representation change is still the biggest blocker for feature completeness
