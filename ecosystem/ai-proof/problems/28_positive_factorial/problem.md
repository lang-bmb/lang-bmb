# Positive Factorial

## Description

Compute factorial with a contract ensuring the input is non-negative and within a safe range.

**Input** (stdin):
- First integer: `n`, number of queries (1 <= n <= 20)
- Next `n` integers: values to compute factorial for (0 <= x <= 20)

**Output** (stdout):
- For each value, print x!

## Contract Requirement

```
pre n >= 0 and n <= 20
post ret >= 1
```

## Example

Input:
```
3 0 5 10
```

Output:
```
1
120
3628800
```

## Constraints

- 0 <= x <= 20 (20! fits in i64)

## Category

Contract (range + postcondition)
