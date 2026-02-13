# Cycle 488: Nullable T? Type Annotation Support in Bootstrap Parser

## Date
2025-02-12

## Scope
Complete nullable T? support by adding `?` token and type annotation parsing to the bootstrap compiler's parser. Also updates roadmap to mark Nullable T? lowering as done.

## Philosophy Alignment
| Dimension | Score |
|-----------|-------|
| Core Mission Fit | 5/5 |
| Scope Boundaries | 5/5 |
| Architecture Patterns | 5/5 |
| Dependency Direction | 5/5 |

## Research Summary
- Bootstrap's `parse_let_skip_type` and `parse_block_let_skip_type` skip type tokens until `=`
- Neither recognized `?` (TK_QUESTION) — caused parse error on `let x: i64? = ...`
- Fix: Add TK_QUESTION token, lexer rule (ASCII 63), and skip logic

## Implementation

### Files Modified
1. **`bootstrap/compiler.bmb`**:
   - Added `TK_QUESTION()` constant (2000000000 + 169)
   - Added `?` (ASCII 63) tokenization in `next_token_raw`
   - Added `TK_QUESTION()` to both `parse_let_skip_type` and `parse_block_let_skip_type` token skip lists

2. **`tests/bootstrap/test_golden_nullable.bmb`**: Updated to use `i64?` type annotations
   - `let x: i64? = 42` (non-null with type annotation)
   - `let y: i64? = null` (null with type annotation)

3. **`docs/ROADMAP.md`**: Marked "Nullable T? 로워링" as ✅ done

### Key Design Decisions
- **Type annotations are skipped, not processed**: Bootstrap compiler ignores type annotations — all values are i64 at MIR level. The `?` is simply consumed and discarded during type skipping.
- **Param/return type parsers not updated**: `parse_param` and `parse_return_type` use direct type matching, not skip logic. Handling `?` there requires a different approach — deferred to future cycle.

## Test Results
| Item | Result |
|------|--------|
| Rust tests | 5,234 passed (2845+47+2319+23) |
| Golden tests (Stage 1) | 27/27 PASS |
| Fixed point (S2==S3) | VERIFIED (74,679 lines) |

## Evaluation
| Criterion | Score | Notes |
|-----------|-------|-------|
| Correctness | 10/10 | All tests pass, T? type annotations work |
| Architecture | 10/10 | Minimal, consistent with existing skip pattern |
| Philosophy Alignment | 10/10 | Completes roadmap item v0.92 |
| Test Quality | 9/10 | Golden test updated with type annotations |
| Documentation | 9/10 | Roadmap updated, version comments |
| Code Quality | 10/10 | Three-line change (token + lexer + skip) |
| **Average** | **9.7/10** | |

## Issues & Improvements
| # | Severity | Description | Action |
|---|----------|-------------|--------|
| I-01 | L | `parse_param` doesn't handle `T?` in function parameters | Future: add ? consumption after base type |
| I-02 | L | `parse_return_type` doesn't handle `T?` return types | Future: add ? consumption after base type |
| I-03 | L | `?` token not handled in generic type contexts (e.g., `Vec<i64?>`) | Future: add to generic type parser |

## Next Cycle Recommendation
- Cycle 489: Bootstrap codegen optimization
  - Eliminate redundant `add nsw i64 0, X` identity copies
  - Or: inline `byte_at` to GEP+load
  - Or: function attributes (memory(none), willreturn)
