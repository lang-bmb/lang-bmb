# Bit Counting

Count the number of 1-bits in the binary representation of n (popcount).

## Input
- First integer: **t** (number of test cases)
- Each of the next t lines: one non-negative integer n

## Output
Popcount of n, one per line (t lines total).

## Examples
n=0→0; n=1→1; n=7(=0b111)→3; n=255(=0xFF)→8

## IMPORTANT: Reading Multiple Queries

The first integer is **t** (query count), NOT the value to compute.

```
let t: i64 = read_int();
let mut i: i64 = 0;
while i < t {
    let n: i64 = read_int();
    // compute popcount of n
    // ...
    set i = i + 1;
};
0
```

## IMPORTANT: Bitwise Operators in BMB

BMB does NOT use `&`, `|`, `^` operators. Use:
- `n band 1` — bitwise AND (NOT `n & 1`)
- `n bor m` — bitwise OR (NOT `n | m`)
- `n bxor m` — bitwise XOR (NOT `n ^ m`)
- `n >> k` — right shift (this syntax IS supported)

## Popcount Algorithm

```
fn popcount(n: i64) -> i64 = {
    let mut count: i64 = 0;
    let mut x: i64 = n;
    while x > 0 {
        set count = count + (x band 1);
        set x = x >> 1;
    };
    count
};
```
