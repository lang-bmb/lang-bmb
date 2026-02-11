# Cycle 258: WASM String Constant Data Section

## Date
2026-02-12

## Scope
Replace TODO placeholder for string constants in WASM text codegen with proper data section emission. Strings are interned (deduplicated) and stored in linear memory via WAT `(data ...)` segments.

## Philosophy Alignment
| Dimension | Score |
|-----------|-------|
| Core Mission Fit | 5/5 |
| Scope Boundaries | 5/5 |
| Architecture Patterns | 5/5 |
| Dependency Direction | 5/5 |

## Research Summary
- WASM linear memory data segments: `(data (i32.const <offset>) "<escaped bytes>")`
- String interning: deduplicate identical strings to same offset
- Data area starts at offset 2048 (after globals/IO buffer reserved area at 0-1024)
- Bytes escaped as `\xx` hex format per WAT spec

## Implementation

### WASM Codegen (`bmb/src/codegen/wasm_text.rs`)
- Added `string_data: RefCell<Vec<(String, u32)>>` — interned string table
- Added `next_data_offset: RefCell<u32>` — next allocation offset (starts at 2048)
- `intern_string(&self, s: &str) -> (u32, u32)` — deduplicates and returns (offset, len)
- `emit_data_section(&self, out: &mut String)` — emits `(data ...)` segments
- Updated `emit_constant` for `Constant::String` — uses `intern_string` instead of `i32.const 0`
- Added `emit_data_section()` call in `generate()` before closing `)`
- Updated `new()` and `with_target()` constructors

### Integration Tests (`bmb/tests/integration.rs`)
- Added `compile_to_wat()` and `compile_to_wat_with_target()` helpers
- Added 10 new tests:
  - `test_wasm_string_constant_data_section`: Data section emitted for strings
  - `test_wasm_string_constant_offset`: String at offset 2048
  - `test_wasm_multiple_string_constants`: Multiple strings get separate segments
  - `test_wasm_string_deduplication`: Same string deduplicated to 1 segment
  - `test_wasm_no_string_no_data_section`: No data section without strings
  - `test_wasm_string_constant_no_todo`: TODO comment removed
  - `test_wasm_browser_target_string_constant`: Works on Browser target
  - `test_wasm_standalone_target_string_constant`: Works on Standalone target
  - `test_wasm_string_data_section_comment`: Descriptive comment present
  - `test_wasm_module_structure_valid`: Data section inside module structure

## Test Results
- Standard tests: 3303 / 3303 passed (+10 from 3293)
- Clippy: CLEAN (0 errors)
- Build: SUCCESS

## Evaluation
| Criterion | Score | Notes |
|-----------|-------|-------|
| Correctness | 10/10 | String interning, dedup, all targets work |
| Architecture | 10/10 | RefCell for interior mutability during codegen traversal |
| Philosophy Alignment | 10/10 | Replaces workaround (i32.const 0) with proper impl |
| Test Quality | 10/10 | 10 tests cover dedup, targets, structure, edge cases |
| Code Quality | 10/10 | Clean, follows existing wasm_text.rs patterns |
| **Average** | **10.0/10** | |

## Issues & Improvements
| # | Severity | Description | Action |
|---|----------|-------------|--------|
| I-01 | M | String representation is (offset) only, no length — callee needs to know length | Future: pair (ptr, len) representation |
| I-02 | L | No string runtime functions (concat, compare, etc.) in WASM | Future WASM stdlib |

## Next Cycle Recommendation
- WASM char constant handling
- WASM enum/struct codegen improvements
- Or: Continue compiler quality improvements
