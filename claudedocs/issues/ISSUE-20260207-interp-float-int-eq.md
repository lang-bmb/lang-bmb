# Interpreter float/int `==` mismatch

## Status: RESOLVED (v0.89.4, Cycle 42)

## Severity: HIGH

## Description
The tree-walking interpreter internally coerces i64 values to float during division operations. When the result of such a division is used in an `==` comparison with an integer literal, the comparison fails even when the values are mathematically equal.

## Reproduction
```bmb
fn sqrt_iter(n: i64, guess: i64, prev: i64) -> i64 =
    if guess == prev { guess } else { sqrt_iter(n, (guess + n / guess) / 2, guess) };

fn sqrt(n: i64) -> i64
  pre n >= 0
  post ret * ret <= n
= if n == 0 { 0 } else if n == 1 { 1 } else { sqrt_iter(n, n / 2, 0) };

fn main() -> i64 = {
    let r = sqrt(4);    // Returns 2
    let _a = print(r);  // Prints: 2
    // But:
    let eq = if r == 2 { 1 } else { 0 };
    let _b = print(eq); // Prints: 0 (false!)
    // While:
    let ge = if r >= 2 { 1 } else { 0 };
    let _c = print(ge); // Prints: 1 (true)
    0
};
```

## Expected Behavior
`sqrt(4) == 2` should evaluate to `true` since `sqrt(4)` returns `2`.

## Actual Behavior
- `sqrt(4)` prints as `2`
- `sqrt(4) == 2` evaluates to `false`
- `sqrt(4) >= 2 and sqrt(4) <= 2` evaluates to `true`

## Root Cause
The interpreter's division implementation (`/` operator) likely produces a float `Value::Float` instead of `Value::Int` when dividing two integers. The `==` operator then performs strict type comparison (float != int), while `>=` and `<=` perform numeric comparison that works across types.

## Impact
- Any function using division that returns a value compared with `==` will silently produce wrong results
- This affects Newton's method (sqrt), averaging, modulo-like operations, etc.
- The `>=`/`<=` workaround exists but is non-obvious and error-prone

## Fix Suggestion
In `bmb/src/interp/eval.rs`, ensure that:
1. Integer division of two `Value::Int` always returns `Value::Int` (truncating division)
2. OR: The `==` operator performs numeric coercion (int 2 == float 2.0 → true)

Option 1 is preferred as it maintains type consistency.

## Related
- bmb-math `sqrt` test uses `>=`/`<=` workaround
- This bug does NOT affect compiled code (LLVM backend uses proper i64 division)
