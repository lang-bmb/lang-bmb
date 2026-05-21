# Cycle 2798: lint Rules 15-17 + UTF-8 boundary fix
Date: 2026-05-13

## Re-plan
Plan valid. 14/20+ rules, continuing lint enhancement ISSUE-20260413.
Scope: Rules 15 (negated_comparison), 16 (long_line), 17 (fn_too_many_params).

## Scope & Implementation

**Rules added (3):**

- **Rule 15: negated_comparison** — `not(a == b)` → `a != b`; `not(a < b)` → `a >= b`.
  Uses `find_pattern_outside_str` (new helper) to locate `not(` and then check for `==`/`<`/`>` after it.
  Reduces false positives vs. simple line_contains approach.

- **Rule 16: long_line** — Line > 100 chars. Applied to ALL lines (including comments) via the outer
  loop alongside `check_todo_comment`. Computes `nl - pos` byte length.

- **Rule 17: fn_too_many_params** — Function with 6+ params (5+ commas in param list).
  Uses `count_commas_in_params` helper that scans `(...)` depth-1 commas. Only fires for
  single-line signatures (`)` on same line as `fn`).

**Bug fix (critical):**
Discovered and fixed UTF-8 boundary panic in `line_contains_outside_str`. When a line ends
with a multibyte UTF-8 character (em dash `—`), `s.slice(i, i+plen)` at positions near the
multibyte char could panic with "X is not a char boundary".

Root cause: `check_todo_comment` (called for ALL lines) uses `line_contains_outside_str` with
patterns "TODO"/"FIXME" (plen=4/5). At `i = nl - (plen+2)`, `i + plen` falls inside the
3-byte em dash → panic.

Fix: Add guards before slice:
```
s.byte_at(i) < 128  (start is ASCII = char boundary)
and (i + plen == end or s.byte_at(i + plen) < 128 or s.byte_at(i + plen) >= 192)  (end is char boundary)
```
Applied to `line_contains_outside_str` AND new `find_pattern_outside_str` (replace_all).

**Files changed:**
- `bootstrap/lint/lint.bmb` (14 rules/518 lines → 17 rules/620 lines)
  - New helpers: `find_pattern_outside_str`, `count_commas_in_params`
  - New checks: `check_negated_comparison`, `check_long_line`, `check_fn_too_many_params`
  - UTF-8 boundary fix in `line_contains_outside_str`

## Verification & Defect Resolution

- `cargo test --release`: 23/23 PASS ✓
- `bmb run lint.bmb lint.bmb`: 86 warnings, no panic ✓
- `bmb run lint.bmb selfhost_test.bmb`: 42 warnings, no panic ✓
- Rule 15 (negated_comparison): fires on lint.bmb's own `not(... ==)` checks ✓
- Rule 16 (long_line): fires for 159-char UTF-8 boundary guard line ✓
- Rule 17 (fn_too_many_params): fires for all 6-param check functions in lint.bmb ✓
- UTF-8 fix: no panic on em dash lines ✓

Defects found and resolved:
- **D1**: UTF-8 boundary panic in `line_contains_outside_str` — FIXED immediately.
  First panic (end=4095): required END boundary guard.
  Second panic (start=4176): required START boundary guard.
  Fix applied to both `line_contains_outside_str` and `find_pattern_outside_str` via replace_all.

## Reflection

- **Scope fit**: 3 rules added as planned; UTF-8 fix was a bonus defect resolution.
- **Latent defects**: The UTF-8 boundary bug existed in `line_contains_outside_str` BEFORE this
  cycle — it was latent since no existing check called it on comment lines with em dashes.
  The new `check_todo_comment` calling it on ALL lines surfaced the bug.
- **Philosophy drift**: None. Line-based pattern analysis remains the correct approach.
- **Roadmap impact**: Still 3 more rules needed (18, 19, 20) to reach 20+ target.
- **User-facing quality**: New warnings are actionable and correctly attributed.

## Carry-Forward
- Actionable: Add Rules 18, 19, 20 (need 3 more to reach 20+ ISSUE target).
- Structural Improvement Proposals:
  - Consider adding `bytes_match` helper to avoid slice entirely (more robust than boundary guards).
  - Rule 15 (negated_comparison) may have false positives when `not(` and `==` are unrelated on same line.
- Pending Human Decisions: None.
- Roadmap Revisions: None.
- Next Recommendation: Cycle 2799 — add Rules 18, 19, 20 and close ISSUE-20260413.
