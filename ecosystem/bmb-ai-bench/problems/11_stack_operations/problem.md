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

## BMB Notes
- Use `vec_push` for push, `vec_pop` for pop, `vec_len` to check empty
- `vec_pop` removes and returns the last element (top of stack)
```
let q: i64 = read_int();
let stack = vec_new();
for _i in 0..q {
    let op: i64 = read_int();
    if op == 1 { let val: i64 = read_int(); vec_push(stack, val) }
    else if op == 2 {
        if vec_len(stack) == 0 { println(-1) }
        else { println(vec_pop(stack)) }
    } else {
        if vec_len(stack) == 0 { println(-1) }
        else { println(vec_get(stack, vec_len(stack) - 1)) }
    }
};
0
```
