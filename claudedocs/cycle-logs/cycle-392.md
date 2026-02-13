# Cycle 392: Type dependency extraction in query system

## Date
2026-02-13

## Scope
Implement the TODO at query/mod.rs:519 — extract type dependencies from function signatures and struct field types in the query system.

## Philosophy Alignment
| Dimension | Score |
|-----------|-------|
| Core Mission Fit | 5/5 |
| Scope Boundaries | 5/5 |
| Architecture Patterns | 5/5 |
| Dependency Direction | 5/5 |

## Implementation

### Core Change
- Added `QueryEngine::extract_named_types()` static method that tokenizes type strings and extracts non-primitive named types
- Filters out primitives: i32, i64, u32, u64, f64, bool, string, unit, char
- Handles: `Point`, `&Point`, `*Point`, `Vec<Point>`, `[Color; 3]`, `Map<K, V>`

### Applied In
1. **`query_function_deps()`** — Extracts type deps from parameter types + return type
2. **`query_type_deps()`** — Extracts type deps from struct field types, excluding self-reference

### Tests (12 new)
| Test | Description |
|------|-------------|
| test_extract_named_types_simple | "Point" → ["Point"] |
| test_extract_named_types_primitive_excluded | "i64" → [] |
| test_extract_named_types_reference | "&Point" → ["Point"] |
| test_extract_named_types_pointer | "*MyStruct" → ["MyStruct"] |
| test_extract_named_types_generic | "Vec<Point>" → ["Vec", "Point"] |
| test_extract_named_types_array | "[Color; 3]" → ["Color"] |
| test_extract_named_types_multiple | "Map<String, Point>" → ["Map", "Point"] (String filtered) |
| test_extract_named_types_no_duplicates | "Pair<Point, Point>" → deduped |
| test_query_function_deps_type_deps_primitive | add(i64, i64) → no type deps |
| test_query_function_deps_type_deps_named | helper(Point) → ["Point"] |
| test_query_type_deps_struct_fields | Point{x: f64, y: f64} → no type deps |
| test_query_type_deps_struct_with_named_field | Line{start: Point, end: Point} → ["Point"] |

## Test Results
- Unit tests: 2175 passed (+12)
- Main tests: 15 passed
- Integration tests: 2179 passed
- Gotgan tests: 23 passed
- **Total: 4392 tests — ALL PASSING**
- Clippy: PASS (0 warnings)
- Build: SUCCESS

## Evaluation
| Criterion | Score | Notes |
|-----------|-------|-------|
| Correctness | 10/10 | All tests pass, TODO resolved |
| Architecture | 10/10 | Follows existing query system patterns |
| Philosophy Alignment | 10/10 | Improves AI query interface |
| Test Quality | 10/10 | 8 unit + 4 integration tests |
| Code Quality | 10/10 | Clean, clippy-clean |
| **Average** | **10.0/10** | |

## Next Cycle Recommendation
- Cycle 393: Proven facts extraction from verification results
