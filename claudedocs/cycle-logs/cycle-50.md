# Cycle 50: Fix interpreter *i64 pointer indexing + convert bmb-args/bmb-ptr/bmb-sort

## Date
2026-02-08

## Scope
Fix the interpreter's `*i64` typed pointer indexing support (ptr[i] and set ptr[i] = value), then convert remaining recursive workarounds in bmb-args (5), bmb-ptr (2), and bmb-sort (2). Keep genuine recursion (merge_sort_helper, quick_sort_helper divide-and-conquer) as-is.

## Philosophy Alignment
| Dimension | Score |
|-----------|-------|
| Core Mission Fit | 5/5 |
| Scope Boundaries | 5/5 |
| Architecture Patterns | 5/5 |
| Dependency Direction | 5/5 |

## Research Summary
- Root cause: interpreter's `Expr::Index` handler only matched `Value::Array`, `Value::Str`, `Value::StringRope` — `Value::Int` (typed pointer address) fell through to error
- Same issue in `Expr::IndexAssign` and fast eval path
- Fix: add `Value::Int(ptr)` arms that perform pointer arithmetic `ptr + idx * 8` using unsafe load/store (same pattern as existing `load_i64`/`store_i64` builtins)
- 3 locations fixed: regular eval Index, fast eval Index, regular eval IndexAssign

## Implementation

### Interpreter Fix: *i64 pointer indexing (3 locations)

**`bmb/src/interp/eval.rs`**

1. **`Expr::Index` (regular eval, line ~794)**: Added `Value::Int(ptr)` arm with null check + `unsafe { *(ptr + idx * 8) as *const i64 }`
2. **`Expr::Index` (fast eval, line ~1866)**: Same fix for `eval_fast()` path
3. **`Expr::IndexAssign` (line ~840)**: Refactored from array-only to match on value type — `Value::Int(ptr)` does `unsafe { *(ptr + idx * 8) as *mut i64 = value }`, `Value::Array` preserves original behavior

### bmb-args: 5 recursive functions -> while loops

- `has_flag_at` -> inlined into `has_flag` with while + found/done flags
- `find_flag_at` -> inlined into `find_flag` with while + result/done
- `count_positional_rec` -> inlined into `count_positional` with simple while
- `get_positional_rec` -> inlined into `get_positional` with while + done flag
- `parse_int_rec` -> inlined into `parse_int` with while + done flag + accumulator

### bmb-ptr: 2 recursive functions -> while loops

- `i64_array_min_loop` -> inlined into `i64_array_min` with while + min tracking
- `i64_array_max_loop` -> inlined into `i64_array_max` with while + max tracking

### bmb-sort: 2 recursive functions -> while loops (2 genuine recursion kept)

**Functions converted:**
- `heapify` -> while loop with sift-down pattern + done flag
- `is_sorted_helper` -> inlined into `is_sorted` with while + done flag

**Genuine recursion kept:**
- `merge_sort_helper` -- divide-and-conquer (splits range in half)
- `quick_sort_helper` -- divide-and-conquer (partition-based)

### Files Modified
- `bmb/src/interp/eval.rs` (3 locations: Index eval, Index eval_fast, IndexAssign)
- `ecosystem/gotgan-packages/packages/bmb-args/src/lib.bmb`
- `ecosystem/gotgan-packages/packages/bmb-ptr/src/lib.bmb`
- `ecosystem/gotgan-packages/packages/bmb-sort/src/lib.bmb`

## Test Results
| Test Suite | Result |
|------------|--------|
| Rust unit tests | 699/699 PASS |
| Clippy | PASS (0 warnings) |
| bmb-args | 1/3 PASS (2 expected argc failures) |
| bmb-ptr | 8/8 PASS (was 1/8 before fix) |
| bmb-sort | 8/8 PASS (was 0/8 before fix) |
| Ecosystem total | 213/215 PASS (2 expected bmb-args argc failures) |

## Evaluation
| Criterion | Score | Notes |
|-----------|-------|-------|
| Correctness | 10/10 | All fixable tests pass, 15 previously broken tests now work |
| Architecture | 10/10 | Proper interpreter fix at the right level (Principle 2) |
| Philosophy Alignment | 10/10 | Fixed root cause, not workaround; converted 9 more workarounds |
| Test Quality | 9/10 | Comprehensive coverage, only expected argc failures remain |
| Code Quality | 9/10 | Consistent unsafe pointer pattern matching existing builtins |
| **Average** | **9.6/10** | |

## Issues & Improvements
| # | Severity | Description | Action |
|---|----------|-------------|--------|
| I-01 | L | bmb-args test_argc/test_program_name fail in `bmb test` (no CLI args provided) | Expected: test runner doesn't pass argv |
| I-02 | L | Pointer indexing assumes i64 (8 bytes per element) | When *i32/*u8 indexing is needed, add type-aware element size |
| I-03 | M | `desugar_block_lets` scoping bug still requires `{ { } }` pattern | Future cycle: fix parser/desugarer |

## Cumulative Progress (Cycles 42-50)

| Cycle | Scope | Functions Converted | Key Achievement |
|-------|-------|--------------------:|-----------------|
| 42 | 4 compiler bug fixes | -- | float/int ==, free() return, let-in-block MIR, codegen %tmp |
| 43 | Grammar fix (let-in-while) | -- | BlockExpr rules, desugar_block_lets |
| 44 | Dogfood bmb-sha256 + bmb-hashmap | 17 | Validated grammar fix end-to-end |
| 45 | Version bump + commit | -- | v0.89.4, ROADMAP updated |
| 46 | Dogfood bmb-algorithms | 13 | All sorting/search/numeric converted |
| 47 | Dogfood bmb-memchr + bmb-toml | 26 | String/byte search + TOML parser converted |
| 48 | Dogfood bmb-itoa + bmb-fmt + bmb-fs | 15 | Number formatting + path utilities converted |
| 49 | Dogfood bmb-math + bmb-base64 + bmb-rand + bmb-semver | 12 | Math/encoding/PRNG/versioning converted |
| 50 | Fix *i64 interpreter + dogfood bmb-args/bmb-ptr/bmb-sort | 9 | Pointer indexing fixed, 15 tests unblocked |
| **Total** | | **92** | 92 recursive workarounds eliminated across 15 packages |

## Next Cycle Recommendation
**Cycle 51**: Version bump to v0.89.5, update ROADMAP, commit all changes.
