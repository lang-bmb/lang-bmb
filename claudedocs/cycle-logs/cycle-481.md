# Cycle 481: Array Representation Fix — [capacity, length, data...] Header

## Date
2025-02-12

## Scope
Fix array representation in bootstrap compiler to include capacity/length header.
Previously arrays were flat `[elem0, elem1, ...]`. Now they use vec-compatible
`[capacity, length, data0, data1, ...]` layout, enabling future array method support.

## Philosophy Alignment
| Dimension | Score |
|-----------|-------|
| Core Mission Fit | 5/5 |
| Scope Boundaries | 5/5 |
| Architecture Patterns | 5/5 |
| Dependency Direction | 5/5 |

## Research Summary
- **Root cause identified in Cycle 479**: Bootstrap creates flat arrays via `calloc(N, 8)` + GEP stores at indices 0,1,2... C runtime vec functions expect `[capacity, length, data...]` layout.
- **Array indexing uses `gep`**, struct field access uses `field-access` — different MIR instructions, allowing targeted offset change.
- **Tuple creation** shares `lower_array_elements_sb` but must NOT get the +2 offset — resolved by adding `base_offset` parameter.
- **Key insight**: `step_array_repeat` needed a `copy` of the array base pointer at the end, since the last temp was a GEP pointer (caused type mismatch: ptr vs i64 in LLVM).

## Implementation

### Files Modified
1. **`bootstrap/compiler.bmb`**:
   - `lower_array_literal_sb`: Allocate `count+2` slots, store capacity at [0] and length at [1]
   - `lower_array_elements_sb` → `lower_elements_with_offset_sb`: Added `base_offset` parameter (2 for arrays, 0 for tuples)
   - `lower_tuple_elements_sb`: Wrapper with offset=0 for tuple element storage
   - `lower_tuple_sb`: Uses `lower_tuple_elements_sb` instead of `lower_array_elements_sb`
   - `lower_array_repeat_sb`: Allocate `count+2`, store capacity/length header, copy base pointer
   - `step_array_repeat`: Same changes for iterative path, parse count with `parse_int_simple`
   - `step_array_index_final`: Add `const 2` + `+` to offset index before GEP
   - `lower_index_sb`: Same offset for recursive path
   - `step_set_index_final`: Add offset before GEP for writes
   - `lower_set_index_sb`: Same offset for recursive path

2. **`tests/bootstrap/test_golden_array_header.bmb`** (NEW):
   - Tests: array literal + sum, array repeat + set/get, single-element array, nested set/get
   - Expected output: 188

3. **`tests/bootstrap/golden_tests.txt`**: Added array_header test

### Key Design Decisions
- **Offset at lowering level, not codegen**: The +2 offset is added in MIR generation, using `const 2` + `+` instructions. This keeps the codegen GEP handler unchanged.
- **Tuple exemption**: Tuples share element storage logic but use offset=0. Solved via `base_offset` parameter.
- **Result temp copy**: After storing headers, a `copy` of the array base is emitted so the step machine finds a pointer (not a GEP result) as the expression result.

## Test Results
| Item | Result |
|------|--------|
| Rust tests | 5,229 passed |
| Golden tests (Stage 1) | 22/22 PASS |
| Golden tests (Stage 2) | 22/22 PASS |
| Fixed point (S2==S3) | VERIFIED (72,445 lines, zero diff) |

## Evaluation
| Criterion | Score | Notes |
|-----------|-------|-------|
| Correctness | 10/10 | All tests pass including sieve (heavy array usage), fixed point verified |
| Architecture | 10/10 | Proper fix at the right abstraction level (lowering), not a workaround |
| Philosophy Alignment | 10/10 | Resolves architecture-level issue I-01 from Cycle 479 |
| Test Quality | 9/10 | Dedicated array header test + all existing array tests pass |
| Documentation | 9/10 | Clear version comments, design decisions documented |
| Code Quality | 9/10 | Clean separation of array vs tuple element storage |
| **Average** | **9.5/10** | |

## Issues & Improvements
| # | Severity | Description | Action |
|---|----------|-------------|--------|
| I-01 | H | Array methods still need type-aware dispatch for `len` | Future cycle |
| I-02 | M | method_to_runtime_fn doesn't know receiver type | Limits shared method support |
| I-03 | L | Array header adds 16 bytes overhead per array | Acceptable for vec compatibility |
| I-04 | L | parse_int_simple limits array_repeat count to literal integers | Sufficient for current usage |

## Next Cycle Recommendation
- Cycle 482: Test array methods (push, pop, len) now that representation matches C runtime
  - Need to solve type-aware dispatch for `len` (currently routes to bmb_string_len)
  - OR: Add `arr_len` as a separate method name to avoid conflict
- Alternative: Continue with non-array features (closures, nullable types)
