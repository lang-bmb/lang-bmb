# Cycle 2785: D5-B — verify_bench_outputs epsilon FP tolerance
Date: 2026-05-12

## Re-plan

⚪ NONE. D5-B (epsilon FP tolerance in verify_bench_outputs.py) is the next actionable
item from the HANDOFF ordering. store_u8 P0 (D1) is already closed (Cycle 2777). D3
(Rule 6 policy) is 1 cycle. D5-B is faster — proceed.

## Scope & Implementation

**D5-B**: Add `--epsilon 1e-6` to `full-cycle.sh` verify step so that floating-point
precision differences (from different arithmetic order/FMA usage) don't produce false
mismatches.

### Evidence

Current `verify_bench_outputs.py --tier 1` (no epsilon):
- `n_body`: MISMATCH — differences are ~2.8e-7 and ~7.5e-7 relative (LLVM vs GCC FP ordering)
- `fibonacci`: FAIL (C run) — C 6B-iteration bench times out; unrelated to D5-B

With `--epsilon 1e-6`:
- `n_body`: PASS ✅ (differences < 1e-6 relative)
- Tier 1: 9/10 matched (fibonacci C timeout is pre-existing, separate issue)

### Changes Made

**`scripts/full-cycle.sh`** line 210:
```bash
# Before:
python3 "$SCRIPT_DIR/verify_bench_outputs.py" --tier all --rebuild --json "$VERIFY_OUT"
# After:
python3 "$SCRIPT_DIR/verify_bench_outputs.py" --tier all --rebuild --epsilon 1e-6 --json "$VERIFY_OUT"
```

Design note: `verify_bench_outputs.py` already implemented the `--epsilon` logic (Cycle 2769).
D5-B is just activating it in the standard CI invocation.

## Verification & Defect Resolution

| Check | Result |
|-------|--------|
| `verify_bench_outputs.py --tier 1 --epsilon 1e-6` | ✅ 9/10 matched, 0 mismatched |
| `n_body` now PASS | ✅ |
| `fibonacci` still FAIL (C run) | ℹ️ pre-existing — C 6B-iter timeout, tracked in fairness-survey |
| `cargo test --release` | ✅ 23/23 |

## Reflection

Scope fit: ✅ minimal change (1 line), targets exactly the n_body false mismatch.
Philosophy drift: none — epsilon 1e-6 is conservative; actual differences are ~7e-7 max.
Roadmap impact: `full-cycle.sh` now reports 9/10 PASS instead of 8/10 on Tier 1.

fibonacci C timeout is not a D5-B issue — it's a benchmark design issue where the C source
uses 6B iterations for performance timing that GCC can't optimize away. Tracked in
ISSUE-20260512-bench-output-fairness-survey.md. No action needed this cycle.

## Carry-Forward

- Actionable:
  - D3: Rule 6 P0 exception policy clarification in CLAUDE.md (next cycle, 2786)
- Structural Improvement Proposals:
  - fibonacci C timeout: add it to a bench-output "exclude" list or reduce C iteration count
    to a verifiable N (e.g., 1000) while keeping BMB comparison meaningful. P3.
- Pending Human Decisions:
  - D5-A (GitHub Actions verify workflow step) — HUMAN approval needed
  - D7 (npm + PyPI publish)
  - D8 (M4-1 B baseline)
- Roadmap Revisions: None
- Next Recommendation: Cycle 2786 — D3 Rule 6 policy + CLAUDE.md update.
