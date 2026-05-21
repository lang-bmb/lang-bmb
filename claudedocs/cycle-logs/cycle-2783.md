# Cycle 2783: D2' — Sorting P0 UB Fix (MkTuple missing store)
Date: 2026-05-12

## Re-plan

Carry-forward from Cycle 2782: D2' sorting regression fix, Rule 6 P0 exception approved.
Root cause from Cycle 2782: "TCE+DAE interaction" — partially correct (final symptom), but
actual root cause is a codegen bug in `MkTuple` handler. Scope: fix + verify. ⚪ NONE.

## Scope & Implementation

**Target**: `bmb/src/codegen/llvm_text.rs` — `MkTuple` instruction handler.

### Root Cause (refined from Cycle 2782)

Cycle 2782 diagnosed "TCE+DAE interaction" and a `notail` fix was attempted in this cycle
but failed (STATUS_STACK_OVERFLOW + undef still present). Binary-search pass isolation:

1. `opt -passes='inline'` only: `%_t19_ret_load.i = load { i64, i64 }, ptr %_t19.addr.i`
   with **no preceding store** to `%_t19.addr.i` — confirmed missing store at inline boundary
2. `opt -passes='inline,sroa'`: SROA replaces load-without-store → `undef` in `%_t2.load.fca.*`
3. `opt -passes='inline,sroa,instcombine'`: `undef` propagates to `pi` → recursive call args `undef`

The `notail` fix (reverted) blocked `tailcallelim` but not SROA's `undef` replacement.

**Actual bug**: In `MkTuple` handler (`llvm_text.rs` ~7234-7265), after building tuple value
via `insertvalue` chain into `%{dest_name}`, there was NO `store %{dest_name}, ptr %{dest.name}.addr`.
The return handler (`~8176`) later does `load from %{dest.name}.addr` — uninitialized alloca.

### Changes Made

**`bmb/src/codegen/llvm_text.rs`**:
1. Reverted incorrect `notail` change (Cycle 2783 initial attempt):
   ```rust
   // Before (wrong): notail for self-recursive calls
   let call_prefix = if *is_tail { "tail " } else if fn_name == &func.name { "notail " } else { "" };
   // After (correct):
   let call_prefix = if *is_tail { "tail " } else { "" };
   ```

2. Added missing store after insertvalue chain in `MkTuple` handler:
   ```rust
   if local_names.contains(&dest.name) {
       writeln!(out, "  store {} %{}, ptr %{}.addr", struct_type, dest_name, dest.name)?;
   }
   ```

### Pre-opt IR verification (post-fix)

```llvm
; partition function return — now correct:
%_t19_0 = insertvalue { i64, i64 } undef, i64 %_t19_tuple_elem0, 0
%_t19   = insertvalue { i64, i64 } %_t19_0, i64 %_t19_tuple_elem1, 1
store { i64, i64 } %_t19, ptr %_t19.addr   ← new
%_t19_ret_load = load { i64, i64 }, ptr %_t19.addr
ret { i64, i64 } %_t19_ret_load
```

## Verification & Defect Resolution

| Check | Result |
|-------|--------|
| `cargo build --release` | ✅ |
| `cargo test --release` | ✅ all pass |
| `main_fix.exe` output | ✅ `403905348` |
| `main_fix.exe` wall time | ✅ 203ms (ref: Feb 9 = 234ms) |
| Pre-opt IR has store | ✅ line 1392 of sorting_fix.ll |

**500× slowdown eliminated.** No hang, correct output.

## Reflection

Scope fit: ✅ P0 fix, minimal patch (2 changes in 1 file), cargo test passes.
Philosophy drift: none — Rule 6 P0 exception correctly applied, minimum-patch principle followed.
Roadmap impact: sorting Tier 3 measurement now unblocked. M1 ≤1.05x 16/16 hypothesis
can now be re-evaluated with sorting included.
User-facing quality: n/a (codegen fix).

**Cycle 2782 diagnosis partial-correction**: TCE+DAE was the observable symptom, not the
direct cause. The codegen missing-store is the true root — LLVM's SROA correctly identified
that the alloca was never initialized, and replaced the load with `undef`. This is LLVM
doing the right thing given the malformed IR we were emitting.

## Carry-Forward

- Actionable:
  - P1 bootstrap parser `parse_if_else_chain` iterative conversion (next cycle, Cycle 2784)
- Structural Improvement Proposals:
  - `llvm_text.rs` inkwell backend parity: `MkTuple` in `codegen/llvm.rs` may have the same
    missing-store pattern. Low priority (inkwell not default), but worth auditing per Rule 7.
- Pending Human Decisions:
  - D5-A workflow push final approval (CI change)
  - D7 (npm + PyPI publish)
  - D8 (M4-1 B baseline with BMB_BENCH_API_KEY)
  - ISSUE-20260512-bootstrap-stack-depth-hash_table P1 (bootstrap parser unbounded recursion)
- Roadmap Revisions: None
- Next Recommendation: Cycle 2784 — P1 bootstrap parser iterative `parse_if_else_chain` fix.
