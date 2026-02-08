# Cycle 52: Fix if-branch block statement parsing

## Date
2026-02-08

## Scope
Fix the grammar so that `if`/`else` branches accept `BlockStmt` sequences (assignments, let bindings) directly, eliminating the `{{ }}` double-block workaround pattern (Issue I-03 from Cycles 47-50).

## Philosophy Alignment
| Dimension | Score |
|-----------|-------|
| Core Mission Fit | 5/5 |
| Scope Boundaries | 5/5 |
| Architecture Patterns | 5/5 |
| Dependency Direction | 5/5 |

**Principle 2 ("Workaround는 존재하지 않는다")**: The `{{ }}` double-block pattern was a workaround forced by a grammar limitation. This cycle fixes the root cause at the grammar level (Decision Framework Level 1: Language Spec).

## Root Cause Analysis

### Problem
`if`-branches used `"{" <t:SpannedExpr> "}"` in the grammar, which only allowed a single expression inside braces. Variable assignments (`name = value`) are parsed as `BlockStmt` (not `Expr`), so `{ count = count + 1; 0 }` failed with a parser error.

The workaround was double blocks: `{ { count = count + 1; 0 } }` — the outer braces are the `if` syntax, the inner braces create a block (which IS an `Expr`).

### Fix
Changed all 6 `if`/`else` grammar rules across 3 nonterminals (`SpannedIfExpr`, `BlockExpr`, `Expr`) to accept `BlockStmt` sequences with `desugar_block_lets`, matching the pattern already used by `while`/`for`/`loop`/`spawn`/closures.

**Before**: `"if" <c:SpannedExpr> "{" <t:SpannedExpr> "}" "else" "{" <e:SpannedExpr> "}"`
**After**: `"if" <c:SpannedExpr> <tl:@L> "{" <tes:(<BlockStmt> ";")*> <tlast:BlockStmt?> "}" <tr:@R> "else" ...`

### Secondary Fix
The grammar change caused `if`-branches to wrap single expressions in `Expr::Block(...)`, which broke `collect_calls` in `index/mod.rs` (didn't traverse into `Block` nodes). Added `Expr::Block` arm to `collect_calls`.

## Changes

### grammar.lalrpop (6 rules updated)
1. `SpannedIfExpr` if-else rule (line ~1201)
2. `SpannedIfExpr` if-else-if rule (line ~1207)
3. `BlockExpr` if-else rule (line ~1288)
4. `BlockExpr` if-else-if rule (line ~1293)
5. `Expr` if-else rule (line ~1435)
6. `Expr` if-else-if rule (line ~1441)

### index/mod.rs
- Added `Expr::Block` arm to `collect_calls` method

### parser/tests.rs (3 new tests)
- `test_parse_assign_in_if_branch` — single block assignments in if/else/else-if
- `test_parse_let_in_if_branch` — let bindings in if-branches
- `test_parse_multi_stmt_if_branch` — multiple statements in if-branch

### Files Modified
- `bmb/src/grammar.lalrpop`
- `bmb/src/index/mod.rs`
- `bmb/src/parser/tests.rs`

## What This Enables
```bmb
// BEFORE (workaround): double blocks required
let _r = if condition { { count = count + 1; 0 } } else { 0 };

// AFTER (proper): single block works
let _r = if condition { count = count + 1; 0 } else { 0 };

// NEW: let bindings in if-branches
let result = if x > 0 {
    let doubled = x * 2;
    doubled
} else {
    let halved = x / 2;
    halved
};
```

## Test Results
| Test Suite | Result |
|------------|--------|
| Rust unit tests | 702/702 PASS (was 699, +3 new parser tests) |
| Clippy | PASS (0 warnings) |
| Ecosystem | 213/215 PASS (2 expected bmb-args argc failures) |

## Evaluation
| Criterion | Score | Notes |
|-----------|-------|-------|
| Correctness | 10/10 | All tests pass, grammar is conflict-free |
| Architecture | 10/10 | Fix at grammar level (Decision Framework Level 1) |
| Philosophy Alignment | 10/10 | Eliminates workaround at root cause |
| Test Quality | 10/10 | 3 new parser tests covering assign/let/multi-stmt |
| Code Quality | 10/10 | Consistent with existing while/for/loop pattern |
| **Average** | **10.0/10** | |

## Issues & Improvements
| # | Severity | Description | Action |
|---|----------|-------------|--------|
| I-01 | L | `desugar_block_lets` wraps single expressions in `Block` unnecessarily | Could optimize to skip Block wrapping for single non-let expressions |
| I-02 | - | All `{{ }}` patterns in ecosystem packages can now be simplified | Next cycle: Cycle 53 |
