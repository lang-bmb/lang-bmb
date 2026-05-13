# Sparse Array

Sparse array: maps integer indices to values; unset indices default to 0.

## Input
- First integer: n (number of operations)
- Operations:
  - op=1 (set): read idx val, store val at index idx
  - op=2 (get): read idx, print value at idx (0 if not set)

## Output
Each op=2 prints one line.

## Example
`3 1 5 10 2 5 2 0` -> set[5]=10, get[5]->10, get[0]->0
