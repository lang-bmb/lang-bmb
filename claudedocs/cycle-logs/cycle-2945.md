# Cycle 2945: vec_clear native codegen bug + 3 error pattern fixes
Date: 2026-05-19

## Re-plan

Carry-forward from Cycle 2944: "새 언어 갭 탐색 or inttoptr UB P3 착수". Advisor pre-cycle recommendation: probe byte_at in hot path first; then explore GPUStack always-fail problems for language gaps before committing to inttoptr Option A (5-10 cycle scope).

Probe result: csv_parse and http_parse both already use `load_u8(ptr + pos)` — the byte_at→load_u8 optimization noted in ROADMAP was already completed in earlier cycles. No byte_at work needed.

Scope adjusted: GPUStack always-fail language gap analysis + targeted fixes.

## Scope & Implementation

### GPUStack always-fail diagnosis (11 problems across 3 runs)

| Problem | Root cause | Actionable? |
|---------|-----------|-------------|
| 25_range_clamp | Error pattern mismatch: `option_type` fires on `memory(none)` in LLVM IR, giving misleading feedback — model stuck in type C loop forever | ✅ Fix pattern |
| 89_topological_sort | `@vec_clear` undefined in native codegen — function not declared in IR header | ✅ Fix codegen |
| 90_nth_prime | Model uses `if n < 2 { return 0 }` without `;` before next `if` — parser error, but misleading `missing_semicolon_eof` pattern fires (not specific to this case) | ✅ Add new pattern |
| 91_ring_buffer | Algorithm error (doesn't overwrite oldest when full) | ❌ AI issue |
| 79_mini_interpreter | op=5 implemented as divide not dup; op=6 pops instead of peek | ❌ AI issue |
| 28_positive_factorial | AI puts contract on `main()` despite explicit instruction not to | ❌ AI issue |
| 34_power_mod | Wrong algorithm (all-zero output) | ❌ AI issue |
| 39_partial_sum_query | Wrong algorithm | ❌ AI issue |
| 41_collatz_length | Collatz loop not running | ❌ AI issue |
| 71_single_element | Outputs extra line | ❌ AI issue |
| 99_bounded_queue_contract | Actually PASSES (final_correct=True, 1 attempt) — was misclassified | N/A |

**Language gaps found**: 3 fixable. 7 are AI algorithm/instruction-following issues not addressable by compiler changes.

### Fix 1: `vec_clear` native codegen declaration (llvm_text.rs)

Root cause: `@vec_clear` was not declared in the IR header section. The runtime has `void vec_clear(i64)` at line 1538 of `bmb_runtime.c`, and the interpreter has it registered (eval.rs:385), but the text codegen never emitted `declare void @vec_clear(i64)`. Also, `infer_call_return_type` didn't have `vec_clear` → fell through to default `"i64"` → wrong return type.

Changes:
- `bmb/src/codegen/llvm_text.rs` line 1294: add `declare void @vec_clear(i64) nocallback nounwind nosync`
- `bmb/src/codegen/llvm_text.rs` line 9214: add `vec_clear` to void-return list

Golden test added: `tests/golden/test_golden_vec_clear.bmb` — verifies len=3→0 after clear, reuse after push→len=1, val=99.

### Fix 2: `option_type` pattern false-positive (diagnostics/patterns.rs)

Root cause: `option_type` pattern had trigger `"None"` which lowercased matches `"none"` in `memory(none)` inside LLVM IR linker error messages. When GPUStack named its function `clamp` (same as stdlib), the linker error contained LLVM IR with `memory(none)`, triggering `option_type` — a completely wrong hint. Model got stuck in type C loop (same code, same error, wrong feedback).

Fix: removed `"None"` trigger (redundant — `"token \`none\`"` already covers parser errors). Added new `function_name_reserved` pattern with trigger `"invalid redefinition of function"`.

### Fix 3: `if_stmt_no_semicolon` pattern (diagnostics/patterns.rs)

Root cause: Model uses `if n < 2 { return 0 }` (no `;`) followed by another statement. Parser produces "Unrecognized token `if`" which triggers `missing_semicolon_eof` (wrong guidance: suggests top-level `;` not intra-block `;`). Model stuck in type B loop.

Fix: added `if_stmt_no_semicolon` pattern with trigger `"Unrecognized token \`if\`"` and clear guidance: "Add ';' after each if-statement that precedes another statement."

New diagnostics tests: 3 tests in `diagnostics_test.rs` (16 total, was 13).

## Verification & Defect Resolution

```
cargo test --release -p bmb
  lib.rs:          3778/3778 PASSED
  main.rs:           47/47   PASSED
  diagnostics_test:  16/16   PASSED  (was 13, +3 new)
  integration.rs:  2388/2388 PASSED
```

Native codegen verified:
- `bmb run tests/golden/test_golden_vec_clear.bmb` → `3\n0\n99\n1` ✓
- `bmb build tests/golden/test_golden_vec_clear.bmb -o /tmp/test_vec_clear.exe && /tmp/test_vec_clear.exe` → `3\n0\n99\n1` ✓

## Reflection

### Scope fit
- ✅ byte_at probe completed (already load_u8 — no work needed, stale ROADMAP note)
- ✅ 11 always-fail problems analyzed, 3 fixable language gaps identified and fixed
- ✅ vec_clear latent native codegen bug fixed (affects all BMB users — topological_sort unblocked)
- ✅ Error pattern quality improved: 2 fixes + 1 addition

### Roadmap impact
- Always-fail problems potentially fixed: 3 of 11 (89_topological_sort, 25_range_clamp, 90_nth_prime)
- B-axis (GPUStack 85.0%) improvement: 3 fixes can unblock 3 persistent failures → potential gain toward 90%
- ROADMAP: no structural changes needed (P축 stable, B축 note added below)

### Philosophy drift
None. These are proper fixes (compiler bug + error message accuracy), not workarounds.

## Carry-Forward

- Actionable: None
- Structural Improvement Proposals:
  1. **GPUStack always-fail: 7 remaining** are AI algorithm/knowledge issues — investigate if they can be fixed via problem.md clarification (e.g., 91_ring_buffer: overwrite-when-full spec not clear enough)
  2. **inttoptr UB (P3)** — Option A codegen change still pending human decision (5-10 cycles)
  3. **Remaining always-fail**: 34_power_mod, 39_partial_sum_query, 41_collatz_length, 71_single_element, 79_mini_interpreter — investigate if these need problem.md clarification or language additions
- Pending Human Decisions: inttoptr Option A scope approval
- Roadmap Revisions: None
- Next Recommendation: Cycle 2946 → re-run GPUStack benchmark to measure B-axis improvement from these fixes, then pick next language gap or inttoptr Option A planning
