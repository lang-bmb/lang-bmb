# Bounded Stack

## Description

Implement a fixed-capacity stack with contracts preventing overflow and underflow.

**Input** (stdin):
- First integer: `capacity` (1 <= capacity <= 100)
- Second integer: `q`, number of operations
- For each operation:
  - `1 x` — push x (only if not full)
  - `2` — pop and print top (only if not empty)
  - `3` — print current size

**Output** (stdout):
- For push when full: print `FULL`
- For pop when empty: print `EMPTY`
- For pop when not empty: print the value
- For size query: print the current size

## Contract Requirement

Push should have: `pre size < capacity`
Pop should have: `pre size > 0`

## Example

Input:
```
2 5 1 10 1 20 1 30 2 3
```

Output:
```
FULL
20
1
```

## Constraints

- 1 <= capacity <= 100
- 1 <= q <= 10000

## Category

Contract (capacity verification)
