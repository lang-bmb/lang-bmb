# Safe Integer Square Root

## Description

Compute the integer square root (floor of sqrt) with a contract ensuring the input is non-negative.

**Input** (stdin):
- First integer: `n`, number of queries (1 <= n <= 1000)
- Next `n` integers: the values (all >= 0)

**Output** (stdout):
- For each value, print its integer square root (floor)

## Contract Requirement

```
pre x >= 0
post ret >= 0 and ret * ret <= x
```

## Example

Input:
```
4 0 1 4 10
```

Output:
```
0
1
2
3
```

## Constraints

- 1 <= n <= 1000
- 0 <= x <= 10^18

## Category

Contract (non-negative verification)

## BMB Notes
- Contract: `pre x >= 0  post ret >= 0 and ret * ret <= x`
- Use binary search for x up to 10^18
```
fn isqrt(x: i64) -> i64
    pre x >= 0
    post ret >= 0 and ret * ret <= x
= {
    if x == 0 { 0 } else {
        let mut lo: i64 = 0; let mut hi: i64 = 1000000000;
        while lo < hi {
            let mid: i64 = (lo + hi + 1) / 2;
            if mid * mid <= x { lo = mid } else { hi = mid - 1 }
        };
        lo
    }
};

fn main() -> i64 = {
    let n: i64 = read_int();
    for _i in 0..n { println(isqrt(read_int())) };
    0
};
```
