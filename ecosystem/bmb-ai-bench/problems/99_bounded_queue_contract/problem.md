# Bounded Queue Contract

Bounded FIFO queue. Push to full queue prints -1 (overflow, item not added).

## Input
- First integer: capacity
- Second integer: n (operations)
- Operations:
  - op=1 (enqueue val): if count == capacity, print -1 (do NOT add); otherwise add val to back
  - op=2 (dequeue): remove and print the FRONT element (oldest)
  - op=3 (size): print current count

## Output
op=1 overflow (-1), op=2 dequeue value, and op=3 size each print one line.

## IMPORTANT: Use Circular Buffer with Modular Arithmetic

Use a circular buffer (ring buffer) with head and tail indices. Use modular arithmetic to wrap around.

```
let buf = vec_new();
// Pre-allocate capacity slots
let i = 0;
while i < capacity {
    let _p = vec_push(buf, 0);
    i = i + 1
};
let mut head = 0;   // front of queue (oldest element)
let mut tail = 0;   // next empty slot
let mut count = 0;

// op=1 enqueue(val):
if count == capacity {
    let _p = println(-1)  // overflow, do NOT add
} else {
    let _w = vec_set(buf, tail, val);
    tail = (tail + 1) % capacity;
    count = count + 1
};

// op=2 dequeue:
let front = vec_get(buf, head);
head = (head + 1) % capacity;  // MUST use modular arithmetic
count = count - 1;
let _p = println(front);

// op=3 size:
let _p = println(count);
```

**Critical**: `head = (head + 1) % capacity` — without modular arithmetic, head will drift past the allocated array bounds, causing garbage values.

## Example

Input: `2 4 1 5 1 10 1 15 3`
(cap=2, ops: enqueue 5, enqueue 10, enqueue 15, size)

Output:
```
-1
2
```
(enqueue 5 → count=1, enqueue 10 → count=2=full, enqueue 15 → FULL print -1, size=2)

Input: `3 6 1 100 2 1 200 2 1 300 2`
(cap=3, ops: enqueue 100, dequeue, enqueue 200, dequeue, enqueue 300, dequeue)

Output:
```
100
200
300
```

## Constraints
- 1 <= capacity <= 1000
- All dequeue ops when count > 0 (tests are valid)

## Category

System (bounded queue with contracts)
