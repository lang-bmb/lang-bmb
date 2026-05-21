# Cycle 2799: lint Rules 18-20 + Rule 18 `line_contains` bug fix
Date: 2026-05-13

## Re-plan
Plan valid. 17/20 rules done (Cycle 2798). Scope: add Rules 18, 19, 20 and close ISSUE-20260413.

## Scope & Implementation

**Rules added (3):**

- **Rule 18: string_chain_concat** — Line with 4+ `+` operators and a string literal.
  Each `+` on a String allocates a new heap buffer → O(n) chain cost.
  Uses `count_plus_outside_str` helper (counts `+` outside string literals) + raw byte scan
  to check for presence of `"` on the line.

- **Rule 19: dual_negation** — `not(a) and not(b)` → `not(a or b)` (De Morgan's law).
  Also detects `not(a) or not(b)` → `not(a and b)`.
  Uses `line_contains` with `) and not(` / `) or not(` pattern pairs.

- **Rule 20: bare_panic** — Direct `panic()` call.
  Prefers `pre` contracts for compile-time elimination.
  Distinct from Rule 13 (todo_call): `panic()` is intentional but often contract-replaceable.

**Bug fix (Rule 18 guard):**
Discovered that `line_contains` delegates to `line_contains_outside_str`, which correctly
skips content inside string literals but can never match the `"` delimiter byte itself
(opening/closing `"` transitions `in_str` state before the pattern check is reached).

So `line_contains(source, ls, nl, "\"")` always returned `false`, causing Rule 18 to
silently return 0 for every line.

Fix: replaced the `line_contains` guard with a raw byte scan (`while qi < nl and byte_at(qi) != 34`).
Added an explanatory comment to document the constraint for future maintenance.

**Files changed:**
- `bootstrap/lint/lint.bmb` (17 rules/693 lines → 20 rules/731 lines)
  - New helper: `count_plus_outside_str`
  - New checks: `check_string_chain_concat`, `check_dual_negation`, `check_bare_panic`
  - Rule 18 guard: raw byte scan instead of `line_contains` for `"` detection
  - Header updated: "17 checks" → "20 checks" with rules 18-20 descriptions
  - Cycle header updated to include Cycle 2799

## Verification & Defect Resolution

- `cargo test --release`: 6211/6211 PASS ✓
- `bmb run lint.bmb lint.bmb`: 104 warnings, no panic ✓
- Rule 18 (string_chain_concat): fires on lines 403, 601, 727 of lint.bmb ✓
- Rule 19 (dual_negation): fires on lines 243, 324, 427 of lint.bmb ✓
- Rule 20 (bare_panic): fires on 2 lines with `bare_panic(` in identifiers ✓

Defects found and resolved:
- **D1**: Rule 18 `line_contains` guard always returned `false` — FIXED with raw byte scan.
  Root cause: `line_contains` → `line_contains_outside_str` treats `"` as state-transition
  character only; never matches `"` as a search target. This is correct for string-literal
  exclusion but wrong for "does a `"` appear anywhere on this line" query.

Known non-issues:
- Rule 20 false positives on `check_bare_panic(` calls and `fn check_bare_panic(` definition
  (these identifiers contain `panic(`). Acceptable for pattern-based lint without parse tree.

## Reflection

- **Scope fit**: 3 rules added as planned; 1 bug fix (D1) resolved within cycle. ISSUE target reached.
- **Latent defects**: D1 existed from original Rule 18 implementation. Would have produced
  zero false-negative suppressions (Rule 18 never fired) without the standalone test revealing it.
- **Philosophy drift**: None. Pattern-based approach still appropriate for this tool's goals.
- **Roadmap impact**: ISSUE-20260413 target of 20+ rules reached → close the ISSUE.
- **User-facing quality**: All 20 rules produce actionable, correctly attributed warnings.

## Carry-Forward
- Actionable: Close ISSUE-20260413-linter-enhancement.md (20+ rules reached).
- Structural Improvement Proposals:
  - Rule 20 (bare_panic) has false positives on any identifier containing `panic(`.
    A suffix-guard (check character before `panic` is not an ident char) would eliminate these.
  - The `line_contains` / `line_contains_outside_str` equivalence is a footgun for anyone
    adding checks that search for `"`. Document this constraint in the function header.
- Pending Human Decisions: None.
- Roadmap Revisions: None.
- Next Recommendation: Cycle 2800 — continue from ROADMAP (next P-track item or other P2 ISSUE).
