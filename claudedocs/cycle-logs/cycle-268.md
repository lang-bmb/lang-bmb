# Cycle 268: Float Method Support

## Date
2026-02-12

## Scope
Add float (f64) methods to both the type checker and interpreter: `abs`, `floor`, `ceil`, `round`, `sqrt`, `is_nan`, `is_infinite`, `is_finite`, `min`, `max`, `to_int`.

## Philosophy Alignment
| Dimension | Score |
|-----------|-------|
| Core Mission Fit | 5/5 |
| Scope Boundaries | 5/5 |
| Architecture Patterns | 5/5 |
| Dependency Direction | 5/5 |

## Research Summary
- No float methods previously existed — only builtin functions like `sqrt(x)`
- Method syntax `x.sqrt()` is more ergonomic than `sqrt(x)`
- All methods map directly to Rust `f64` methods
- `to_int` provides explicit float→integer conversion

## Implementation

### Type Checker (`bmb/src/types/mod.rs`)
Added `Type::F64` arm in `check_method_call`:
- `abs()`, `floor()`, `ceil()`, `round()`, `sqrt()` → `f64`
- `is_nan()`, `is_infinite()`, `is_finite()` → `bool`
- `min(f64)`, `max(f64)` → `f64` (with type unification)
- `to_int()` → `i64`

### Interpreter (`bmb/src/interp/eval.rs`)
Added `Value::Float(f)` arm in `eval_method_call`:
- All 11 methods implemented using Rust's `f64` methods
- `min`/`max` validate argument type
- `to_int` uses truncation (`as i64`)

### Integration Tests (`bmb/tests/integration.rs`)
Added `run_program_f64` helper and 14 new tests:
- `test_float_abs`, `test_float_floor`, `test_float_ceil`, `test_float_round`
- `test_float_sqrt`
- `test_float_is_nan`, `test_float_is_not_nan`, `test_float_is_infinite`, `test_float_is_finite`
- `test_float_min`, `test_float_max`
- `test_float_to_int`
- `test_float_method_chaining` (abs + ceil)
- `test_float_unknown_method_rejected`

## Test Results
- Standard tests: 3378 / 3378 passed (+14 from 3364)
- Clippy: CLEAN (0 errors)
- Build: SUCCESS

## Evaluation
| Criterion | Score | Notes |
|-----------|-------|-------|
| Correctness | 10/10 | All methods work including edge cases (NaN, Inf) |
| Architecture | 10/10 | Follows String/Array method dispatch pattern |
| Philosophy Alignment | 10/10 | Method syntax more ergonomic for performance-critical float math |
| Test Quality | 10/10 | Covers all methods + chaining + error case |
| Code Quality | 10/10 | Clean, consistent patterns |
| **Average** | **10.0/10** | |

## Issues & Improvements
| # | Severity | Description | Action |
|---|----------|-------------|--------|
| I-01 | L | `to_int` uses truncation, no rounding option | Users can use `.round().to_int()` |
| I-02 | L | No `pow(exp)` method | Could add in future cycle |

## Next Cycle Recommendation
- Integer methods (abs, min, max, to_float)
- Additional array methods (push, pop, sort for interpreter)
- WASM codegen improvements
