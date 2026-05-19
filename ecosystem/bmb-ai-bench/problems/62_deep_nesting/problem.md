# Deep Nesting

Count how many times n can be divided by 10 before reaching a single-digit value (0-9). For negative n, return **-1** (not n itself).

## Input
- First integer: **t** (number of test cases)
- Each test case: one integer n

## Output
Nesting depth, one per line (t lines total).

## Examples
- n=5 → 0 (already single-digit)
- n=50 → 1 (50/10=5)
- n=500 → 2
- n=0 → 0
- n=-1 → **-1** (negative: return -1, NOT the original value)
- n=-100 → **-1** (negative: always return -1, NOT -100)

## Algorithm
If n < 0: return -1. Otherwise: depth=0; while n>=10: n=n/10, depth++. Return depth.

**CRITICAL**: For ANY negative n, always return -1, NOT n itself. `if n < 0 { -1 }` not `if n < 0 { n }`.

## IMPORTANT: t is the query count, not the value

```
let t: i64 = read_int();  // number of test cases
let mut i: i64 = 0;
while i < t {
    let n: i64 = read_int();
    println(nesting_depth(n));
    set i = i + 1;
};
0
```
