# Dispatch Table

Apply one of four operations to two operands.

## Input
- First integer: **t** (number of test cases)
- Each test case: **op a b** (three integers)
  - op=1: a+b; op=2: a-b; op=3: a*b; op=4: max(a,b)

## Output
Result, one per line (t lines total)

## Example
- `t=1, op=1, a=3, b=4` → 7
- `t=1, op=2, a=10, b=3` → 7
- `t=1, op=3, a=6, b=7` → 42
- `t=1, op=4, a=5, b=8` → 8

## IMPORTANT: t is the query count

```
let t: i64 = read_int();
let mut i: i64 = 0;
while i < t {
    let op: i64 = read_int();
    let a: i64 = read_int();
    let b: i64 = read_int();
    let result: i64 = if op == 1 { a + b }
                      else if op == 2 { a - b }
                      else if op == 3 { a * b }
                      else { if a > b { a } else { b } };
    println(result);
    set i = i + 1;
};
0
```
