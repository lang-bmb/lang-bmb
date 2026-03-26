# Stack Operations

## Description

Simulate a stack using an array. Process a sequence of push and pop operations.

**Input** (stdin):
- First integer: `q`, the number of operations (1 <= q <= 10000)
- For each operation:
  - `1 x` — push x onto the stack
  - `2` — pop the top element and print it
  - `3` — print the top element without popping

**Output** (stdout):
- For each operation 2 or 3, print the result on a new line
- If pop/top on empty stack, print `-1`

## Example

Input:
```
5 1 10 1 20 3 2 2
```

Output:
```
20
20
10
```

## Constraints

- 1 <= q <= 10000
- All values fit in a 64-bit signed integer

## Category

System (data structure)
