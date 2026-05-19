# Empty Input

Given an array (possibly empty), output its sum and count.

## Input
- First integer: n (may be 0)
- Next n integers: the array

## Output (2 lines)
1. Sum (0 if empty)
2. Count (= n)

## Example
`0` -> 0 and 0; `3 1 2 3` -> 6 and 3

## BMB Notes
- `for _i in 0..n` handles n=0 correctly (no iterations)
- Output sum first, then n (no conditional needed)
```
let n: i64 = read_int();
let mut sum: i64 = 0;
for _i in 0..n { sum = sum + read_int() };
println(sum);
println(n);
0
```
