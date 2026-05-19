# Boundary Values

Clamp a value to the range [lo, hi].

## Input
- First integer: **t** (number of test cases)
- Each test case: **value lo hi** (three integers in this order)

## Output
max(lo, min(hi, value)), one per line (t lines total)

## Examples
(value=5, lo=0, hi=10) → 5; (value=-5, lo=0, hi=10) → 0; (value=15, lo=0, hi=10) → 10

## IMPORTANT: t is the query count

Reading order per test case: **value first**, then lo, then hi.

```
let t: i64 = read_int();
let mut i: i64 = 0;
while i < t {
    let value: i64 = read_int();
    let lo: i64 = read_int();
    let hi: i64 = read_int();
    let clamped: i64 = if value < lo { lo } else if value > hi { hi } else { value };
    println(clamped);
    set i = i + 1;
};
0
```
