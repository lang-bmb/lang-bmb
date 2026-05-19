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

## BMB Notes
- Use `vec_push` to enqueue, `vec_remove(q, 0)` to dequeue from front
- `vec_remove(v, idx)` removes element at idx and returns it (shifts elements left)
- Check `vec_len(q) == 0` before dequeue
```
let q_count: i64 = read_int();
let queue = vec_new();
for _i in 0..q_count {
    let op: i64 = read_int();
    if op == 1 { let val: i64 = read_int(); vec_push(queue, val) }
    else {
        if vec_len(queue) == 0 { println(-1) }
        else { println(vec_remove(queue, 0)) }
    }
};
0
```
