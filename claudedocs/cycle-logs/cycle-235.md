# Cycle 235: Query System & Index Integration Tests

## Date
2026-02-11

## Scope
Add integration tests for Index (IndexGenerator, ProjectIndex, ProofIndex, write/read) and Query (QueryEngine, symbol/function/type/metrics queries).

## Philosophy Alignment
| Dimension | Score |
|-----------|-------|
| Core Mission Fit | 5/5 |
| Scope Boundaries | 5/5 |
| Architecture Patterns | 5/5 |
| Dependency Direction | 5/5 |

## Research Summary
- Index module: IndexGenerator takes parsed AST, produces ProjectIndex
- `gen` is reserved keyword in Rust 2024 — renamed to `indexer`
- ProofEntry fields pre_status/post_status are `Option<ProofStatus>` not `ProofStatus`
- ProjectStats uses `functions`/`types` not `total_functions`/`total_types`
- QueryEngine takes ProjectIndex, provides symbol/function/type/metrics queries
- clippy: `map_or(true, ...)` → `is_none_or(...)`

## Implementation

### Integration Tests (`bmb/tests/integration.rs`)
Added `source_to_index()` helper and 17 new tests:

**Index Generator (7 tests)**
- `test_index_generator_simple_function`: Parse → index → function entry
- `test_index_generator_multiple_functions`: Two functions indexed
- `test_index_generator_struct`: Struct → type entry with kind "struct"
- `test_index_generator_enum`: Enum → type entry with kind "enum"
- `test_index_generator_with_contract`: Pre condition indexed
- `test_index_manifest_counts`: Manifest function/type counts
- `test_index_symbol_entries`: SymbolEntry creation

**Index I/O (1 test)**
- `test_index_write_and_read`: Write → read round-trip with temp dir

**Proof Index (2 tests)**
- `test_index_proof_index_creation`: ProofIndex with ProofEntry
- `test_index_proof_status_variants`: All 6 ProofStatus variants

**Query Engine (7 tests)**
- `test_query_engine_symbols`: Pattern search finds function
- `test_query_engine_symbols_not_found`: Missing pattern → error/empty
- `test_query_engine_function`: Exact function lookup
- `test_query_engine_function_not_found`: Missing function → error
- `test_query_engine_type`: Struct type lookup
- `test_query_engine_metrics`: Project metrics (function/type counts)
- `test_query_engine_functions_with_contracts`: Filter by has_pre

## Test Results
- Standard tests: 2829 / 2829 passed (+17 from 2812)
- Clippy: CLEAN (0 errors)
- Build: SUCCESS

## Evaluation
| Criterion | Score | Notes |
|-----------|-------|-------|
| Correctness | 10/10 | All tests pass |
| Architecture | 9/10 | Tests source→index→query pipeline |
| Philosophy Alignment | 10/10 | AI query system core to BMB's AI-native philosophy |
| Test Quality | 9/10 | Covers generation, I/O, querying |
| Code Quality | 9/10 | Fixed reserved keyword, Option wrapping, field names |
| **Average** | **9.4/10** | |

## Issues & Improvements
| # | Severity | Description | Action |
|---|----------|-------------|--------|
| I-01 | L | query_deps and query_context not tested | Complex dependency setup needed |
| I-02 | L | Batch query not tested | Requires file-based query input |

## Next Cycle Recommendation
- Add CIR verification pipeline integration tests
