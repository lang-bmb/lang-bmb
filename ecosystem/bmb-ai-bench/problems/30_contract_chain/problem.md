# Contract Chain

## Description

Implement a multi-function pipeline where each function's postcondition satisfies the next function's precondition, demonstrating contract propagation.

Pipeline: normalize → scale → bound

1. `normalize(x, min, max)` — maps x from [min,max] to [0, 100]
2. `scale(x, factor)` — multiplies by factor (factor > 0)
3. `bound(x, limit)` — caps at limit

**Input** (stdin):
- First three integers: `min max factor` (min < max, factor > 0)
- Second integer: `limit`
- Third integer: `n`, number of values
- Next `n` integers: the values (each in [min, max])

**Output** (stdout):
- For each value, print the result of bound(scale(normalize(x, min, max), factor), limit)

## Contract Requirement

```
normalize: pre min < max, post ret >= 0 and ret <= 100
scale: pre x >= 0, post ret >= 0
bound: pre limit >= 0, post ret >= 0 and ret <= limit
```

## Example

Input:
```
0 100 2 150 3 0 50 100
```

Output:
```
0
100
150
```

## Constraints

- min < max
- factor > 0
- limit >= 0
- All values in [min, max]

## Category

Contract (multi-function propagation)

## BMB Notes
- Three helper functions each with pre/post; main reads params then calls chain per value
- normalize maps [min,max] → [0,100] via `(x - min) * 100 / (max - min)`
```
fn normalize(x: i64, mn: i64, mx: i64) -> i64
    pre mn < mx
    post ret >= 0 and ret <= 100
= (x - mn) * 100 / (mx - mn);

fn scale(x: i64, factor: i64) -> i64
    pre x >= 0
    post ret >= 0
= x * factor;

fn bound(x: i64, limit: i64) -> i64
    pre limit >= 0
    post ret >= 0 and ret <= limit
= if x > limit { limit } else { x };

fn main() -> i64 = {
    let mn: i64 = read_int(); let mx: i64 = read_int();
    let factor: i64 = read_int(); let limit: i64 = read_int();
    let n: i64 = read_int();
    for _i in 0..n {
        let x: i64 = read_int();
        println(bound(scale(normalize(x, mn, mx), factor), limit))
    };
    0
};
