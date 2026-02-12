# Cycle 362: Tuple methods — len, first, last, swap, to_array, contains

## Date
2026-02-13

## Scope
Add methods for tuple types in both type checker and interpreter.

## Philosophy Alignment
| Dimension | Score |
|-----------|-------|
| Core Mission Fit | 5/5 |
| Scope Boundaries | 5/5 |
| Architecture Patterns | 5/5 |
| Dependency Direction | 5/5 |

## Implementation

### Type Checker (types/mod.rs)
Added `Type::Tuple` method dispatch with 6 methods:
- `len()` → i64: number of elements
- `first()` → T0: first element type
- `last()` → Tn: last element type
- `swap()` → (T1, T0): only for 2-element tuples
- `to_array()` → [T; N]: only when all elements are same type
- `contains(val: T)` → bool: only when all elements are same type

Includes "did you mean?" suggestions in catchall.

### Interpreter (interp/eval.rs)
Added `Value::Tuple` method dispatch:
- `len`, `first`, `last`, `swap`, `to_array`, `contains` — runtime implementations

### Tests
- 8 integration tests: len, first, last, swap, contains, to_array type check, to_array mixed error, unknown method

## Test Results
- Standard tests: 4175 / 4175 passed (+8)
- Clippy: PASS (0 warnings)
- Build: SUCCESS

## Evaluation
| Criterion | Score | Notes |
|-----------|-------|-------|
| Correctness | 10/10 | Type-safe tuple methods with proper validation |
| Architecture | 10/10 | Follows existing method dispatch pattern |
| Philosophy Alignment | 10/10 | Compile-time type safety for heterogeneous tuples |
| Test Quality | 10/10 | Runtime + type-check + error tests |
| Code Quality | 10/10 | Clean, consistent with other type handlers |
| **Average** | **10.0/10** | |

## Issues & Improvements
| # | Severity | Description | Action |
|---|----------|-------------|--------|
| None | - | Clean implementation | - |

## Next Cycle Recommendation
- Cycle 363: String regex-like methods
