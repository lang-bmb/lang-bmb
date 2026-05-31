# Climbing Stairs

## Description

You are climbing a staircase. It takes `n` steps to reach the top. Each time you can climb either 1 or 2 steps. Count the number of distinct ways to climb to the top.

**Input** (stdin):
- One integer `n` (1 <= n <= 45)

**Output** (stdout):
- Print the number of distinct ways

## Example

Input:
```
5
```

Output:
```
8
```

(1+1+1+1+1, 1+1+1+2, 1+1+2+1, 1+2+1+1, 2+1+1+1, 1+2+2, 2+1+2, 2+2+1)

## Constraints

- 1 <= n <= 45
- Answer fits in i64

## Category

Algorithm (dynamic programming / Fibonacci)

## BMB Notes
- Same as Fibonacci: ways(n) = ways(n-1) + ways(n-2)
- Base cases: n=1 → 1, n=2 → 2
- Iterative with two variables or vec
```
fn main() -> i64 = {
    let n: i64 = read_int();
    if n == 1 { println(1) }
    else if n == 2 { println(2) }
    else {
        let mut a: i64 = 1; let mut b: i64 = 2;
        for _i in 2..n {
            let c = a + b;
            a = b;
            b = c
        };
        println(b)
    };
    0
};
```
