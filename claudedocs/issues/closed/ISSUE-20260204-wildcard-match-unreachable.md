# ISSUE-20260204: Wildcard Pattern in Match Generates Unreachable

## Status: FIXED (v0.60.262)

## Summary

Match expressions with wildcard `_` patterns were incorrectly generating `unreachable` instead of jumping to the wildcard arm's code.

## Symptoms

```bmb
pub fn ordering_is_eq(ord: Ordering) -> bool = match ord {
    Ordering::Equal => true,
    _ => false,  // Should return false for non-Equal cases
};

// Actual behavior:
ordering_is_eq(Ordering::Less)  // Returns true (WRONG)
ordering_is_eq(Ordering::Equal) // Returns true (correct)
ordering_is_eq(Ordering::Greater) // Returns true (WRONG)
```

## Root Cause

In `bmb/src/mir/lower.rs`, the `compile_match_patterns` function correctly identified wildcard arms but did not update the switch default target. The default block was always generated with `Terminator::Unreachable`.

**Generated MIR (before fix)**:
```
fn ordering_is_eq(ord: Ordering) -> bool {
entry:
  switch %ord, [1574 -> match_arm_0_0], match_default_3
match_arm_0_0:
  return B:1
match_default_3:
  unreachable  // <-- WRONG: Should return false
}
```

**Generated LLVM IR (before fix)**:
```llvm
define i1 @ordering_is_eq(ptr %0) {
entry:
  switch i64 %enum_disc, label %match_default_3 [
    i64 1574, label %match_arm_0_0
  ]
match_arm_0_0:
  ret i1 true
match_default_3:
  unreachable  ; LLVM optimizes this away incorrectly
}
```

LLVM's optimizer sees `unreachable` and assumes all switch cases are covered, leading to incorrect code generation that always returns true.

## Fix

Modified `compile_match_patterns` to return `(cases, Option<usize>)` where the second element is the wildcard arm's index if present. The match lowering code now:

1. Uses the wildcard arm's label as the switch default target
2. Only generates the default unreachable block if there's no wildcard

**Generated MIR (after fix)**:
```
fn ordering_is_eq(ord: Ordering) -> bool {
entry:
  switch %ord, [1574 -> match_arm_0_0], match_arm_1_1  ; default → wildcard arm
match_arm_0_0:
  return B:1
match_arm_1_1:
  return B:0  ; Wildcard returns false
}
```

## Files Changed

- `bmb/src/mir/lower.rs`:
  - `compile_match_patterns()`: Returns `(Vec<(i64, String)>, Option<usize>)` instead of just `Vec<(i64, String)>`
  - Match lowering: Uses wildcard arm label as switch default when present

## Verification

```bash
# All tests pass
cargo test --release

# Bootstrap passes
./scripts/bootstrap.sh

# bmb-traits tests pass (999)
./target/x86_64-pc-windows-gnu/release/bmb.exe build \
    packages/bmb-traits/src/lib.bmb -o /tmp/test && /tmp/test
```

## Impact

- All match expressions with wildcard patterns now work correctly
- This was blocking bmb-traits and other packages that use wildcard patterns
- No performance regression (more efficient code without unnecessary default blocks)
