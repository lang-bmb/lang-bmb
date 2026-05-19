# Digit Sum

## Description

Compute the sum of digits of a non-negative integer.

**Input** (stdin):
- Single non-negative integer: `n` (0 <= n <= 10^18)

**Output** (stdout):
- Print the sum of digits of n

## Example

Input:
```
12345
```

Output:
```
15
```

## Constraints

- 0 <= n <= 10^18
- n fits in a 64-bit signed integer

## Category

System (number processing)

## BMB Notes
- Extract digits with `n % 10` (last digit) and `n / 10`; handle n=0 specially
```
let n: i64 = read_int();
if n == 0 { println(0) } else {
    let mut x: i64 = n;
    let mut sum: i64 = 0;
    while x > 0 { sum = sum + x % 10; x = x / 10 };
    println(sum)
};
0
```
