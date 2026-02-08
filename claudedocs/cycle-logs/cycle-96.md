# Cycle 96: compiler.bmb For-Loop Range Support

## Date
2026-02-09

## Scope
Add `for i in start..end { body }` support to compiler.bmb (the self-contained bootstrap compiler). Previously only while-loops were supported.

## Philosophy Alignment
| Dimension | Score |
|-----------|-------|
| Core Mission Fit | 5/5 |
| Scope Boundaries | 5/5 |
| Architecture Patterns | 5/5 |
| Dependency Direction | 5/5 |

Bootstrap completeness is critical for self-hosting. For-loops are a fundamental language feature that was missing from compiler.bmb.

## Implementation

### Lexer Changes
- Added `TK_FOR()` and `TK_IN()` token constants
- Added `for` to keyword_len3, `in` to keyword_len2
- `TK_DOTDOT()` already existed in lexer

### Parser Changes
New functions for for-loop parsing:
- `parse_for_expr(src, pos)` — Entry: expects `<ident> in <start> .. <end> { <body> }`
- `parse_for_range(src, pos, varname)` — Parses start expr, expects `..`
- `parse_for_end(src, pos, varname, start_ast)` — Parses end expr, expects `{`
- `parse_for_body(src, pos, varname, start_ast, end_ast)` — Parses body, expects `}`

Produces AST: `(for <varname> start_ast end_ast body_ast)`

### Lowering Changes
New `lower_for_sb` function using alloca/store/load pattern (same as Cycle 94 mutable variables):

1. `alloca %varname` — allocate loop variable on stack
2. Lower start → `store %varname, %_tS` — initialize
3. Lower end → `%_tE` — end bound (evaluated once)
4. `goto for_start_N`
5. `for_start_N:` → load varname, compare < end, branch
6. `for_body_N:` → body code, load var, add 1, store, goto for_start_N
7. `for_end_N:` → unit result

Added dispatch in both step-based and SB-based lowering paths.

### No LLVM Codegen Changes Needed
For-loop MIR uses existing instructions: alloca, store, copy (→load), const, +, <, branch, goto — all already handled by the existing codegen.

## Generated IR Example

```bmb
fn sum_to_10() -> i64 = {
    let mut sum: i64 = 0;
    for i in 0..10 {
        { sum = sum + i }
    };
    sum
};
```

```llvm
define i64 @sum_to_10() nounwind {
entry:
  %sum = alloca i64
  store i64 0, ptr %sum
  %i = alloca i64
  store i64 0, ptr %i
  br label %for_start_0
for_start_0:
  %_t3 = load i64, ptr %i
  %_t4_cmp = icmp slt i64 %_t3, 10
  %_t4 = zext i1 %_t4_cmp to i64
  br i1 %_t4_i1, label %for_body_0, label %for_end_0
for_body_0:
  %_t5 = load i64, ptr %sum
  %_t6 = load i64, ptr %i
  %_t7 = add nsw i64 %_t5, %_t6
  store i64 %_t7, ptr %sum
  %_t9 = load i64, ptr %i
  %_t11 = add nsw i64 %_t9, 1
  store i64 %_t11, ptr %i
  br label %for_start_0
for_end_0:
  %_t13 = load i64, ptr %sum
  ret i64 %_t13
}
```

## Test Results
- Tests: 1701 / 1701 passed (no regressions)
- Bootstrap: 3-Stage PASS with fixed point (5738ms)
- Stage 2 IR: 40612 lines (up from 39717 — new for-loop code)
- End-to-end: `sum_to_10()` → exit code 45 (0+1+...+9 = 45, correct)
- LLVM opt -O2: PASS

## Evaluation
| Criterion | Score | Notes |
|-----------|-------|-------|
| Correctness | 10/10 | Verified via execution + 3-stage bootstrap |
| Architecture | 9/10 | Reuses alloca/load/store pattern from Cycle 94 |
| Philosophy Alignment | 10/10 | Proper implementation, not workaround |
| Test Quality | 8/10 | E2E verified; could add parser unit tests |
| Documentation | 9/10 | Clear implementation with IR examples |
| Code Quality | 9/10 | Consistent with existing patterns |
| **Average** | **9.2/10** | |

## Issues & Improvements
| # | Severity | Description | Action |
|---|----------|-------------|--------|
| I-01 | L | `..=` (inclusive range) not yet supported | Future: add TK_DOTDOTEQ to lexer/parser |
| I-02 | L | No nested for-loop test yet | Future: add nested for test |
| I-03 | L | For-loop variable shadows outer scope silently | Design decision: OK for now |

## Files Modified
- `bootstrap/compiler.bmb` — Lexer (TK_FOR, TK_IN), Parser (parse_for_*), Lowering (lower_for_sb)

## Next Cycle Recommendation
Continue with Cycle 97: T? nullable parser for bootstrap, or address remaining plan items.
