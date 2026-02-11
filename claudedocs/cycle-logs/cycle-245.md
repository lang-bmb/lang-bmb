# Cycle 245: Warning System, Error Reporting & Index/Query Integration Tests

## Date
2026-02-11

## Scope
Add integration tests for warning system (TypeChecker warning collection), error reporting (span/message APIs), index generation (IndexGenerator), and query engine (QueryEngine).

## Philosophy Alignment
| Dimension | Score |
|-----------|-------|
| Core Mission Fit | 5/5 |
| Scope Boundaries | 5/5 |
| Architecture Patterns | 5/5 |
| Dependency Direction | 5/5 |

## Research Summary
- LSP Backend requires tower-lsp Client, not directly testable from integration tests
- `gen` is reserved keyword in Rust 2024 — renamed to `idx_gen`
- CompileWarning::generic kind() returns "warning" not "generic"
- QueryResult.matches is Option<Vec<T>>, query_function uses .result (singular)
- ProjectMetrics uses .project.functions not .stats.total_functions
- TypeChecker warning API: add_warning, has_warnings, warnings, take_warnings, clear_warnings

## Implementation

### Integration Tests (`bmb/tests/integration.rs`)
Added 20 new tests:

**Warning System (7 tests)**
- `test_warning_unused_function_detected`: Unused function warning
- `test_warning_unused_binding_detected`: Unused binding API works
- `test_warning_clear_resets`: clear_warnings empties warnings
- `test_warning_add_custom_warning`: Add and retrieve custom warning
- `test_warning_multiple_warnings_accumulated`: Multiple warnings + take_warnings
- `test_warning_unused_struct_detected`: Unused struct warning
- `test_warning_unused_enum_detected`: Unused enum warning

**Error Reporting (4 tests)**
- `test_error_type_error_has_span`: Type error includes span
- `test_error_parse_error_has_span`: Parse error includes span
- `test_error_message_contains_context`: Error message is non-empty
- `test_error_compile_error_display_format`: Display format includes message

**Index Generator (4 tests)**
- `test_index_generator_pub_function_indexed`: Public function indexed
- `test_index_generator_struct_and_enum`: Struct and enum indexed
- `test_index_generator_multiple_files`: Cross-file indexing
- `test_index_function_entries`: Function entry with contract indexed

**Query Engine (5 tests)**
- `test_query_engine_find_function`: Find function by name
- `test_query_engine_find_symbols`: Find symbols by pattern
- `test_query_engine_project_metrics`: Project metrics counting
- `test_query_format_output_json`: JSON output formatting
- `test_query_engine_no_match`: Nonexistent function error

## Test Results
- Standard tests: 3063 / 3063 passed (+20 from 3043)
- Clippy: CLEAN (0 errors)
- Build: SUCCESS

## Evaluation
| Criterion | Score | Notes |
|-----------|-------|-------|
| Correctness | 10/10 | All tests pass |
| Architecture | 9/10 | Tests warning, error, index, query modules |
| Philosophy Alignment | 10/10 | Query/index supports IDE tooling for AI-native development |
| Test Quality | 9/10 | First QueryEngine and IndexGenerator integration tests |
| Code Quality | 9/10 | Fixed gen keyword, QueryResult API, ProjectMetrics API |
| **Average** | **9.4/10** | |

## Issues & Improvements
| # | Severity | Description | Action |
|---|----------|-------------|--------|
| I-01 | L | LSP Backend not directly testable from integration tests | Needs tower-lsp mock |
| I-02 | L | Lint functionality is in main.rs, not library | Can't test as integration |

## Next Cycle Recommendation
- Add Preprocessor, Resolver, or additional TypeChecker integration tests
