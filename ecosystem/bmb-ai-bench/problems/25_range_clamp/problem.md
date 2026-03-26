# Range Clamp

## Description

Clamp each element of an array to a given range [lo, hi] using contracts.

**Input** (stdin):
- First two integers: `lo hi` (lo <= hi)
- Third integer: `n`, the number of elements (1 <= n <= 100000)
- Next `n` integers: the elements

**Output** (stdout):
- Print the clamped elements separated by spaces

## Contract Requirement

The clamp function must have:
```
pre lo <= hi
post ret >= lo and ret <= hi
```

## Example

Input:
```
0 10 5 -3 5 15 0 7
```

Output:
```
0 5 10 0 7
```

## Constraints

- lo <= hi
- 1 <= n <= 100000
- All values fit in a 64-bit signed integer

## Category

Contract (range verification)
