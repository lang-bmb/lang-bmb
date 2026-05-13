# Bounded Queue Contract

Bounded FIFO queue. Push to full queue prints -1 (overflow, item not added).

## Input
- First integer: capacity
- Second integer: n (operations)
- Operations:
  - op=1 (enqueue val): if full, print -1 (do NOT add); otherwise, add val
  - op=2 (dequeue): remove and print front
  - op=3 (size): print current count

## Output
op=1 overflow (-1), op=2 dequeue, and op=3 size each print one line.

## Example
`2 4 1 5 1 10 1 15 3` -> enqueue(5) OK, enqueue(10) full->-1, size->2
Wait: cap=2, enqueue(5) OK (1 item), enqueue(10) OK (2 items, full), enqueue(15) FULL->-1, size->2
