# Cycle 264: WASM Allocation Consistency

## Date
2026-02-12

## Scope
Fix remaining WASM allocation sites that didn't use the $bump_alloc function from Cycle 260: TupleInit (manual heap bump) and ArrayAlloc (nonexistent $__stack_pointer global).

## Philosophy Alignment
| Dimension | Score |
|-----------|-------|
| Core Mission Fit | 5/5 |
| Scope Boundaries | 5/5 |
| Architecture Patterns | 5/5 |
| Dependency Direction | 5/5 |

## Research Summary
- TupleInit: Used direct `global.get/set $heap_ptr` without 8-byte alignment
- ArrayAlloc: Referenced `$__stack_pointer` global which doesn't exist in generated WAT
- Both should use `$bump_alloc` for consistency and correctness

## Implementation

### WASM Codegen (`bmb/src/codegen/wasm_text.rs`)
- TupleInit: Replaced manual heap bump (5 lines) with `call $bump_alloc` (2 lines)
- ArrayAlloc: Replaced `$__stack_pointer` reference (5 lines) with `call $bump_alloc` (2 lines)
- Updated unit test `test_tuple_init` to check for `call $bump_alloc`

### Integration Tests
Added 3 new tests:
- `test_wasm_tuple_uses_bump_alloc`: Tuple init uses allocator
- `test_wasm_no_stack_pointer_global`: No __stack_pointer references
- `test_wasm_all_allocs_use_bump`: Struct + enum + array + tuple all use bump_alloc

## Test Results
- Standard tests: 3336 / 3336 passed (+3 from 3333)
- Clippy: CLEAN (0 errors)
- Build: SUCCESS

## Evaluation
| Criterion | Score | Notes |
|-----------|-------|-------|
| Correctness | 10/10 | Fixes broken ArrayAlloc ($__stack_pointer doesn't exist) |
| Architecture | 10/10 | All allocations now go through single allocator |
| Philosophy Alignment | 10/10 | Eliminates workarounds |
| Test Quality | 10/10 | Comprehensive allocation path coverage |
| Code Quality | 10/10 | Simpler code after refactor |
| **Average** | **10.0/10** | |

## Issues & Improvements
| # | Severity | Description | Action |
|---|----------|-------------|--------|
| I-01 | L | Bool/Char in ArrayAlloc still use 1-byte elem_size but bump_alloc aligns to 8 | Waste acceptable for correctness |

## Next Cycle Recommendation
- Continue compiler quality improvements
- Interpreter completeness
- Type checker edge cases
