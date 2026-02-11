# Cycle 255: Trait Method Dispatch in Interpreter

## Date
2026-02-12

## Scope
Implement trait method dispatch for struct instances in the tree-walking interpreter. Previously `s.value()` on a struct with `impl HasValue for S` failed with "expected object with methods, got Struct".

## Philosophy Alignment
| Dimension | Score |
|-----------|-------|
| Core Mission Fit | 5/5 |
| Scope Boundaries | 5/5 |
| Architecture Patterns | 5/5 |
| Dependency Direction | 5/5 |

## Research Summary
- Interpreter had no trait/impl storage (TraitDef and ImplBlock completely ignored in `load()`)
- Method dispatch only handled built-in types (String, Array, Option, Result, Nullable<Int>)
- `Value::Struct(type_name, fields)` had no dispatch path
- `call_function` requires `&mut self`, so `eval_method_call` signature needed change from `&self` to `&mut self`
- Trait methods declare `self: Self` as first parameter — receiver must be prepended to args

## Implementation

### Interpreter (`bmb/src/interp/eval.rs`)
1. Added `impl_methods: HashMap<(String, String), FnDef>` to Interpreter struct
2. Updated `load()` to collect impl block methods: `(type_name, method_name) -> FnDef`
3. Changed `eval_method_call` from `&self` to `&mut self` (needed for `call_function`)
4. Added `Value::Struct` match arm before catch-all:
   - Looks up `(type_name, method)` in `impl_methods`
   - Prepends receiver as `self` parameter
   - Calls `call_function` normally

### Integration Tests (`bmb/tests/integration.rs`)
Added 8 new tests:
- `test_trait_method_dispatch_basic`: Simple trait method call
- `test_trait_method_dispatch_with_args`: Method with additional arguments
- `test_trait_method_dispatch_two_structs`: Same trait, two impls, correct dispatch
- `test_trait_method_dispatch_multiple_methods`: Trait with multiple methods
- `test_trait_method_dispatch_in_function`: Method called inside function
- `test_trait_method_dispatch_chain`: Method result used in computation
- `test_trait_method_dispatch_undefined_method_error`: Unknown method errors correctly
- `test_trait_method_dispatch_bool_return`: Non-i64 return type

## Test Results
- Standard tests: 3287 / 3287 passed (+8 from 3279)
- Clippy: CLEAN (0 errors)
- Build: SUCCESS

## Evaluation
| Criterion | Score | Notes |
|-----------|-------|-------|
| Correctness | 10/10 | All dispatch patterns work, errors handled |
| Architecture | 10/10 | Minimal change, reuses existing call_function |
| Philosophy Alignment | 10/10 | Root cause fix, not workaround |
| Test Quality | 10/10 | Covers basic, args, multi-struct, multi-method, error |
| Code Quality | 10/10 | Clean addition, follows existing patterns |
| **Average** | **10.0/10** | |

## Issues & Improvements
| # | Severity | Description | Action |
|---|----------|-------------|--------|
| I-01 | L | No dynamic dispatch (vtable) — dispatch is static by type name | Language design decision |
| I-02 | L | Trait method on enums not supported | Future enhancement |

## Next Cycle Recommendation
- Missing method detection (impl must provide all trait methods)
- Trait method dispatch for enum types
- Or: Move to Nullable T? lowering (next major feature)
