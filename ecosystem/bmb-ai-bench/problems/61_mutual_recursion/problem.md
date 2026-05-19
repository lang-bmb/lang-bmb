# Mutual Recursion

Determine if n is even (1) or odd (0) using mutual recursion.

## Input
- First integer: **t** (number of test cases)
- Each test case: one non-negative integer n

## Output
1 if n is even, 0 if n is odd (one per line, t lines total)

## Algorithm
isEven(0)=1; isEven(n)=isOdd(n-1)
isOdd(0)=0; isOdd(n)=isEven(n-1)

## IMPORTANT: t is the query count, not the value

**CRITICAL**: Do NOT discard t. You MUST loop t times. `let _a = read_int()` discards t and processes only one query.

```
fn is_even(n: i64) -> i64 =
    if n == 0 { 1 } else { is_odd(n - 1) };

fn is_odd(n: i64) -> i64 =
    if n == 0 { 0 } else { is_even(n - 1) };

fn main() -> i64 = {
    let t: i64 = read_int();
    let mut i: i64 = 0;
    while i < t {
        let n: i64 = read_int();
        println(is_even(n));
        set i = i + 1
    };
    0
};
```
