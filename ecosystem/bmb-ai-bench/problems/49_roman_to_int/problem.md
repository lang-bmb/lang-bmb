# Roman To Int

## Description

Convert a sequence of Roman numeral digit values to an integer.

You are given the Roman numeral digit values as a list of integers (e.g., M=1000, D=500, C=100, L=50, X=10, V=5, I=1). Apply the standard Roman numeral subtraction rule: if the current digit value is **less than** the next digit value, add (next - current) to the result and skip two positions; otherwise add the current and advance by one.

**Input** (stdin):
- First integer: `t`, number of test cases
- For each test case:
  - Integer `n`: count of digit values in this numeral
  - Next `n` integers: the Roman digit values in order

**Output** (stdout):
- For each test case, print the integer result on its own line

## Example

Input:
```
2 3 1000 100 1000 2 1 5
```

Output:
```
1900
4
```

Explanation:
- Test 1: [1000, 100, 1000] → at i=0: 100 < 1000, add 1000-100=900, i+=2; at i=2: 1000, add 1000, i+=1 → 1900 (MCM)
- Test 2: [1, 5] → 1 < 5: add 5-1=4 → 4 (IV)

## Constraints

- 1 ≤ t ≤ 100
- 1 ≤ n ≤ 20
- Each digit value is one of: 1, 5, 10, 50, 100, 500, 1000
- Results fit in 64-bit signed integer

## Category

Practical / math / conversion
