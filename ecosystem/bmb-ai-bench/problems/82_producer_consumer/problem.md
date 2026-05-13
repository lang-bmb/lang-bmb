# Producer Consumer

FIFO queue simulation with enqueue (op=1) and dequeue (op=2) operations.

## Input
- First integer: n (number of operations)
- Operations:
  - op=1 (enqueue): read one integer, add to back
  - op=2 (dequeue): remove and print front element

## Output
Each dequeue prints one line.

## Example
`4 1 10 1 20 2 2` -> enqueue 10, enqueue 20, dequeue->10, dequeue->20
