# Mini Interpreter

Stack-based bytecode interpreter.

## Input
- First integer: `n` (total instruction count)
- Instructions follow as token stream:
  - `op=1` (push): reads one extra integer `val`, pushes `val` onto stack
  - `op=2` (add): pops two values `b` (top) then `a`, pushes `a+b`
  - `op=3` (subtract): pops two values `b` (top) then `a`, pushes `a-b`
  - `op=4` (multiply): pops two values `b` (top) then `a`, pushes `a*b`
  - `op=5` (dup): duplicates the top of stack (pushes a copy without removing)
  - `op=6` (print): prints the **current top** of stack (does NOT pop it)

## Output

Each `op=6` prints one line.

## Example

Input:
```
4 1 3 1 4 2 6
```
Output:
```
7
```
(push 3, push 4, add→7, print top=7)

Input:
```
5 1 5 5 2 6 6
```
Output:
```
10
10
```
(push 5, dup→[5,5], add→[10], print 10, print 10 again)

## Constraints

- Stack operations are always valid (never pop from empty stack in tests)
- 1 <= n <= 1000

## Category

Integration (stack machine)
