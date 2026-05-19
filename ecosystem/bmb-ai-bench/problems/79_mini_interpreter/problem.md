# Mini Interpreter

Stack-based bytecode interpreter.

## Input
- First integer: `n` (total instruction count)
- Instructions follow as token stream:
  - `op=1` (push): reads one extra integer `val`, pushes `val` onto stack
  - `op=2` (add): pops two values `b` (top) then `a`, pushes `a+b`
  - `op=3` (subtract): pops two values `b` (top) then `a`, pushes `a-b`
  - `op=4` (multiply): pops two values `b` (top) then `a`, pushes `a*b`
  - `op=5` (dup): duplicates the top of stack (pushes a copy WITHOUT removing the original)
  - `op=6` (print): prints the **current top** of stack WITHOUT popping it

## Output

Each `op=6` prints one line.

## IMPORTANT: op=5 and op=6 Details

**op=5 (dup)**: Read the top element WITHOUT popping, then push a copy.
Stack grows by 1. Both the original and the copy remain on the stack.

```
// op=5: dup
let top_idx = vec_len(stack) - 1;
let top_val = vec_get(stack, top_idx);
let _p = vec_push(stack, top_val);
// stack size increases by 1
```

**op=6 (print)**: Read the top WITHOUT popping. Stack size does NOT change.

```
// op=6: print top (no pop)
let top_idx = vec_len(stack) - 1;
let top_val = vec_get(stack, top_idx);
let _p = println(top_val);
// stack size unchanged
```

**op=2,3,4 (binary ops)**: Pop twice (top=b, then a), push result. Stack shrinks by 1.

```
// op=2: add
let b = vec_pop(stack);
let a = vec_pop(stack);
let _p = vec_push(stack, a + b);
```

## Example

Input:
```
4 1 3 1 4 2 6
```
Output:
```
7
```
(push 3, push 4, add→7, print top=7. Stack after: [7])

Input:
```
5 1 5 5 2 6 6
```
Output:
```
10
10
```
(push 5→[5], dup→[5,5], add→[10], print top=10 no-pop→[10], print top=10 no-pop→[10])

## Constraints

- Stack operations are always valid (never pop from empty stack in tests)
- 1 <= n <= 1000

## Category

Integration (stack machine)
