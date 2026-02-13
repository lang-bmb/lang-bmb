# Cycle 456: Match Expression Support in Bootstrap Compiler

## Date
2026-02-14

## Scope
Add match expression parsing to compiler.bmb (bootstrap compiler), desugaring to nested if-else chains with let-bound scrutinee. This was the primary recommended follow-up from Cycle 455.

## Philosophy Alignment
| Dimension | Score |
|-----------|-------|
| Core Mission Fit | 5/5 |
| Scope Boundaries | 5/5 |
| Architecture Patterns | 5/5 |
| Dependency Direction | 5/5 |

## Research Summary
- Audited bootstrap compiler's token system: integer tokens pack value as kind (NOT TK_INT()), requiring `is_int_literal()` for detection
- Studied existing parsing patterns (if-chain iterative accumulation, parse_atom dispatching)
- Verified AST format: variables `(var <name>)`, let bindings `(let <name> value body)`, binops `(binop == left right)`
- Confirmed `lower_let_sb` and step-based `LS` work type handle let nodes correctly in MIR lowering

## Implementation
### Files Modified
- `bootstrap/compiler.bmb` — Match expression support:
  1. **Token definitions**: Added `TK_MATCH()` (168) and `TK_FAT_ARROW()` (324) for `=>`
  2. **Keyword recognition**: Added `"match"` to `keyword_len5` dispatch
  3. **Tokenizer**: Added `=>` detection in `next_token_raw` (`c == 61, next == 62`)
  4. **Parser dispatch**: Added `TK_MATCH()` → `parse_match_expr` in `parse_expr`
  5. **Match parsing** (11 new functions):
     - `parse_match_expr` — entry: parse scrutinee, expect `{`
     - `parse_match_open` — expect `{`, start arm iteration
     - `parse_match_arms` — dispatch on pattern type (int literal, `_` wildcard, negative int)
     - `parse_match_arm_arrow` — expect `=>` after pattern
     - `parse_match_arm_body` — parse body, build if-else prefix/suffix
     - `parse_match_arm_sep` — expect `,` or `}` after arm
     - `parse_match_wildcard` — expect `=>` after `_`
     - `parse_match_wildcard_body` — parse default body, assemble result
     - `parse_match_wildcard_end` — expect trailing `,` and `}`

### Key Design Decisions
1. **Desugar to if-else chains**: `match n { 0 => a, 1 => b, _ => c }` becomes `(let <__match> (var <n>) (if (binop == (var <__match>) (int 0)) a (if (binop == (var <__match>) (int 1)) b c)))`. This leverages the existing if/let lowering infrastructure with zero new MIR lowering code.
2. **Let-bound scrutinee**: The scrutinee is bound to `__match` via `let` to avoid re-evaluation. Nested matches shadow correctly via lexical scoping.
3. **Iterative prefix/suffix accumulation**: Same pattern as `parse_if_chain_iter` — builds nested parens from left to right, O(n) in number of arms.

### Bug Found During Implementation
Integer tokens in the bootstrap compiler don't use `TK_INT()` kind — they pack the actual integer value as the kind field. Initially used `kind == TK_INT()` for pattern matching, which failed for all integer patterns. Fixed by using `is_int_literal(kind)` which correctly detects the packed integer token format.

## Test Results
| Test | Status |
|------|--------|
| Rust tests | 5,229 passed |
| Bootstrap Stage 1 | Built successfully |
| Stage 1 == Stage 2 | Fixed point verified (67,972 lines) |
| Golden: basic | 220 |
| Golden: strings | 27 |
| Golden: arrays | 150 |
| Golden: float | 1 |
| Golden: break | 33 |
| Golden: for-in | 141 |
| Golden: loop | 18 |
| Golden: match | 200 (NEW - was FAIL) |
| Golden: struct | FAIL (PARSE - known gap) |

## Evaluation
| Criterion | Score | Notes |
|-----------|-------|-------|
| Correctness | 9/10 | Match desugars correctly, all tests pass, 8/9 golden tests pass |
| Architecture | 10/10 | Zero new MIR code — pure parser-level desugaring leveraging existing infrastructure |
| Philosophy Alignment | 9/10 | Proper implementation, not a workaround; re-uses proven patterns |
| Test Quality | 8/10 | Golden test covers basic integer patterns and wildcard; no nested match or complex body test |
| Documentation | 8/10 | AST format and desugaring strategy documented in code comments |
| Code Quality | 9/10 | Consistent with existing parser function style, proper error messages |
| **Average** | **8.8/10** | |

## Issues & Improvements
| # | Severity | Description | Action |
|---|----------|-------------|--------|
| I-01 | M | Struct init (`Point { x: 3, y: 4 }`) not in compiler.bmb parser — last remaining golden test failure | Add struct init parsing in next cycle |
| I-02 | L | Match only supports integer literal and wildcard patterns — no string, bool, or variable patterns | Extend patterns when needed |
| I-03 | L | No nested match golden test (match inside match arm body) | Add test when needed |
| I-04 | L | Match arm bodies limited to single expressions (no block `{ ... }` bodies in arms) | Block bodies already work via parse_expr → parse_atom → parse_block_expr |

## Next Cycle Recommendation
- Add struct init support to compiler.bmb parser — the last remaining golden test failure
- Struct init `Point { x: 3, y: 4 }` needs: field parsing, struct layout lookup, GEP-based initialization
- This requires more complexity than match since it needs struct type information during codegen
