# Cycle 357: Naming convention lint rules

## Date
2026-02-13

## Scope
Add snake_case function and PascalCase type naming convention lint rules.

## Philosophy Alignment
| Dimension | Score |
|-----------|-------|
| Core Mission Fit | 5/5 |
| Scope Boundaries | 5/5 |
| Architecture Patterns | 5/5 |
| Dependency Direction | 5/5 |

## Implementation

### Warning Types (error/mod.rs)
Added 2 new warning variants:
- `NonSnakeCaseFunction { name, suggestion, span }` — functions should use snake_case
- `NonPascalCaseType { name, suggestion, kind, span }` — structs/enums/traits should use PascalCase

Warning kinds: `non_snake_case`, `non_pascal_case`

### Utility Functions (util.rs)
Added 4 naming convention helpers:
- `is_snake_case(name)` — check if lowercase + underscores + digits only
- `to_snake_case(name)` — convert camelCase/PascalCase to snake_case
- `is_pascal_case(name)` — check if starts uppercase + no underscores
- `to_pascal_case(name)` — convert snake_case to PascalCase

### Type Checker (types/mod.rs)
Added naming checks in 4 locations:
- **Struct definitions**: Check PascalCase on struct name
- **Enum definitions**: Check PascalCase on enum name
- **Trait definitions**: Check PascalCase on trait name
- **Function definitions**: Check snake_case on function name (skip `main` and `_`-prefixed)

### Tests
- 4 unit tests for naming convention helpers
- 7 integration tests (positive + negative for functions, structs, enums, main exemption)

## Test Results
- Standard tests: 4118 / 4118 passed (+11)
- Clippy: PASS (0 warnings)
- Build: SUCCESS

## Evaluation
| Criterion | Score | Notes |
|-----------|-------|-------|
| Correctness | 10/10 | All convention checks accurate |
| Architecture | 10/10 | Follows existing warning infrastructure |
| Philosophy Alignment | 10/10 | Consistent naming improves AI readability |
| Test Quality | 10/10 | Positive + negative tests |
| Code Quality | 10/10 | Clean utility functions |
| **Average** | **10.0/10** | |

## Issues & Improvements
| # | Severity | Description | Action |
|---|----------|-------------|--------|
| None | - | Clean implementation | - |

## Next Cycle Recommendation
- Cycle 358: Unused parameter detection
