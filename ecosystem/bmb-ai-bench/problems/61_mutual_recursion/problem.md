# Mutual Recursion

Determine if n is even (1) or odd (0) using mutual recursion.

## Input
- First integer: t
- Each test case: one non-negative integer n

## Output
1 if n is even, 0 if n is odd (one per line)

## Algorithm
isEven(0)=1; isEven(n)=isOdd(n-1)
isOdd(0)=0; isOdd(n)=isEven(n-1)
