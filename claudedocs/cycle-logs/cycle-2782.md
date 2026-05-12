# Cycle 2782: D2' — Sorting Regression IR-Level Diagnosis
Date: 2026-05-12

## Re-plan

Carry-forward from Cycle 2781: D2' — sorting bench rebuild regression, HANDOFF scope =
"진단 only, fix 안 함". Plan valid. ⚪ NONE.

## Scope & Implementation

**Diagnostic target**: `ecosystem/benchmark-bmb/benches/real_world/sorting/bmb/main.bmb`
rebuild hangs >120s. Feb 9 binary: 234ms, output `403905348`.

**No code changes committed** — diagnostic only per HANDOFF.

### Findings

**Confirmed**: 100% reproducible hang with current compiler. Feb 9 binary unaffected.

**Pre-opt IR** (`/tmp/sorting_current.ll`):
- `quick_sort_helper` and `merge_sort_helper` carry `willreturn mustprogress` attributes
- `norecurse` correctly absent on both

**Post-opt IR** (`/tmp/sorting_opt.ll`) — the bug:
```llvm
; partition.exit block
%_t9.call = tail call fastcc i64 @quick_sort_helper(ptr nonnull %arr, i64 %low, i64 undef)
br label %bb_then_0
```

**Root cause**: LLVM TCE (Tail Call Elimination) + DAE (Dead Argument Elimination) interaction:
1. Second recursive call `quick_sort_helper(arr, pivot+1, high)` → TCE loop-back
2. First recursive call `quick_sort_helper(arr, low, pivot-1)` is NOT tail → DAE sees its
   `high` argument as unused in the TCE loop → substitutes `undef`
3. `bb_then_0` loop header has **NO phi node for `%low`** — original function parameter
   used throughout; `pivot+1` never feeds back into `low`
4. `getelementptr inbounds i64, ptr %arr, i64 %high` hoisted to preheader as loop-invariant

Result: `quick_sort_helper(arr, low, undef)` → UB → undefined termination behavior (hang).

**Rejected hypothesis**: removing `willreturn mustprogress` from recursive functions in
`llvm_text.rs` (tested inline, reverted). Post-opt IR still showed `i64 undef`. These
attributes are not the cause.

**Reverted change**: `bmb/src/codegen/llvm_text.rs` restored to HEAD via
`git checkout -- bmb/src/codegen/llvm_text.rs`.

## Verification & Defect Resolution

No fix applied (HANDOFF "진단 only"). Diagnostic artifacts:
- `/tmp/sorting_current.ll` — pre-opt IR (TCE+DAE input)
- `/tmp/sorting_opt.ll` — post-opt IR (undef visible at `partition.exit`)

ISSUE-20260512-sorting-rebuild-regression.md updated: priority P1→P0 (UB confirmed),
root cause section expanded with IR evidence, hypothesis section updated (willreturn
hypothesis formally rejected), fix direction documented.

## Reflection

Scope fit: ✅ HANDOFF "진단 only" exactly met — root cause identified, no fix applied.
Philosophy drift: none — diagnostic cycle; Rule 6 P0 exception considered but blocked by
HANDOFF scope.
Roadmap impact: sorting P1→P0 escalation; Option A fix is now well-defined (1-2 cycles).
The blocking question is human decision on Rule 6 exception applicability.
User-facing quality: n/a (diagnostic).

## Carry-Forward

- Actionable: None — HANDOFF autonomous range fully complete (D6→D4→D1→D5-B→D5-A→D2→D3→D2')
- Structural Improvement Proposals: None
- Pending Human Decisions:
  - **D2' fix (P0 UB)**: sorting regression fix via pre-opt IR restructuring in
    `llvm_text.rs`. Root cause confirmed. Fix estimated 1-2 cycles. Needs Rule 6 P0
    exception approval (human sign-off) before autonomous execution.
  - D5-A workflow push final approval (CI change)
  - D7 (npm + PyPI publish)
  - D8 (M4-1 B baseline with BMB_BENCH_API_KEY)
  - ISSUE-20260512-bootstrap-stack-depth-hash_table P1 (bootstrap parser unbounded recursion)
- Roadmap Revisions: None
- Next Recommendation: **Early termination** — HANDOFF autonomous range complete.
  Highest-priority autonomous next step: sorting P0 fix (pending human approval).
