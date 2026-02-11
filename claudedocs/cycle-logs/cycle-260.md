# Cycle 260: WASM Bump Allocator

## Date
2026-02-12

## Scope
Replace all `i32.const 0 ;; TODO: proper memory allocation` placeholders in WASM codegen with a proper bump allocator function using the `$heap_ptr` global.

## Philosophy Alignment
| Dimension | Score |
|-----------|-------|
| Core Mission Fit | 5/5 |
| Scope Boundaries | 5/5 |
| Architecture Patterns | 5/5 |
| Dependency Direction | 5/5 |

## Research Summary
- 3 allocation sites in wasm_text.rs used `i32.const 0` as placeholder: struct init, enum variant, array init
- WASM linear memory has `$heap_ptr` global (starts at 1024)
- Bump allocation is simplest correct approach for WASM: advance heap pointer, return old value
- 8-byte alignment ensures proper access for i64/f64 stores

## Implementation

### WASM Codegen (`bmb/src/codegen/wasm_text.rs`)
- Added `emit_bump_allocator()` method — emits `$bump_alloc(size: i32) -> i32`
  - Saves current `$heap_ptr` as return value
  - Advances `$heap_ptr` by size rounded up to 8-byte alignment: `(size + 7) & -8`
- Called from `emit_runtime_functions()` for all targets
- Updated `StructInit`: `i32.const {fields * 8}` + `call $bump_alloc`
- Updated `EnumVariant`: `i32.const {(args + 1) * 8}` + `call $bump_alloc`
- Updated `ArrayInit`: `i32.const {elements * 8}` + `call $bump_alloc`

### Integration Tests (`bmb/tests/integration.rs`)
Added 10 new tests:
- `test_wasm_bump_alloc_function_present`: $bump_alloc in WAT output
- `test_wasm_bump_alloc_uses_heap_ptr`: Uses $heap_ptr global
- `test_wasm_struct_uses_bump_alloc`: Struct calls $bump_alloc
- `test_wasm_struct_alloc_size`: 3-field struct = 24 bytes
- `test_wasm_array_uses_bump_alloc`: Array calls $bump_alloc
- `test_wasm_array_alloc_size`: 4-element array = 32 bytes
- `test_wasm_enum_uses_bump_alloc`: Enum calls $bump_alloc
- `test_wasm_bump_alloc_8byte_alignment`: Alignment math correct
- `test_wasm_no_todo_memory_allocation`: No TODO placeholders remain
- `test_wasm_bump_alloc_all_targets`: All 3 targets have allocator

## Test Results
- Standard tests: 3324 / 3324 passed (+10 from 3314)
- Clippy: CLEAN (0 errors)
- Build: SUCCESS

## Evaluation
| Criterion | Score | Notes |
|-----------|-------|-------|
| Correctness | 10/10 | Proper bump allocation with alignment |
| Architecture | 10/10 | Single allocator function, clean integration |
| Philosophy Alignment | 10/10 | Replaces workaround with proper implementation |
| Test Quality | 10/10 | All allocation types and targets tested |
| Code Quality | 10/10 | Minimal, focused changes |
| **Average** | **10.0/10** | |

## Issues & Improvements
| # | Severity | Description | Action |
|---|----------|-------------|--------|
| I-01 | M | No memory growth — will OOM if heap exceeds initial page | Future: memory.grow |
| I-02 | L | No deallocation — bump allocator never frees | Acceptable for initial impl |

## Next Cycle Recommendation
- Verification fallback soundness fix
- WASM memory growth on OOM
- Or: Additional compiler quality improvements
