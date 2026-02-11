# Cycle 263: Interpreter Enum Method Dispatch

## Date
2026-02-12

## Scope
Add trait method dispatch for enum values in the tree-walking interpreter. Previously only struct instances could dispatch trait methods; enum instances fell through to the "object with methods" type error.

## Philosophy Alignment
| Dimension | Score |
|-----------|-------|
| Core Mission Fit | 5/5 |
| Scope Boundaries | 5/5 |
| Architecture Patterns | 5/5 |
| Dependency Direction | 5/5 |

## Research Summary
- Cycle 255 added struct trait dispatch via `impl_methods: HashMap<(String, String), FnDef>`
- Enum values are `Value::Enum(enum_name, variant, args)` — `enum_name` is the dispatch key
- Same pattern as struct dispatch: look up `(enum_name, method)` in `impl_methods`
- Enum impl methods already collected during `load()` (Cycle 255 handles ImplBlock for all types)

## Implementation

### Interpreter (`bmb/src/interp/eval.rs`)
- Added `Value::Enum` match arm in `eval_method_call` before catch-all
- Same pattern as struct dispatch: look up `(enum_name, method)` key
- Prepend receiver as `self` parameter, call `call_function`

### Integration Tests (`bmb/tests/integration.rs`)
Added 6 new tests:
- `test_enum_method_dispatch_basic`: Simple trait method on enum
- `test_enum_method_dispatch_with_args`: Method with additional args
- `test_enum_method_dispatch_multiple_methods`: Multiple trait methods
- `test_enum_method_dispatch_undefined_error`: Undefined method detected
- `test_enum_and_struct_both_impl_same_trait`: Both types impl same trait
- `test_enum_method_in_function`: Enum dispatch inside function

## Test Results
- Standard tests: 3333 / 3333 passed (+6 from 3327)
- Clippy: CLEAN (0 errors)
- Build: SUCCESS

## Evaluation
| Criterion | Score | Notes |
|-----------|-------|-------|
| Correctness | 10/10 | All dispatch patterns work |
| Architecture | 10/10 | Follows Cycle 255 struct dispatch pattern exactly |
| Philosophy Alignment | 10/10 | Completes trait dispatch for all value types |
| Test Quality | 10/10 | Covers basic, args, multi-method, error, mixed types |
| Code Quality | 10/10 | 8-line addition, reuses existing infrastructure |
| **Average** | **10.0/10** | |

## Issues & Improvements
| # | Severity | Description | Action |
|---|----------|-------------|--------|
| I-01 | L | Enum dispatch doesn't use variant info (all variants share methods) | Correct by language semantics |

## Next Cycle Recommendation
- WASM codegen for function calls (call emission improvements)
- Type checker improvements
- Or: More interpreter completeness
