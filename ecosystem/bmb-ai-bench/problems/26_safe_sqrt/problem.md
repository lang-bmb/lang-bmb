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
