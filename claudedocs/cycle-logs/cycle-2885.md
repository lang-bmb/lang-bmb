# Cycle 2885: for-in-vec Native Porting
Date: 2026-05-15

## Re-plan
Plan valid. Carry-forward from C2884: for-in-vec native using index loop.

## Scope & Implementation

`for x in v` (where v came from vec_new/vec_with_capacity) now generates native index loop:
```
idx = 0; len = vec_len(v);
for_vec_cond: if idx < len → body else exit
for_vec_body: x = vec_get(v, idx); [body]; idx += 1; goto cond
for_vec_exit:
```

**Changes**:
- `bmb/src/mir/mod.rs`: Added `vec_vars: HashSet<String>` field to `LoweringContext`
- `bmb/src/mir/lower.rs`:
  - `Expr::Let`: detect `vec_new`/`vec_with_capacity` call → insert unique_name into `ctx.vec_vars`; propagate membership on Copy
  - `Expr::For`: new `else if Expr::Var(iter_name)` branch that checks `vec_vars`; if vec → index loop, else → existing ChannelRecvOpt path
- Tests: `tests/native_for_in_vec.bmb` (sum=15) + `tests/native_for_in_vec2.bmb` (vec_with_capacity, sum=60)

**Limitation**: Only detects direct `let v = vec_new()` assignments. Function-returned vecs (e.g., `fn make_vec() -> i64 = ...`) not tracked — use manual `for i in 0..vec_len(v)` for those.

## Verification & Defect Resolution

- Native: `bmb build tests/native_for_in_vec.bmb` → output 15 ✅
- Native: `bmb build tests/native_for_in_vec2.bmb` → output 60 ✅  
- `cargo test --release -p bmb` → 2388 PASS, 0 FAIL ✅

## Reflection

- **Scope fit**: Core for-in-vec native ported successfully.
- **Limitation scope**: Function-returned vec handles not tracked. Minor — most usage is `let v = vec_new()` pattern. Add-on tracking (e.g., mark all i64 returns from vec-aware functions as vec handles) deferred.
- **Roadmap impact**: Enables native programs that use vec iteration. Next: svec (BmbSvec) native + for-in-svec, or other remaining interpreter-only builtins.

## Carry-Forward
- Actionable: svec native infrastructure (BmbSvec C struct + for-in-svec index loop)
- Structural Improvement Proposals: Extend vec_vars tracking to function-return sites (low priority)
- Pending Human Decisions: None
- Roadmap Revisions: None
- Next Recommendation: Cycle 2886 — svec native porting (svec_new, svec_push, svec_len, svec_get, for-in-svec)
