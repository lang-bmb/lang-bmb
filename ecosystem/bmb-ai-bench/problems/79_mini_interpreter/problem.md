# Mini Interpreter

Stack-based bytecode interpreter. op=1 (push) reads an extra integer; other ops take no extra argument.

## Input
- First integer: n (total instruction count)
- Instructions follow as token stream:
  - op=1 (push val): push val onto stack
  - op=2 (add): pop two, push sum
  - op=3 (subtract): pop a then b, push b-a
  - op=4 (multiply): pop two, push product
  - op=5 (divide): pop a then b, push b/a
  - op=6 (print): pop top and print it

## Output
Each op=6 prints one line.

## Example
`4 1 3 1 4 2 6` -> push(3), push(4), add, print -> 7
