# Integer Sqrt

## Description

Compute the integer square root (floor of the square root) of n for multiple queries.

**Input** (stdin):
- First integer: `t`, the number of queries (1 <= t <= 100)
- For each query: a single integer `n` (0 <= n <= 10^18)

**Output** (stdout):
- For each query, print floor(sqrt(n)) on its own line

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

- 1 <= t <= 100
- 0 <= n <= 10^18

## Category

Algorithm (binary search)

## BMB Notes
- t is the query count; for each query read ONE integer n
- Binary search: hi = 1000000000 (since sqrt(10^18) = 10^9)
```
fn isqrt(n: i64) -> i64 = {
    if n == 0 { 0 } else {
        let mut lo: i64 = 0; let mut hi: i64 = 1000000000;
        while lo < hi {
            let mid: i64 = (lo + hi + 1) / 2;
            if mid * mid <= n { lo = mid } else { hi = mid - 1 }
        };
        lo
    }
};

fn main() -> i64 = {
    let t: i64 = read_int();
    for _i in 0..t { println(isqrt(read_int())) };
    0
};
```
