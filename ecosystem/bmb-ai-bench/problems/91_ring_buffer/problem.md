# Ring Buffer

Circular buffer with capacity limit. Full buffer: discard new pushes.

## Input
- First integer: capacity
- Second integer: n (number of operations)
- Operations:
  - op=1 (push val): if not full, add val to back; if full, discard val (no output)
  - op=2 (dequeue): remove and print front (FIFO)
  - op=3 (size): print current element count

## Output
op=2 and op=3 each print one line.

## Example
`3 5 1 10 1 20 1 30 3 2` -> push 10,20,30; size->3; dequeue->10
