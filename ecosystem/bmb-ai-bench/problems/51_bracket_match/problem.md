# Bracket Match

Check if a bracket sequence is balanced. Brackets are given as ASCII codes.

## Input
- First integer: t
- Each test case: n followed by n ASCII codes
  - 40='(' 41=')' 91='[' 93=']' 123='{' 125='}'

## Output
For each test case: 1 if balanced, 0 if not (one per line)

## Notes
- Empty (n=0) is balanced
- Pairs: '(' ')'; '[' ']'; '{' '}'

## Example
`1 4 40 91 93 41` -> "([])" -> 1
