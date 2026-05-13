# Alternating

## Description

Compute the sum of an alternating sequence of +1 and -1.

The sequence is: 1, -1, 1, -1, 1, ... (always starts with +1). For `n` terms, the sum is:
- n=0: 0
- n=1: 1
- n=2: 1 + (-1) = 0
- n=3: 1 + (-1) + 1 = 1
- n=4: 0
- In general: 1 if n is odd, 0 if n is even (or zero)

**Input** (stdin):
- First integer: `t`, number of test cases
- For each test case: one integer `n`

**Output** (stdout):
- For each test case, print the sum (1 or 0) on its own line

## Example

Input:
```
4 1 2 3 4
```

Output:
```
1
0
1
0
```

Explanation:
- n=1: sequence [1] → sum=1
- n=2: sequence [1,-1] → sum=0
- n=3: sequence [1,-1,1] → sum=1
- n=4: sequence [1,-1,1,-1] → sum=0

## Constraints

- 1 ≤ t ≤ 100
- 0 ≤ n ≤ 1,000,000,000

## Key Notes

- n=0 → output 0
- n is odd → output 1
- n is even (and n > 0) → output 0
- Equivalently: `n % 2` (0 or 1)

## Category

Math / patterns
