# Cycle 3244: Bool/u8/i32 Comparison Store Bug Fix (M11-C Phase 9)
Date: 2026-05-28

## Re-plan
Plan valid. Cycle 3243 Carry-Forward: investigate and fix the bug where storing a comparison result (`i < 2`, `i % 2 == 0`, etc.) into a `*bool`, `[bool; N]`, `*u8`, `[u8; N]`, `*i32`, or `[i32; N]` array generates invalid LLVM IR (`store i8 %_t8_cmp` where `%_t8_cmp` is `i1`).

## Scope & Implementation

**Root Cause Discovery**: The bug was in `eliminate_zext_trunc_ir` pass in `bootstrap/compiler.bmb`.

### What `eliminate_zext_trunc_ir` does (intended):
- Pass A: Finds `%dest = zext i1 %src to i64` → records `dest → src` in zext_map
- Pass B: Finds `%result = trunc i64 %src to i1` where `%src` is in zext_map → records `result → original_i1` in aliases
- Pass C: Removes trunc instructions and substitutes the original `i1` source wherever `result` was used

### The Bug:
In `try_add_trunc_alias` (~line 17125), Pass B matched `trunc i64 %X to TYPE` for ANY `TYPE`, not just `i1`. So:
- `%_t8 = zext i1 %_t8_cmp to i64` → recorded: `%_t8 → %_t8_cmp`
- `%_t9_v = trunc i64 %_t8 to i8` → incorrectly aliased: `%_t9_v → %_t8_cmp` (i1)
- `store i8 %_t9_v, ptr %_t9` → substituted to `store i8 %_t8_cmp, ptr %_t9` (i1 as i8 → INVALID!)
- Then `eliminate_dead_zexts_ir` removed `%_t8 = zext ...` (no longer used)
- Final IR: `store i8 %_t8_cmp, ptr %_t9, align 1` — LLVM error: i1 used where i8 expected

### The Fix:
Changed `try_add_trunc_alias` to only match `trunc i64 %X to i1` (not `to i8`, `to i32`):

```bmb
-- Before:
let to_pos = find_pattern_at_slow(rhs, " to ", 10);

-- After:
let to_pos = find_pattern_at_slow(rhs, " to i1", 10);
-- Only eliminate zext i1 → trunc to i1 round-trips (not to i8/i32)
```

### Additional defensive fix (belt-and-suspenders):
Also updated `llvm_gen_store_ptr_u8_sb`, `llvm_gen_store_ptr_i32_sb`, `llvm_gen_store_ptr_bool_sb` to use `zext i1` instead of `trunc i64` when `val.ends_with("_cmp")` (i.e., when the value is a raw comparison result that somehow bypasses the zext). This handles any edge cases where the zext was skipped.

### Golden test: `tests/bootstrap/test_golden_bool_cmp_store.bmb`
- `fill_bool_lt(arr: *bool, n)`: `set arr[i] = (i < 2)` → [1,1,0,0], sum=2 (s1)
- `fill_u8_eq(arr: *u8, n)`: `set arr[i] = (i % 2 == 0)` → [1,0,1,0], sum=2 (s2)  
- `fill_i32_gt(arr: *i32, n)`: `set arr[i] = (i > 1)` → [0,0,1,1], sum=2 (s3)
- Expected: s1+s2+s3 = 6 ✅

## Verification & Defect Resolution

**Bug trace**:
1. First attempt (wrong approach): Fixed `llvm_gen_store_ptr_bool_sb` with `ends_with("_cmp")` check — but this couldn't work because `val = "%_t8"` (not `"%_t8_cmp"`). The root cause was upstream.
2. Investigation revealed `eliminate_zext_trunc_ir` at line 16983 as the culprit.
3. `try_add_trunc_alias` (line 17125) was incorrectly matching `trunc to i8/i32` — the comment said "to i1" but the code checked `" to "` (any type).
4. Root cause fix: change `" to "` to `" to i1"` in `try_add_trunc_alias`.

**Verification**:
- Golden test `test_golden_bool_cmp_store.bmb` → output `6` ✅
- Phase 3-8 golden tests (26 total): all PASS ✅ (build+run method)
- `cargo test --release`: 3800+2390+... all PASS ✅
- Stage 2 bootstrap: ✅ (compiler.exe built, took ~26s compile + 13s link)
- Fixed Point (S3 IR == S4 IR): diff 0 ✅

**IR validation**: After fix, `fill_bool_lt` generates:
```llvm
%_t8_cmp = icmp slt i64 %_t6, 2
%_t8 = zext i1 %_t8_cmp to i64    ← PRESERVED (was being dropped)
%_t9_gp = inttoptr i64 %arr to ptr
%_t9 = getelementptr inbounds nuw i8, ptr %_t9_gp, i64 %_t5
%_t9_v = trunc i64 %_t8 to i8     ← PRESERVED (was being eliminated)
store i8 %_t9_v, ptr %_t9, align 1 ← correct i8 type!
```

## Reflection

**Scope fit**: The comparison-store bug is fully fixed for bool/u8/i32 arrays (both pointer params and stack-allocated).

**Latent defects**: 
- The `eliminate_zext_trunc_ir` bug was masked because `trunc to i1` is the common case (comparisons used in `if` conditions). `trunc to i8/i32` (storing bool values into byte/int arrays) was the rare case that revealed the bug.
- The bug would have affected ALL three typed array stores: `store_ptr_bool`, `store_ptr_u8`, `store_ptr_i32`. The `store_ptr_f64` path goes through a different code path (no i8/i32 truncation) so it was unaffected.

**Structural improvement opportunities**: None beyond what was fixed.

**Philosophy drift**: None. Correct type handling is essential for zero-overhead compilation correctness.

**Roadmap impact**: M11-C Phase 9 complete. This was a critical correctness bug. The belt-and-suspenders fix in `store_ptr_*` functions adds robustness.

## Carry-Forward
- Actionable: None
- Structural Improvement Proposals:
  - `test_golden_stack_i64_subscript` still has no source file (only `.out`). Low priority.
  - The `__bmb_test_tmp.exe` Windows file locking issue causes the test runner to fail non-deterministically. Not a code bug, but worth noting.
- Pending Human Decisions: None
- Roadmap Revisions: M11-C Phase 9 ✅ COMPLETE in ROADMAP.md
- Next Recommendation: Cycle 3245 — explore next language gap (see ROADMAP.md §M11). Possible next: `[f32; N]` arrays, `struct` fields in arrays, or other missing type coverage.
