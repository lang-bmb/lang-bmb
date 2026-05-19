# Ring Buffer

Circular buffer with fixed capacity. When the buffer is **full**, writing a new value **overwrites** the oldest element (head advances).

## Input
- First integer: capacity
- Second integer: n (number of operations)
- Operations:
  - `op=1` (write val): always writes val to the buffer. If full, head advances (oldest overwritten).
  - `op=2` (read): remove and print the oldest element (front). Print -1 if empty.
  - `op=3` (size): print current element count.

## Output

`op=2` and `op=3` each print one line.

## IMPORTANT: Overwrite-When-Full Logic

When writing to a FULL buffer:
1. Store the new value at `buf[tail]`
2. Advance `tail = (tail + 1) % capacity`
3. **Also advance `head = (head + 1) % capacity`** (oldest is lost)
4. `count` stays at `capacity` (buffer remains full)

When writing to a NOT-FULL buffer:
1. Store the new value at `buf[tail]`
2. Advance `tail = (tail + 1) % capacity`
3. `count = count + 1`

The `head` pointer ONLY advances when writing to a full buffer (to discard the oldest element).

## Implementation Sketch

```
let mut head = 0;
let mut tail = 0;
let mut count = 0;
// buf = vec of capacity elements (pre-allocated with zeros)

// op=1 write val:
if count == capacity {
    // full: overwrite oldest, advance head
    let _w = vec_set(buf, tail, val);
    tail = (tail + 1) % capacity;
    head = (head + 1) % capacity   // head advances!
} else {
    // not full: insert at tail
    let _w = vec_set(buf, tail, val);
    tail = (tail + 1) % capacity;
    count = count + 1
};

// op=2 read (dequeue oldest):
let oldest = vec_get(buf, head);
head = (head + 1) % capacity;
count = count - 1;
let _p = println(oldest);
```

## Example

Input:
```
3 5 1 10 1 20 1 30 3 2
```
Output:
```
3
10
```
(capacity=3, write 10→[10] count=1, write 20→[10,20] count=2, write 30→[10,20,30] count=3, size=3, dequeue head=10)

Input:
```
2 4 1 10 1 20 1 30 2
```
Output:
```
20
```
(capacity=2, write 10→buf[0]=10, count=1, write 20→buf[1]=20, count=2 FULL,
write 30→FULL: buf[tail=0]=30, tail→1, head→1 (advance!). Buffer=[30,20], head=1, count=2.
dequeue oldest=buf[head=1]=20)

## Constraints

- 1 <= capacity <= 1000
- All read ops when non-empty, prints are valid

## BMB Notes

**CRITICAL**: In BMB, an `if ... else if ... else if ...` chain used as a statement (not last expression) MUST end with `;`:
```
if op == 1 {
    // write
} else if op == 2 {
    // read
} else if op == 3 {
    // size
};     // <-- semicolon REQUIRED — without it, next statement causes parse error
```

**CRITICAL**: Use `set` for mutable variable updates: `set tail = (tail + 1) % capacity`.

Complete example:
```
fn main() -> i64 = {
    let capacity: i64 = read_int();
    let n: i64 = read_int();
    let buf = vec_new();
    for _i in 0..capacity { let _p = vec_push(buf, 0) };
    let mut head: i64 = 0; let mut tail: i64 = 0; let mut count: i64 = 0;
    let mut op_idx: i64 = 0;
    while op_idx < n {
        let op: i64 = read_int();
        if op == 1 {
            let val: i64 = read_int();
            let _w = vec_set(buf, tail, val);
            set tail = (tail + 1) % capacity;
            if count == capacity {
                set head = (head + 1) % capacity
            } else {
                set count = count + 1
            }
        } else if op == 2 {
            if count > 0 {
                let oldest: i64 = vec_get(buf, head);
                set head = (head + 1) % capacity;
                set count = count - 1;
                println(oldest)
            } else { println(-1) }
        } else if op == 3 {
            println(count)
        };
        set op_idx = op_idx + 1
    };
    0
};
```

## Category

System (circular buffer)
