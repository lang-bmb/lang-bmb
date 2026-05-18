# Power Mod

## Description

Compute modular exponentiation: a^b mod m for multiple queries using fast exponentiation.

**Input** (stdin):
- First integer: `n`, the number of queries (1 <= n <= 100)
- For each query: three integers `a b m` on the same line

**Output** (stdout):
- For each query, print a^b mod m on its own line

## Example

Input:
```
3 2 10 1000 3 3 7 5 1 1000
```

Output:
```
24
6
5
```

(2^10 = 1024, 1024 mod 1000 = 24; 3^3 = 27, 27 mod 7 = 6; 5^1 = 5, 5 mod 1000 = 5)

## Constraints

- 1 <= n <= 100
- 1 <= a, m <= 1000000007
- 0 <= b <= 10^18

## Category

Algorithm (modular arithmetic)
