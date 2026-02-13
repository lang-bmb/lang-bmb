# Cycle 479: Integer Methods — clamp, pow + Array Method Investigation

## Date
2025-02-12

## Scope
Add integer clamp/pow methods to bootstrap. Investigate array method support.
Array functional methods (push/pop/concat/slice) discovered to be blocked by
array representation mismatch.

## Philosophy Alignment
| Dimension | Score |
|-----------|-------|
| Core Mission Fit | 5/5 |
| Scope Boundaries | 5/5 |
| Architecture Patterns | 5/5 |
| Dependency Direction | 5/5 |

## Research Summary
- **Array representation mismatch discovered**: Bootstrap creates arrays as flat
  `[elem0, elem1, elem2]` via `calloc(N, 8)` + direct GEP stores. NO capacity/length
  header. But C runtime vec functions expect `[capacity, length, data...]` layout.
- Array functional methods (push/pop returning new arrays) require knowing the array
  length, which is NOT stored in the flat representation.
- **Resolution**: Array methods deferred until array representation includes length.
  This is an architecture-level issue (CLAUDE.md Decision Framework level 2).
- Integer clamp/pow implemented successfully.

## Implementation

### Files Modified
1. **`bmb/runtime/bmb_runtime.c`**:
   - `bmb_clamp(i64, i64, i64)`: Clamp value to range [lo, hi]
   - `bmb_pow(i64, i64)`: Fast exponentiation by squaring
   - `bmb_array_push/pop/concat/slice/len`: Added but NOT usable yet (flat array mismatch)

2. **`bootstrap/compiler.bmb`**:
   - `method_to_runtime_fn`: Added clamp→bmb_clamp, pow→bmb_pow, push→bmb_array_push,
     pop→bmb_array_pop mappings
   - Added extern declarations for clamp, pow, and array functions
   - `map_runtime_fn_full`: Added @clamp→@bmb_clamp, @pow→@bmb_pow
   - `gen_runtime_decls_io`: Added clamp/pow/array declarations

3. **`tests/bootstrap/test_golden_int_clamp_pow.bmb`** (NEW):
   - Tests: clamp with positive/negative/boundary values, pow with powers of 2/3,
     pow(x,0)=1, combined clamp+pow
   - Expected output: 161

4. **`tests/bootstrap/golden_tests.txt`**: Added int_clamp_pow test

## Test Results
| Item | Result |
|------|--------|
| Rust tests | 5,229 passed |
| Golden tests (Stage 1) | 20/20 PASS |
| Golden tests (Stage 2) | 20/20 PASS |
| Fixed point (S2==S3) | VERIFIED (71,064 lines, zero diff) |

## Evaluation
| Criterion | Score | Notes |
|-----------|-------|-------|
| Correctness | 10/10 | All tests pass, fixed point verified |
| Architecture | 8/10 | Array representation issue correctly identified as architecture-level |
| Philosophy Alignment | 9/10 | Honest research, no workarounds |
| Test Quality | 9/10 | Comprehensive clamp/pow edge case testing |
| Documentation | 9/10 | Array mismatch documented |
| Code Quality | 9/10 | Clean clamp/pow, pow uses bit-squaring |
| **Average** | **9.0/10** | |

## Issues & Improvements
| # | Severity | Description | Action |
|---|----------|-------------|--------|
| I-01 | H | Array flat representation lacks length header | Architecture change needed |
| I-02 | H | Array methods (push/pop/concat/slice) blocked by I-01 | Future cycle |
| I-03 | M | method_to_runtime_fn doesn't know receiver type | Limits shared method support |
| I-04 | L | Array C functions added but not usable | Will be used after representation fix |

## Next Cycle Recommendation
- Cycle 480: Fix array representation to include length (enables array methods)
  OR: Continue with non-array features (enum support, closures)
- The array representation change is a Decision Framework level 2 issue
  (compiler structure: MIR/AST must express array length)
