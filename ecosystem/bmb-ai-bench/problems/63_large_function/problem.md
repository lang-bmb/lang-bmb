# Large Function

Given n integers, output 5 statistics on separate lines.

## Input
- First integer: n
- Next n integers: the array

## Output (5 lines)
1. Sum
2. Minimum
3. Maximum
4. Count of even values (divisible by 2, including 0)
5. Count of positive values (strictly > 0)

## Example
`5 1 2 3 4 5` -> sum=15, min=1, max=5, even=2, positive=5

## BMB Notes
- Single pass with 5 accumulators; initialize min/max with the first element
- Even: `v % 2 == 0`; Positive: `v > 0`
```
let n: i64 = read_int();
let first: i64 = read_int();
let mut sum: i64 = first; let mut mn: i64 = first; let mut mx: i64 = first;
let mut even: i64 = if first % 2 == 0 { 1 } else { 0 };
let mut pos: i64 = if first > 0 { 1 } else { 0 };
for _i in 1..n {
    let v: i64 = read_int();
    set sum = sum + v;
    set mn = min(mn, v); set mx = max(mx, v);
    if v % 2 == 0 { set even = even + 1 } else { () };
    if v > 0 { set pos = pos + 1 } else { () }
};
println(sum); println(mn); println(mx); println(even); println(pos);
0
