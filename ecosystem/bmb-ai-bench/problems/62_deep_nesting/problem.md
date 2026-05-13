# Deep Nesting

Count how many times n can be divided by 10 before reaching a single-digit value (0-9). For negative n, return n unchanged.

## Input
- First integer: t
- Each test case: one integer n

## Output
Nesting depth, one per line.

## Examples
- n=5 -> 0 (already single-digit)
- n=50 -> 1 (50/10=5)
- n=500 -> 2
- n=0 -> 0
- n=-1 -> -1 (negative: pass through)

## Algorithm
depth=0; while n>=10: n=n/10, depth++. Return depth. If n<0: return n.
