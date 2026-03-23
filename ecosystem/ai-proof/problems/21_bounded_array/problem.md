# Bounded Array Access

## Description

Implement a safe bounded array access function that uses contracts to guarantee
bounds safety at compile time.

The function must use a precondition contract (`pre`) to ensure the index is
within bounds, eliminating the need for runtime bounds checks.

**Input** (stdin):
- First integer: `n`, the number of elements (1 <= n <= 10)
- Next `n` integers: the array elements
- Last integer: `idx`, the 0-based index to access

**Output** (stdout):
- Print the element at the given index

## Contract Requirement

The core access function MUST include a contract of the form:
```
pre idx >= 0 and idx < n
```

This ensures:
1. No runtime bounds check is needed
2. The compiler can prove the access is safe
3. Out-of-bounds access is a compile-time error, not a runtime crash

## Example

Input:
```
5 10 20 30 40 50 2
```

Output:
```
30
```

## Constraints

- 1 <= n <= 10
- 0 <= idx < n (guaranteed by caller / test data)
- All values fit in a 64-bit signed integer

## Category

Contract (bounds verification)
