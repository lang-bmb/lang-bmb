# Cycle 2784: P1 bootstrap int_to_string i64::MIN fix
Date: 2026-05-12

## Re-plan

Inherited from Cycle 2783: P1 bootstrap parser `parse_if_else_chain` iterative conversion.
Issue document diagnosed overflow as parser recursion depth; this was contradicted by evidence
(hash_table has only 3 `else if` chains — impossible to overflow 64MB with 3 recursive calls).
🟡 SCOPE ADJUST: bisect hash_table to find actual cause, then fix.

## Scope & Implementation

**Target**: `bootstrap/compiler.bmb` — `int_to_string` function (line 22).

### Bisection (STEP 1)

Isolated minimum reproducer by binary-removing functions from hash_table:
1. Removed all while-loop functions → still crashes (ruling out parser recursion)
2. Removed all entry/hm functions → still crashes (just `not_found + main`)
3. Tried `fn main() -> i64 = 0;` alone → works
4. Isolated to: `fn not_found() -> i64 = 0 - 9223372036854775807 - 1;`

Precision test matrix:
- `0 - 9223372036854775807` → works
- `0 - 9223372036854775806 - 1` → works
- `1 - 9223372036854775807 - 1` → works
- `0 - 9223372036854775807 - 2` → works
- `0 - 9223372036854775807 - 1` → OVERFLOW (only `i64::MIN`)

### Root Cause (confirmed)

`int_to_string` at line 22:
```bmb
fn int_to_string(n: i64) -> String =
    if n < 0 { "-" + int_to_string(0 - n) } else ...
```

For `n = i64::MIN`: `0 - i64::MIN = i64::MIN` (i64 wrap-around) → infinite recursion.

The expression `0 - 9223372036854775807 - 1` constant-folds to `i64::MIN`. When the compiler
emits this constant in LLVM IR, it calls `int_to_string(i64::MIN)` → infinite recursion.

The issue document's parser-recursion hypothesis was wrong. hash_table has only 3 while loops
and ~3 `else if` chains — nowhere near deep enough to overflow 64MB stack.

### Changes Made

**`bootstrap/compiler.bmb`** lines 22-25:
```bmb
// Before:
fn int_to_string(n: i64) -> String =
    if n < 0 { "-" + int_to_string(0 - n) } else if n < 10 { digit_char(n) } else { int_to_string(n / 10) + digit_char(n - (n / 10) * 10) };

// After:
fn int_to_string_neg(n: i64) -> String =
    if n > 0 - 10 { digit_char(0 - n) } else { int_to_string_neg(n / 10) + digit_char(0 - (n - (n / 10) * 10)) };

fn int_to_string(n: i64) -> String =
    if n < 0 { "-" + int_to_string_neg(n) } else if n < 10 { digit_char(n) } else { int_to_string(n / 10) + digit_char(n - (n / 10) * 10) };
```

`int_to_string_neg` handles negative numbers directly without negating them:
- Base case: `n > -10` → single digit, `0 - n ∈ [1,9]`
- Recursive case: `n/10` is always safe (no overflow), digit `0-(n-(n/10)*10) ∈ [0,9]`

## Verification & Defect Resolution

| Check | Result |
|-------|--------|
| Minimal i64::MIN reproducer | ✅ exit 0 (was -1073741571) |
| `stage1.exe build hash_table/main.bmb` | ✅ exit 0 (was STATUS_STACK_OVERFLOW) |
| hash_table output vs Rust compiler | ✅ `95259 100000 46445` (identical) |
| `cargo test --release` | ✅ 23/23 |
| Stage 1 bootstrap (stage1 → stage2_test) | ✅ exit 0 |

## Reflection

Scope fit: ✅ P1 fix, minimal patch (2 lines added, 1 line changed).
Philosophy drift: none — proper fix for a well-known i64::MIN trap, not a workaround.
Issue document correction: Original hypothesis (parser recursion) was wrong. Updated issue
document with confirmed root cause, bisection evidence, and fix.
Roadmap impact: hash_table benchmark can now be compiled and verified with stage1. Stage 1
bootstrap coverage of Tier 1 benchmarks unblocked.

The i64::MIN negation trap (`0 - i64::MIN = i64::MIN`) is a classic correctness issue.
Every correct int-to-string implementation must handle this separately.

## Carry-Forward

- Actionable:
  - Check other `.bmb` files with separate `int_to_string` definitions (types.bmb, mir.bmb, etc.)
    that are built as standalone binaries (not compiler.bmb) — apply same fix if they contain
    the `0 - n` negation pattern. Low priority (only compiler.bmb builds stage1/stage2/stage3).
- Structural Improvement Proposals:
  - None
- Pending Human Decisions:
  - D5-A workflow push final approval (CI change)
  - D7 (npm + PyPI publish)
  - D8 (M4-1 B baseline with BMB_BENCH_API_KEY)
- Roadmap Revisions: None
- Next Recommendation: Cycle 2785 — verify Tier 1 benchmarks compile with stage1 (hash_table
  unblocked; check other Tier 1 files for similar i64::MIN patterns).
