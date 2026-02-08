# Cycle 44: Dogfood let-in-while — Simplify Ecosystem Packages

## Date
2026-02-08

## Scope
Validate the Cycle 43 grammar fix by converting recursive workaround functions in bmb-sha256 and bmb-hashmap to use imperative while loops with let bindings.

## Philosophy Alignment
| Dimension | Score |
|-----------|-------|
| Core Mission Fit | 5/5 |
| Scope Boundaries | 5/5 |
| Architecture Patterns | 5/5 |
| Dependency Direction | 5/5 |

Per CLAUDE.md Principle 2: removing workarounds that were forced by a language defect.

## Research Summary
- Identified 32+ recursive workaround functions across ecosystem packages
- bmb-sha256: 7 recursive loop functions (w_fill, w_extend, crounds, sadd, cpybytes, zblk, pfull)
- bmb-hashmap: 10 recursive loop functions (fnv1a, key_eq, copy_key, smap_init, smap_free_keys, smap_put_at, smap_reinsert, smap_get_at, smap_del_at, smap_backshift, smap_rehash)
- Also cleaned up hmap_* functions that used `{ }` nested block workaround

## Implementation

### bmb-sha256 (7 functions converted)
All tail-recursive loop functions converted to while loops:
- `w_fill(w, block)` — removed `i` param, uses `let mut i = 0; while i < 16`
- `w_extend(w)` — removed `i` param, uses `let mut i = 16; while i < 64`
- `crounds(wv, w)` — removed `r` param, uses `let mut r = 0; while r < 64`
- `sadd(s, wv)` — removed `i` param, uses `let mut i = 0; while i < 8`
- `cpybytes(blk, inp, src, dst, n)` — uses `let mut j = 0; while j < n`
- `zblk(blk, start, end)` — uses `let mut i = start; while i < end`
- `pfull(s, w, wv, blk, inp, cnt)` — removed `idx` param, uses `let mut idx = 0; while idx < cnt`
- Updated `pblock()` callers to use new signatures

### bmb-hashmap (10+ functions converted)
- `fnv1a_string(s)` — inlined loop, removed `fnv1a_loop`
- `key_eq(kptr, klen, s)` — inlined loop, removed `key_eq_loop`
- `alloc_key(s)` — inlined loop, removed `copy_key`
- `smap_new()` — inlined init loop, removed `smap_init`
- `smap_free()` — inlined loop, removed `smap_free_keys`
- `smap_put()` — inlined probing, removed `smap_put_at`
- `smap_reinsert()` — converted to while loop with mut swap vars
- `smap_get()` — inlined probing, removed `smap_get_at`
- `smap_delete()` — inlined probing, removed `smap_del_at`
- `smap_backshift()` — converted to while loop
- `smap_grow()` — inlined init and rehash loops, removed `smap_rehash`
- Cleaned up `hmap_*` functions: removed `{ }` block workarounds inside while

### Key Pattern: Assignments in if-else branches
Discovered that variable assignments (`x = value`) are only valid in `BlockStmt` context, not in `Expr` context. Inside if-else branches (which use `SpannedExpr`), assignments must be wrapped in `{ }` blocks:
```bmb
// Wrong: assignment in SpannedExpr context
if cond { done = true; 0 }

// Correct: block wrapping makes assignment a BlockStmt
if cond { { done = true; 0 } }
```

### Files Modified
- `ecosystem/gotgan-packages/packages/bmb-sha256/src/lib.bmb`
- `ecosystem/gotgan-packages/packages/bmb-hashmap/src/lib.bmb`

## Test Results
| Test Suite | Result |
|------------|--------|
| Rust unit tests | 699/699 PASS |
| Clippy | PASS (0 warnings) |
| bmb-sha256 | 9/9 PASS |
| bmb-hashmap | 9/9 PASS |
| All ecosystem (12 packages) | 182/182 PASS |

## Evaluation
| Criterion | Score | Notes |
|-----------|-------|-------|
| Correctness | 10/10 | All tests pass, code produces identical results |
| Architecture | 9/10 | While loops with let are cleaner than tail recursion |
| Philosophy Alignment | 10/10 | Removed workarounds forced by language defect |
| Test Quality | 8/10 | Existing test suites validate correctness end-to-end |
| Documentation | 8/10 | Cycle log documents key pattern (block wrapping for assignments) |
| Code Quality | 9/10 | Simpler control flow, fewer function parameters, eliminated helper functions |
| **Average** | **9.0/10** | |

## Issues & Improvements
| # | Severity | Description | Action |
|---|----------|-------------|--------|
| I-01 | M | bmb-algorithms has 13 recursive workarounds not yet converted | Future cycle |
| I-02 | L | `{ }` block wrapping for assignments in if-else branches is slightly awkward | Language design consideration |
| I-03 | L | Some converted functions (smap_reinsert) are long; consider refactoring | Not urgent |
| I-04 | M | bmb-ptr/bmb-sort still can't run in interpreter (*i64 typed pointers) | Cycle 45 |

## Next Cycle Recommendation
**Cycle 45**: Version bump + commit all changes from Cycles 43-44.
