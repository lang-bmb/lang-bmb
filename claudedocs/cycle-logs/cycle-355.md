# Cycle 355: Chained method error context — show receiver type

## Date
2026-02-13

## Scope
Restore and enhance type parameter display in method-not-found errors for generic types. Show concrete element/inner types in error messages so chained method errors are more informative.

## Philosophy Alignment
| Dimension | Score |
|-----------|-------|
| Core Mission Fit | 5/5 |
| Scope Boundaries | 5/5 |
| Architecture Patterns | 5/5 |
| Dependency Direction | 5/5 |

## Implementation

### Type Checker (types/mod.rs)
Restored and enhanced type parameter display in 17 catchall error messages:
- **Array**: `"for Array"` → `"for [i64]"` (shows element type)
- **Option**: `"for Option"` → `"for Option<i64>"` (shows inner type)
- **Result**: `"for Result"` → `"for Result<i64, String>"` (shows both type params)
- **Thread<T>**: Restored `<T>` display
- **Mutex<T>**: Restored `<T>` display (both direct and Generic variants)
- **Arc<T>**: Restored `<T>` display (both variants)
- **Atomic<T>**: Restored `<T>` display (both variants)
- **Sender<T>**: Restored `<T>` display (both variants)
- **Receiver<T>**: Restored `<T>` display (both variants)
- **RwLock<T>**: Restored `<T>` display (both variants)

Example chain: `"hello".chars().foobar()` → `unknown method 'foobar' for [String]`

### Integration Tests
Added 4 tests:
- `test_chain_error_shows_array_elem_type`: `[1,2,3].foobar()` → "for [i64]"
- `test_chain_error_shows_option_inner_type`: `x.foobar()` on `i64?` → "Option<i64>"
- `test_chain_method_result_type_context`: `"hello".len().abz()` → "for i64"
- `test_chain_method_string_to_array`: `"hello".chars().foobar()` → "for [String]"

## Test Results
- Standard tests: 4089 / 4089 passed (+4)
- Clippy: PASS (0 warnings)
- Build: SUCCESS

## Evaluation
| Criterion | Score | Notes |
|-----------|-------|-------|
| Correctness | 10/10 | All type params correctly displayed |
| Architecture | 10/10 | Fixed Cycle 353 regression, enhanced display |
| Philosophy Alignment | 10/10 | Concrete types help AI understand errors |
| Test Quality | 9/10 | Tests cover Array, Option, chained contexts |
| Code Quality | 10/10 | Clean format strings |
| **Average** | **9.8/10** | |

## Issues & Improvements
| # | Severity | Description | Action |
|---|----------|-------------|--------|
| None | - | Clean implementation | - |

## Next Cycle Recommendation
- Cycle 356: Integration tests for error message quality
