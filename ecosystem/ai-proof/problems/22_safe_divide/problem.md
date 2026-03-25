# Safe Division

## Description

Implement integer division with a contract ensuring the divisor is non-zero.

The function MUST use a precondition contract to guarantee safe division at compile time.

**Input** (stdin):
- First integer: `n`, number of operations (1 <= n <= 1000)
- For each operation: two integers `a b` (b is guaranteed non-zero by test data)

**Output** (stdout):
- For each operation, print `a / b` (integer division, truncated toward zero)

## Contract Requirement

```
pre b != 0
```

## Example

Input:
```
3 10 3 -7 2 100 10
```

Output:
```
3
-3
10
```

## Constraints

- 1 <= n <= 1000
- b != 0 (guaranteed by test data)
- All values fit in a 64-bit signed integer

## Category

Contract (division safety)
