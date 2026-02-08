# Cycle 43: Fix Grammar for `let` in While/For/Loop Blocks

## Date
2026-02-08

## Scope
Fix parser grammar (grammar.lalrpop) to allow `let` bindings inside while/for/loop/spawn/closure block bodies. This is the parser-level root cause of ISSUE-20260207-let-in-while-block, completing the fix started in Cycle 42 (MIR scope).

## Philosophy Alignment
| Dimension | Score |
|-----------|-------|
| Core Mission Fit | 5/5 |
| Scope Boundaries | 5/5 |
| Architecture Patterns | 5/5 |
| Dependency Direction | 5/5 |

Per CLAUDE.md Principle 2: "Workaround는 존재하지 않는다" - forcing recursive rewrites instead of imperative while loops with let bindings is a language defect.

## Research Summary
- Analyzed LALRPOP shift/reduce conflict between `BlockStmt`'s `let` and `Expr`'s `let ... ; body`
- Three approaches attempted before finding correct solution:
  1. Direct `let` in `BlockStmt` with `SpannedExpr` → shift/reduce conflict (let reachable from both rules)
  2. Remove `LetUninit` from BlockStmt, strip type annotations → still conflicts
  3. **Working solution**: `BlockExpr` rule (Expr minus let/LetUninit) + `SpannedBlockExpr` wrapper

## Implementation

### Grammar Architecture (grammar.lalrpop)

**Problem**: `BlockStmt` had fallback `SpannedExpr` which reaches `Expr` which includes `let ... = value ; body`. Adding `let` to `BlockStmt` creates ambiguity: when parser sees `let name = value`, should it use BlockStmt's `let` (no body) or Expr's `let` (with `; body`)?

**Solution**: Created `BlockExpr` rule identical to `Expr` but excluding `let`/`LetUninit`. `BlockStmt` uses `SpannedBlockExpr` (wrapping `BlockExpr`) instead of `SpannedExpr`. The parser unambiguously routes `let` tokens to `BlockStmt`'s explicit `let` productions.

```
BlockStmt → "let" name "=" SpannedBlockExpr    (statement-style, no body)
          | "let" name ":" Type "=" SpannedBlockExpr  (typed variant)
          | name "=" SpannedBlockExpr             (assignment)
          | SpannedBlockExpr                      (expression)

BlockExpr → ImpliesExpr | if/else | {block} | match | select
          | while | for | loop | spawn | set | break | continue | return
          | Atomic::new | Mutex::new | channel | RwLock::new | Barrier::new | Condvar::new
          | forall | exists
          (everything in Expr EXCEPT let/LetUninit)
```

### AST Desugaring (ast/expr.rs)

Added `desugar_block_lets()` function that transforms statement-style let sequences into nested Let expressions:

```
Input:  [Let(x, 1, Unit), Let(y, 2, Unit), x + y]
Output: Let(x, 1, Let(y, 2, Block([x + y])))
```

Applied in all block-body grammar productions:
- Block `{ stmts }` (in both Expr and BlockExpr)
- While loop bodies
- For loop bodies
- Loop bodies
- Spawn bodies
- Closure bodies

### Parser Tests (parser/tests.rs)

- Updated `test_parse_while_loop_invariant` to expect `Expr::Let` (from desugaring) instead of `Expr::Block`
- Updated `test_parse_spawn` to expect `Expr::Let` instead of `Expr::Block`
- Added 5 new tests:
  - `test_parse_let_in_while` - basic let in while body
  - `test_parse_let_in_for` - let in for body
  - `test_parse_let_in_loop` - let in loop body with break
  - `test_parse_multiple_lets_in_while` - multiple sequential lets
  - `test_parse_typed_let_in_while` - typed let binding

### Files Modified
- `bmb/src/grammar.lalrpop` (BlockStmt, BlockExpr, SpannedBlockExpr rules; closure bodies)
- `bmb/src/ast/expr.rs` (desugar_block_lets, desugar_stmts functions)
- `bmb/src/parser/tests.rs` (2 updated tests, 5 new tests)
- `claudedocs/issues/ISSUE-20260207-let-in-while-block.md` (RESOLVED)

## Test Results
| Test Suite | Result |
|------------|--------|
| Rust unit tests | 699/699 PASS (was 694, +5 new parser tests) |
| Clippy | PASS (0 warnings) |
| bmb-math | 12/12 PASS |
| bmb-sha256 | 9/9 PASS |
| bmb-hashmap | 9/9 PASS |
| bmb-hash | 8/8 PASS |
| bmb-tree | 8/8 PASS |
| Ecosystem (interpreter-compatible) | 16/21 PASS (same 5 pre-existing failures) |

## Evaluation
| Criterion | Score | Notes |
|-----------|-------|-------|
| Correctness | 10/10 | Grammar compiles without conflicts, all tests pass, let-in-while works |
| Architecture | 9/10 | BlockExpr mirrors Expr but without let/LetUninit; some code duplication is inherent to LALRPOP |
| Philosophy Alignment | 10/10 | Root cause fix at parser level (Decision Framework level 1: language spec) |
| Test Quality | 9/10 | 5 new targeted parser tests covering while/for/loop/typed/multiple |
| Documentation | 9/10 | Issue file fully resolved, grammar well-commented |
| Code Quality | 8/10 | BlockExpr has production duplication with Expr; acceptable for LALRPOP |
| **Average** | **9.2/10** | |

## Issues & Improvements
| # | Severity | Description | Action |
|---|----------|-------------|--------|
| I-01 | L | BlockExpr duplicates ~15 productions from Expr | Acceptable for LALRPOP; alternative would require restructuring entire grammar |
| I-02 | M | Ecosystem packages not yet using let-in-while (still have recursive patterns) | Simplify in future cycle when dogfooding |
| I-03 | L | `desugar_block_lets` creates synthetic spans that may affect error messages | Monitor during testing |
| I-04 | M | bmb-ptr/bmb-sort still fail in interpreter (*i64 typed pointers) | Cycle 45 |

## Next Cycle Recommendation
**Cycle 44**: Dogfood let-in-while by simplifying ecosystem packages (bmb-hashmap, bmb-sha256) that use recursive workarounds. This validates the fix end-to-end with real code.
