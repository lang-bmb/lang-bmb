# Pipeline

## Description

Apply a sequence of transformation operations to an integer array in order.

**Input** (stdin):
- Integer `n`: array length
- Next `n` integers: initial array values
- Integer `m`: number of operations
- For each operation, one integer `op` (1–5), optionally followed by argument `k`:
  - `op=1 k`: add `k` to every element
  - `op=2 k`: multiply every element by `k`
  - `op=3`: negate every element (multiply by -1)
  - `op=4`: replace each element with its absolute value
  - `op=5`: reverse the array

**Output** (stdout):
- Print the final array elements separated by single spaces, followed by a newline

## Example

Input:
```
3 1 2 3 1 1 10
```

Output:
```
11 12 13
```

Explanation:
- Array starts as [1, 2, 3]
- m=1, op=1, k=10: add 10 to all → [11, 12, 13]

Another example:

Input:
```
3 1 2 3 2 1 1 2 3
```

Output:
```
6 9 12
```

Explanation:
- Array starts as [1, 2, 3]
- m=2
- op=1, k=1: add 1 → [2, 3, 4]
- op=2, k=3: multiply by 3 → [6, 9, 12]

## Constraints

- 1 ≤ n ≤ 1000
- 0 ≤ m ≤ 1000
- -10,000,000 ≤ array elements ≤ 10,000,000
- -1,000,000 ≤ k ≤ 1,000,000
- `op` is one of 1, 2, 3, 4, 5
- Only `op=1` and `op=2` have an argument `k`; ops 3, 4, 5 have no argument

## Parse Order (explicit)

```
n = read_int()
for i in 0..n: array[i] = read_int()    // read n array values
m = read_int()                            // number of operations to apply
for _ in 0..m:                            // loop m times
    op = read_int()                       // op code 1–5
    if op == 1 or op == 2: k = read_int()  // only op 1,2 have an arg
    apply(op, k, array)
print(array)
```

## Key Notes

- **`m` is the count of operations** (read it in its own `read_int()` call before the op loop)
- Operations 3, 4, 5 do NOT read an extra integer — they apply immediately
- Print space-separated values with NO trailing space, followed by `\n`
- No test cases wrapper (single problem per input)

**CRITICAL parse order** (do not confuse `m` and `k`):
1. Read `n`, then read exactly `n` array values
2. Read `m` (number of operations)
3. For each of `m` operations: read `op`, then if `op==1` or `op==2` read `k`; ops 3/4/5 read nothing extra

## Category

Arrays / simulation / pipeline

## BMB Notes
- Use `abs(x)` for absolute value (built-in)
- Negate: `0 - val` (not `-val` directly)
- Reverse: two-pointer swap with `tmp` variable
- No trailing space: use `print(x)` + `print_str(" ")` pattern with index check
```
// op 4 abs:
for j in 0..n {
    let val = vec_get(v, j);
    if val < 0 { let _s = vec_set(v, j, 0 - val) } else { () }
};
// op 5 reverse:
let mut lo: i64 = 0;
let mut hi: i64 = n - 1;
while lo < hi {
    let tmp = vec_get(v, lo);
    let _s1 = vec_set(v, lo, vec_get(v, hi));
    let _s2 = vec_set(v, hi, tmp);
    lo = lo + 1;
    hi = hi - 1
};
```
