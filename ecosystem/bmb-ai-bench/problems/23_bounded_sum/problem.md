# Bounded Sum

## Description

Sum elements of an array with bounds-checked access using contracts.

The sum function must use a contract to ensure safe array access at every iteration.

**Input** (stdin):
- First integer: `n`, the number of elements (0 <= n <= 100000)
- Next `n` integers: the array elements

**Output** (stdout):
- Print the sum of all elements

## Contract Requirement

The access function must have:
```
pre idx >= 0 and idx < len
```

## Example

Input:
```
4 10 20 30 40
```

Output:
```
100
```

## Constraints

- 0 <= n <= 100000
- Sum fits in a 64-bit signed integer

## Category

Contract (bounds verification)

## BMB Notes
- The contract goes on the accessor helper; main loop can just use for-in
```
fn bounded_get(arr: i64, idx: i64, len: i64) -> i64
    pre idx >= 0 and idx < len
= vec_get(arr, idx);

fn main() -> i64 = {
    let n: i64 = read_int();
    let arr = vec_new();
    for _i in 0..n { vec_push(arr, read_int()) };
    let mut sum: i64 = 0;
    for i in 0..n { sum = sum + bounded_get(arr, i, n) };
    println(sum);
    0
};
```
