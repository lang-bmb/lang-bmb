# Sorted Insert

## Description

Insert a value into a sorted array while maintaining sorted order. Use contracts to verify the result remains sorted.

**Input** (stdin):
- First integer: `n`, the number of elements (0 <= n <= 10000)
- Next `n` integers: a sorted array (non-decreasing)
- Last integer: `val`, the value to insert

**Output** (stdout):
- Print ALL n+1 elements of the resulting sorted array, space-separated on a **single line**

## IMPORTANT: Output Format

Output exactly ONE line with all n+1 elements separated by spaces:
```
// CORRECT:
// Build result into vec, then print all at end
let i = 0;
while i < vec_len(result) {
    if i > 0 { let _s = print_str(" ") };
    let _p = print(vec_get(result, i));
    i = i + 1
};
println_str("");  // newline at end

// WRONG — print elements during insertion:
// Do NOT print elements while you are searching/shifting.
// First complete the insertion, THEN print.
```

## Algorithm

1. Find the insertion position (first index where array[i] >= val)
2. Insert val at that position (shift elements right)
3. Print all n+1 elements space-separated on one line

## Contract Requirement

The insertion function should verify:
```
fn insert_sorted(v: i64, arr: i64, n: i64) -> i64
    pre n >= 0
    post ret == n + 1
= ...;
```

## Example

Input:
```
5 1 3 5 7 9 4
```

Output:
```
1 3 4 5 7 9
```

(Insert 4 into [1,3,5,7,9] → [1,3,4,5,7,9], printed space-separated on one line)

## Constraints

- 0 <= n <= 10000
- Input array is guaranteed sorted
- All values fit in a 64-bit signed integer

## Category

Contract (sorted invariant)
