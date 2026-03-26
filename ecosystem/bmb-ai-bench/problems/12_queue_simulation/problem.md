# Queue Simulation

## Description

Simulate a FIFO queue using an array. Process enqueue and dequeue operations.

**Input** (stdin):
- First integer: `q`, the number of operations (1 <= q <= 10000)
- For each operation:
  - `1 x` — enqueue x
  - `2` — dequeue and print the front element

**Output** (stdout):
- For each dequeue, print the value on a new line
- If dequeue on empty queue, print `-1`

## Example

Input:
```
5 1 10 1 20 2 2 2
```

Output:
```
10
20
-1
```

## Constraints

- 1 <= q <= 10000
- All values fit in a 64-bit signed integer

## Category

System (data structure)
