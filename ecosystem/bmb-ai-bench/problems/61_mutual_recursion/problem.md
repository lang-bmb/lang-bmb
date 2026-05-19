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

```
let t: i64 = read_int();  // number of test cases
let mut i: i64 = 0;
while i < t {
    let n: i64 = read_int();
    println(is_even(n));
    set i = i + 1;
};
0
```
