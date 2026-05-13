# Calculator

Stack-based calculator. Each instruction has an opcode; op=0 (push) also reads a value.

## Input
- First integer: n (number of instructions)
- Instructions follow as a token stream:
  - op=0 (push): next integer is the value to push
  - op=1 (add): pop two, push sum
  - op=2 (subtract): pop a then b, push b-a
  - op=3 (multiply): pop two, push product
  - op=4 (divide): pop a then b, push b/a (integer)

## Output
Print the top of the stack after all n instructions.

## Example
`3 0 3 0 4 1` -> push(3), push(4), add -> 7
