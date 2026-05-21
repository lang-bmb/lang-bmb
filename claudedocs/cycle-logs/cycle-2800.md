# Cycle 2800: lint Rule 20 false positive fix + line_contains constraint doc
Date: 2026-05-13

## Re-plan
Carry-forward from Cycle 2799: Rule 20 (bare_panic) false positives on identifiers
containing `panic(` (e.g., `check_bare_panic(`). Structural improvement proposal →
actionable defect-level fix for a 1-cycle scope.

## Scope & Implementation

**Rule 20 false positive fix:**

Root cause: `check_bare_panic` used `line_contains(source, ls, nl, "panic(")`, which
correctly skips string literals but cannot distinguish `panic(` as a bare call vs.
`check_bare_panic(` as an identifier containing "panic(".

Fix: switch to `find_pattern_outside_str` which returns the byte position of "panic(",
then check if the preceding character is an identifier char (`is_ident_char`). If true,
skip — it's part of a larger identifier, not a bare `panic()` call.

```bmb
let panic_pos = find_pattern_outside_str(source, ls, nl, "panic(");
if panic_pos == -1 { 0 }
else if panic_pos > ls and is_ident_char(source.byte_at(panic_pos - 1)) { 0 }
else { emit_warning(...) }
```

**`line_contains_outside_str` constraint documentation:**
Added inline NOTE comment explaining that the function cannot find `"` (byte 34) as a
pattern — the quote byte is consumed as a state-transition delimiter, never reaching
the pattern-match branch. Documents the workaround (raw byte scan) for future
maintenance.

**Files changed:**
- `bootstrap/lint/lint.bmb` (731 lines → 735 lines)
  - `check_bare_panic`: `line_contains` → `find_pattern_outside_str` + ident-char guard
  - `line_contains_outside_str`: NOTE comment added

## Verification & Defect Resolution

- `cargo test --release`: 6211/6211 PASS ✓
- `bmb run lint.bmb lint.bmb`: 102 warnings (was 104 — 2 FP removed), no panic ✓
- Rule 20 no longer fires for `fn check_bare_panic(` or `check_bare_panic(source,` ✓
- Rule 20 would still fire for genuine `panic(` calls (bare call, preceded by space/`=`) ✓

No defects found.

## Reflection

- **Scope fit**: Minimal fix for a specific false positive. No over-engineering.
- **Latent defects**: None found.
- **Philosophy drift**: None.
- **Roadmap impact**: None. Lint quality improvement only.
- **User-facing quality**: Rule 20 is now precision-improved — no more noise from function names.

## Carry-Forward
- Actionable: None.
- Structural Improvement Proposals:
  - Similar false positive pattern could exist for Rule 13 (todo_call): identifiers
    containing `todo(` would also trigger. `check_todo_call` should apply the same
    ident-char prefix guard. However, there are currently no identifiers with `todo(`
    in lint.bmb, so this is a latent issue.
- Pending Human Decisions: None.
- Roadmap Revisions: None.
- Next Recommendation: Cycle 2801 — SIMD codegen analysis (P1 ISSUE) or next P2 work.
