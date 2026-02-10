# Cycle 192: i32 Full Pipeline Integration Tests

## Date
2026-02-10

## Scope
Add comprehensive i32 integration tests verifying the full pipeline: type checking, interpreter evaluation, casting, bitwise ops, control flow.

## Philosophy Alignment
| Dimension | Score |
|-----------|-------|
| Core Mission Fit | 5/5 |
| Scope Boundaries | 5/5 |
| Architecture Patterns | 5/5 |
| Dependency Direction | 5/5 |

## Research Summary
- i32 is fully implemented in Rust compiler (lexer→codegen) but NOT in bootstrap
- BMB uses `band`/`bor`/`bxor` keywords for bitwise operations (not `&`/`|`/`^`)
- Integer literals default to i64; explicit let bindings needed for i32 context
- Existing i32 tests: only 1 in integration, 1 in eval.rs, 1 in types/mod.rs

## Implementation
- Added 41 new integration tests covering:
  - **Type checking (17 tests)**: basic function, arithmetic, comparison, bitwise, shift, negation, if, let, while, for, cast (i32↔i64, i32↔f64, bool→i32), contracts, modulo, type errors (i32+i64 mismatch, return type mismatch)
  - **Interpreter (24 tests)**: arithmetic (+,-,*,/,%), negation, bitwise (band/bor/bxor), shift (<<,>>), comparison ops, cast (i32→i64, i64→i32, bool→i32, i32→f64, f64→i32), overflow truncation, negative sign-extension, if expression, while loop, recursion

### Findings
1. **Integer literals default to i64**: `1` in expression context is i64, causing type mismatch with i32. Requires explicit `let one: i32 = 1` bindings.
2. **Bitwise operators**: BMB uses keyword syntax (`band`/`bor`/`bxor`) not symbol syntax (`&`/`|`/`^`)
3. **i32 overflow behavior**: `2147483648 as i32` correctly wraps to `-2147483648`
4. **Sign extension**: `-100` stored as i32, cast to i64 properly sign-extends

## Test Results
- Tests: 2334 / 2334 passed (1960 lib + 15 main + 336 integration + 23 doc)
- Build: SUCCESS

## Evaluation
| Criterion | Score | Notes |
|-----------|-------|-------|
| Correctness | 10/10 | All 41 new tests pass |
| Architecture | 9/10 | Follows existing test patterns |
| Philosophy Alignment | 9/10 | Performance-relevant type validation |
| Test Quality | 9/10 | Covers arithmetic, bitwise, cast, control flow, edge cases |
| Documentation | 8/10 | Tests are self-documenting |
| Code Quality | 9/10 | Consistent with codebase style |
| **Average** | **9.0/10** | |

## Issues & Improvements
| # | Severity | Description | Action |
|---|----------|-------------|--------|
| I-01 | M | Integer literal inference: literals always default to i64, no i32 literal syntax | Consider i32 literal suffix (e.g., `42i32`) |
| I-02 | L | No i32-specific benchmark variants yet | Next cycle |

## Next Cycle Recommendation
- Create i32 benchmark variants for collatz/sieve to measure performance impact
- Begin codegen/llvm.rs unit tests (critical gap: 5607 LOC, 0 tests)
