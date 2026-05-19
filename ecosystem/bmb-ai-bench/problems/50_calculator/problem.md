# Calculator

Stack-based calculator. Each instruction has an opcode; op=0 (push) also reads a value.

## Input
- First integer: n (number of instructions)
- Instructions follow as a token stream:
  - op=0 (push): next integer is the value to push
  - op=1 (add): pop a (top), pop b (next), push a+b
  - op=2 (subtract): pop a (top), pop b (next), push b-a
  - op=3 (multiply): pop a (top), pop b (next), push a*b
  - op=4 (divide): pop a (top), pop b (next), push b/a (integer division)

## Output
Print the top of the stack after all n instructions.

## IMPORTANT: Pop Order for Binary Operations

For ALL binary operations (add/subtract/multiply/divide):
1. Pop `a` = TOP element (most recently pushed)
2. Pop `b` = NEXT element (second-most recently pushed)
3. Push the result

```
// op=4 divide (same pattern for op=1,2,3):
let a = vec_pop(stack);   // a = top (divisor)
let b = vec_pop(stack);   // b = second (dividend)
let _p = vec_push(stack, b / a);  // push b/a

// Example: push(20), push(4), divide → a=4, b=20, result=20/4=5
```

For subtract: `b - a` = (second from top) - (top)
For divide: `b / a` = (second from top) / (top)

## Example
`3 0 3 0 4 1` → push(3), push(4), add(a=4,b=3)→push(7) → output: 7

`3 0 20 0 4 4` → push(20), push(4), divide(a=4,b=20)→push(20/4=5) → output: 5

## BMB Notes

**CRITICAL**: `if op==0 {...} else if op==1 {...} else if op==4 {...}` used as a statement (not last expression) MUST end with `;`. Use nested `if-else` or ensure trailing `;`.

Recommended pattern (nested if-else avoids the semicolon issue):
```
fn main() -> i64 = {
    let n: i64 = read_int();
    let stack = vec_new();
    let mut i: i64 = 0;
    while i < n {
        let op: i64 = read_int();
        if op == 0 {
            let val: i64 = read_int();
            let _p = vec_push(stack, val)
        } else {
            let b: i64 = vec_pop(stack);
            let a: i64 = vec_pop(stack);
            if op == 1 { let _p = vec_push(stack, a + b) }
            else if op == 2 { let _p = vec_push(stack, b - a) }
            else if op == 3 { let _p = vec_push(stack, a * b) }
            else { let _p = vec_push(stack, b / a) }
        };
        set i = i + 1
    };
    println(vec_get(stack, vec_len(stack) - 1));
    0
};
```

## Constraints
- Stack is always valid (no underflow in tests)
- 1 <= n <= 1000
