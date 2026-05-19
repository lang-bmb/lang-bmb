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

**CRITICAL**: Use `set` for ALL variable updates. `tail = ...`, `count = ...`, `head = ...` do NOT work — must be `set tail = ...`, `set count = ...`, `set head = ...`.
**CRITICAL**: op=2 (dequeue) on an EMPTY queue prints -1. Check count == 0 first.

```
fn main() -> i64 = {
    let capacity: i64 = read_int();
    let n: i64 = read_int();
    let buf = vec_new();
    let mut i: i64 = 0;
    while i < capacity {
        let _p = vec_push(buf, 0);
        set i = i + 1
    };
    let mut head: i64 = 0;
    let mut tail: i64 = 0;
    let mut count: i64 = 0;
    let mut op_idx: i64 = 0;
    while op_idx < n {
        let op: i64 = read_int();
        if op == 1 {
            let val: i64 = read_int();
            if count == capacity {
                println(-1)
            } else {
                let _w = vec_set(buf, tail, val);
                set tail = (tail + 1) % capacity;
                set count = count + 1
            }
        } else {
            if op == 2 {
                if count == 0 {
                    println(-1)
                } else {
                    let front: i64 = vec_get(buf, head);
                    set head = (head + 1) % capacity;
                    set count = count - 1;
                    println(front)
                }
            } else {
                println(count)
            }
        };
        set op_idx = op_idx + 1
    };
    0
};
```

**Critical**: `set head = (head + 1) % capacity` — without modular arithmetic, head will drift past the allocated array bounds.

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
