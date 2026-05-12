---
id: ISSUE-20260512-bootstrap-stack-depth-hash_table
title: Bootstrap stage1.exe: int_to_string infinite recursion on i64::MIN
priority: P1
status: RESOLVED (Cycle 2784)
discovered: 2026-05-12 (Cycle 2780)
resolved: 2026-05-12 (Cycle 2784)
---

## Symptom

`bootstrap/stage1.exe` (bootstrap compiler built from `bootstrap/compiler.bmb`) crashes
with STATUS_STACK_OVERFLOW (0xC00000FD) when compiling
`ecosystem/benchmark-bmb/benches/compute/hash_table/bmb/main.bmb` (226 LOC).

Minimal reproducer:
```bmb
fn not_found() -> i64 = 0 - 9223372036854775807 - 1;
fn main() -> i64 = 0;
```
Stage1 exits with -1073741571 (STATUS_STACK_OVERFLOW).

## Confirmed Root Cause (Cycle 2784)

**NOT** parser recursion depth (the original hypothesis was wrong).

The actual cause is `int_to_string` in `bootstrap/compiler.bmb` (line 22):

```bmb
fn int_to_string(n: i64) -> String =
    if n < 0 { "-" + int_to_string(0 - n) } ...
```

For `n = i64::MIN = -9223372036854775808`:
- `n < 0` → true
- `0 - n = 0 - (-9223372036854775808) = i64::MIN` (i64 overflow wraps)
- Infinite recursion → STATUS_STACK_OVERFLOW

The expression `0 - 9223372036854775807 - 1` is constant-folded to `i64::MIN` during
compilation. When the compiler emits this constant as a string literal in the IR,
`int_to_string(i64::MIN)` is called, triggering infinite recursion.

## Bisection Evidence

| Expression | Exit Code | Result |
|-----------|-----------|--------|
| `fn main() -> i64 = 0;` | 0 | ✅ |
| `0 - 9223372036854775807` | 0 | ✅ (-MAX, works) |
| `0 - 9223372036854775806 - 1` | 0 | ✅ (-MAX, different path) |
| `1 - 9223372036854775807 - 1` | 0 | ✅ (-MAX, different start) |
| `0 - 9223372036854775807 - 2` | 0 | ✅ (wraps to MAX-1) |
| `0 - 9223372036854775807 - 1` | -1073741571 | ❌ **i64::MIN** |

## Fix (Cycle 2784)

**File**: `bootstrap/compiler.bmb` lines 22-23

Added `int_to_string_neg` helper that processes negative numbers directly
(without negating them), avoiding the wrap-around overflow:

```bmb
fn int_to_string_neg(n: i64) -> String =
    if n > 0 - 10 { digit_char(0 - n) } else { int_to_string_neg(n / 10) + digit_char(0 - (n - (n / 10) * 10)) };

fn int_to_string(n: i64) -> String =
    if n < 0 { "-" + int_to_string_neg(n) } else if n < 10 { digit_char(n) } else { int_to_string(n / 10) + digit_char(n - (n / 10) * 10) };
```

`n / 10` is always safe for i64::MIN (`-9223372036854775808 / 10 = -922337203685477580`).
Digit extraction `0 - (n - (n/10)*10)` is always in [0,9] for negative inputs.

## Verification

| Check | Result |
|-------|--------|
| `stage1.exe build hash_table/main.bmb` | ✅ exit 0 |
| hash_table output vs Rust compiler | ✅ `95259 100000 46445` (identical) |
| Minimal `0 - i64::MAX - 1` reproducer | ✅ exit 0 |
| `cargo test --release` | ✅ 23/23 pass |
| Stage 1 → stage2 (compiler.bmb self-compile) | ✅ exit 0 |

## Original Hypothesis (Wrong)

The issue was originally diagnosed as parser recursion depth:
> "The while loop body parser likely recurses through parse_stmt → parse_expr → parse_if_expr →
> parse_block → parse_stmt without tail-call optimization, creating O(LOC) stack frames."

This was incorrect. hash_table is only 226 LOC with 3 simple while loops. The real cause
was unrelated to the parser — it was a well-known i64::MIN negation trap in integer-to-string conversion.

The proposed Options A/B/C (iterative parse_if_else_chain, depth guard, stack increase)
were all addressing the wrong root cause and are not needed.

## Carry-Forward

None — issue fully resolved.
