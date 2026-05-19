# Fibonacci

## Description

Compute the n-th Fibonacci number using iterative dynamic programming.

F(0) = 0, F(1) = 1, F(n) = F(n-1) + F(n-2)

**Input** (stdin):
- Single integer: `n` (0 <= n <= 90)

**Output** (stdout):
- Print F(n)

## Example

Input:
```
10
```

Output:
```
55
```

## Constraints

- 0 <= n <= 90
- Result fits in a 64-bit signed integer
- Must use iterative approach (not recursive) for efficiency

## Category

Algorithm (dynamic programming)

## BMB Notes
- Iterative with two variables — no array needed
- Handle n=0 (output 0) and n=1 (output 1) as base cases before the loop
```
let n: i64 = read_int();
if n == 0 { println(0) } else {
    let mut a: i64 = 0; let mut b: i64 = 1; let mut i: i64 = 1;
    while i < n { let tmp: i64 = b; set b = a + b; set a = tmp; set i = i + 1 };
    println(b)
};
0

```
