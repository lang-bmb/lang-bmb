# Cycle 279: Array Windows, Chunks, Count, Unique

## Date
2026-02-12

## Scope
Add array methods for sliding windows, chunking, predicate counting, and unique element extraction.

## Philosophy Alignment
| Dimension | Score |
|-----------|-------|
| Core Mission Fit | 5/5 |
| Scope Boundaries | 5/5 |
| Architecture Patterns | 5/5 |
| Dependency Direction | 5/5 |

## Implementation

### Type Checker (`bmb/src/types/mod.rs`)
- `windows(i64) -> [[T]]` — sliding window of given size
- `chunks(i64) -> [[T]]` — partition into fixed-size chunks
- `count(fn(T) -> bool) -> i64` — count elements matching predicate
- `unique() -> [T]` — remove duplicates preserving first occurrence order

### Interpreter (`bmb/src/interp/eval.rs`)
- `windows` — uses Rust's `slice::windows()`, returns empty for oversized window
- `chunks` — uses Rust's `slice::chunks()`, last chunk may be smaller
- `count` — iterates with closure predicate, counts truthy results
- `unique` — O(n²) contains check for simplicity (suitable for interpreter)

### Integration Tests
Added 10 tests covering all methods + edge cases.

## Test Results
- Standard tests: 3520 / 3520 passed (+10 from 3510)
- Build: SUCCESS

## Evaluation
| Criterion | Score | Notes |
|-----------|-------|-------|
| Correctness | 10/10 | All methods work correctly |
| Architecture | 10/10 | Consistent with existing array methods |
| Philosophy Alignment | 10/10 | Extends functional array toolkit |
| Test Quality | 9/10 | Covers normal and edge cases |
| Code Quality | 10/10 | Clean, leverages Rust stdlib |
| **Average** | **9.8/10** | |

## Issues & Improvements
| # | Severity | Description | Action |
|---|----------|-------------|--------|
| I-01 | L | unique() is O(n²) — acceptable for interpreter, codegen should use hashset | Future: codegen optimization |

## Next Cycle Recommendation
- Boolean array methods (any direct, all direct)
- Array swap, rotate methods
- Method chaining improvements
