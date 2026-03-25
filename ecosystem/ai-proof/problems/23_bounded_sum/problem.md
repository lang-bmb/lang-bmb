# Bounded Sum

## Description

Sum elements of an array with bounds-checked access using contracts.

The sum function must use a contract to ensure safe array access at every iteration.

**Input** (stdin):
- First integer: `n`, the number of elements (0 <= n <= 100000)
- Next `n` integers: the array elements

**Output** (stdout):
- Print the sum of all elements

## Contract Requirement

The access function must have:
```
pre idx >= 0 and idx < len
```

## Example

Input:
```
4 10 20 30 40
```

Output:
```
100
```

## Constraints

- 0 <= n <= 100000
- Sum fits in a 64-bit signed integer

## Category

Contract (bounds verification)
