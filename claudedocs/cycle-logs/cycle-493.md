# Cycle 493: T? Nullable Type in Function Params and Return Types

## Date
2025-02-12

## Scope
Add `T?` nullable type annotation support to `parse_param` and `parse_return_type` in the bootstrap parser. Previously, `T?` only worked in `let` bindings (via `parse_let_skip_type`). Function signatures like `fn foo(x: i64?) -> String?` would fail to parse.

## Philosophy Alignment
| Dimension | Score |
|-----------|-------|
| Core Mission Fit | 5/5 |
| Scope Boundaries | 5/5 |
| Architecture Patterns | 5/5 |
| Dependency Direction | 5/5 |

## Research Summary
- `TK_QUESTION` (token 169) already defined and handled in lexer (char 63 = `?`)
- `parse_let_skip_type` and `parse_block_let_skip_type` already include `TK_QUESTION()` in their skip lists
- `parse_param` (line 1676) had no `?` handling — stopped at base type, leaving `?` as unexpected token
- `parse_return_type` (line 1769) same issue
- Sub-parsers (`parse_param_array_type`, `parse_param_ref_type`, `skip_array_type_tokens`, `skip_tuple_type_tokens`) also needed updates
- Nullable representation: `null = 0`, non-null = any non-zero value — same i64 representation at codegen level

## Implementation

### Files Modified
1. **`bootstrap/compiler.bmb`**: Added `skip_nullable` helper + updated 7 parser functions
2. **`tests/bootstrap/test_golden_nullable.bmb`**: Added function-level T? tests
3. **`tests/bootstrap/golden_tests.txt`**: Updated expected output (193 → 293)

### New Function
- **`skip_nullable(src, pos) -> i64`**: Consumes `TK_QUESTION()` if present, returns updated position

### Functions Updated
| Function | Change |
|----------|--------|
| `parse_param` | `skip_nullable` on all 7 type branches (i32, i64, f64, bool, String, *T, ident) |
| `parse_param_array_type` | `skip_nullable` after `]` |
| `parse_param_ref_type` | `skip_nullable` on `&T` and `&mut T` branches |
| `parse_return_type` | `skip_nullable` on all 8 type branches |
| `skip_array_type_tokens` | `skip_nullable` after `]` |
| `skip_tuple_type_tokens` | `skip_nullable` after `)` |

### Key Design Decisions
- **`skip_nullable` as separate helper**: Clean, reusable — called from 15 locations
- **No AST type change**: `i64?` still emits `i64` in AST — nullable is semantic, not representational
- **Covers all type forms**: `T?`, `*T?`, `[T; N]?`, `&T?`, `&mut T?`, `(T1, T2)?` all supported

## Test Results
| Item | Result |
|------|--------|
| Rust tests | 5,234 passed (2845+47+2319+23) |
| Golden tests (Stage 1) | 27/27 PASS |
| Fixed point (S2==S3) | VERIFIED (68,486 lines) |

## Evaluation
| Criterion | Score | Notes |
|-----------|-------|-------|
| Correctness | 10/10 | All tests pass, golden test validates both param and return T? |
| Architecture | 10/10 | Clean helper function, consistent application across all type branches |
| Philosophy Alignment | 10/10 | Completes nullable type support across all positions |
| Test Quality | 9/10 | Golden test covers nullable params, return types, null/non-null paths |
| Documentation | 9/10 | Version comments on all changes |
| Code Quality | 10/10 | Minimal, focused changes — single helper called from all positions |
| **Average** | **9.7/10** | |

## Issues & Improvements
| # | Severity | Description | Action |
|---|----------|-------------|--------|
| I-01 | L | Nullable type info not preserved in AST (i64? → i64) | Future: when type checker needs nullable info, add `?` suffix to AST type |
| I-02 | L | No error for `i32??` (double nullable) — silently accepts | Low priority: edge case |
| I-03 | M | Identity copies still 31% of IR | Future: codegen-level copy propagation |

## Next Cycle Recommendation
- Roadmap v0.93 items: byte_at inline, select direct generation, copy propagation
- Or: Additional bootstrap parser features (generics, where clauses)
