# 124. Count Primes

LeetCode #204

## Problem

Given an integer `n`, return the number of prime numbers strictly less than `n`.

## Input

- One line: the integer `n`

## Output

The count of primes less than `n`

## Examples

```
Input: 10
Output: 4
(primes: 2, 3, 5, 7)
```

```
Input: 0
Output: 0
```

## BMB Notes

Trial division approach — check if each number 2..n-1 is prime:

```bmb
fn is_prime(n: i64, d: i64) -> bool
= if d * d > n { true }
    else if n % d == 0 { false }
    else { is_prime(n, d + 1) };

fn count_primes(n: i64, i: i64, acc: i64) -> i64
= if i >= n { acc }
    else if is_prime(i, 2) { count_primes(n, i + 1, acc + 1) }
    else { count_primes(n, i + 1, acc) };
```
