# Contract Chain

## Description

Implement a multi-function pipeline where each function's postcondition satisfies the next function's precondition, demonstrating contract propagation.

Pipeline: normalize → scale → bound

1. `normalize(x, min, max)` — maps x from [min,max] to [0, 100]
2. `scale(x, factor)` — multiplies by factor (factor > 0)
3. `bound(x, limit)` — caps at limit

**Input** (stdin):
- First three integers: `min max factor` (min < max, factor > 0)
- Second integer: `limit`
- Third integer: `n`, number of values
- Next `n` integers: the values (each in [min, max])

**Output** (stdout):
- For each value, print the result of bound(scale(normalize(x, min, max), factor), limit)

## Contract Requirement

```
normalize: pre min < max, post ret >= 0 and ret <= 100
scale: pre x >= 0, post ret >= 0
bound: pre limit >= 0, post ret >= 0 and ret <= limit
```

## Example

Input:
```
0 100 2 150 3 0 50 100
```

Output:
```
0
100
150
```

## Constraints

- min < max
- factor > 0
- limit >= 0
- All values in [min, max]

## Category

Contract (multi-function propagation)
