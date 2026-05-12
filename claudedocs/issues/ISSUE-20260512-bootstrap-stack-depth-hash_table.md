---
id: ISSUE-20260512-bootstrap-stack-depth-hash_table
title: Bootstrap stage1.exe: STATUS_STACK_OVERFLOW on hash_table (226 LOC)
priority: P1
status: open
discovered: 2026-05-12 (Cycle 2780)
---

## Symptom

`bootstrap/stage1.exe` (bootstrap compiler built from `bootstrap/compiler.bmb`) crashes
with STATUS_STACK_OVERFLOW (0xC00000FD) when compiling
`ecosystem/benchmark-bmb/benches/compute/hash_table/bmb/main.bmb` (226 LOC).

Reproducer:
```
bootstrap/stage1.exe build ecosystem/benchmark-bmb/benches/compute/hash_table/bmb/main.bmb -o /tmp/test.exe
# → exit code -1073741571 (STATUS_STACK_OVERFLOW)
```

The Rust-based `target/release/bmb` handles hash_table without issue.

## D2 Context

Cycle 2780 raised the stack reserve on all stage1-style BMB binaries from 1MB → 64MB via
`-Wl,--stack,67108864` in `bmb/src/build/mod.rs`. This is a strict improvement and fixes
overflow on simpler files. hash_table still crashes at 64MB, revealing that the parser
recursion is unbounded for `while` + `if/else if/else` + `@inline` patterns common in
real-world BMB code.

## Root Cause (hypothesis)

`bootstrap/compiler.bmb` parser uses deep recursive descent. The `while` loop body
parser likely recurses through `parse_stmt → parse_expr → parse_if_expr → parse_block → parse_stmt`
without tail-call optimization, creating O(LOC) stack frames. With complex files that have
many nested `if/else if/else` chains and `@inline` attribute parsing, the depth grows well
beyond 64MB.

The Rust compiler avoids this because the lalrpop-generated parser is iterative (LR table-driven).

## Files to Investigate

- `bootstrap/parser.bmb` — entry points: `parse_stmt`, `parse_expr`, `parse_block`
- `bootstrap/compiler.bmb` — driver that calls parser
- Focus: any mutual recursion path from `parse_stmt` through `parse_expr` → `parse_if_chain`

## Proposed Fix

Option A: Convert the deepest recursion chains to iterative (while-loop accumulator)
  - e.g., `parse_if_else_chain` should consume `else if` arms in a loop
  - Targeted refactor; minimal risk

Option B: Add explicit stack depth guard + error message
  - Quick mitigation; doesn't eliminate the recursion

Option C: Increase stack further (128MB+)
  - Not a fix — hash_table is only 226 LOC; production BMB programs will be larger

**Recommended**: Option A for `parse_if_else_chain`, Option B as safety net.

## Why P1

hash_table is a Tier 1 benchmark. If stage1 can't compile it, bootstrap verification
of benchmark equivalence is blocked for an entire category. Real BMB programs with
similar patterns (deeply nested if/else) will also fail to compile with stage1.

## Not P0

The Rust compiler (`target/release/bmb`) is unaffected. CI does not yet test stage1
compilation of benchmark files. Users are not impacted in the short term.
