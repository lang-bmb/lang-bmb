# Acc Recursion

Compute the triangular number sum(1..n) using accumulating recursion.

## Input
- First integer: **t** (number of test cases)
- Each test case: one non-negative integer n

## Output
n*(n+1)/2, one per line (t lines total)

## Examples
n=0→0, n=1→1, n=5→15, n=10→55

## IMPORTANT: t is the query count, not the value

**CRITICAL**: There is NO operation type (no op=1/op=2). Each test case is JUST ONE integer n. Read t first, then loop t times reading ONE n each time.

```
fn main() -> i64 = {
    let t: i64 = read_int();
    let mut i: i64 = 0;
    while i < t {
        let n: i64 = read_int();
        println(n * (n + 1) / 2);
        set i = i + 1
    };
    0
};
```
