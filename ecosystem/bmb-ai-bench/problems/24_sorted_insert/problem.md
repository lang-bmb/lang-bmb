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
// CORRECT — use for-in loop with set for index mutation:
let total = n + 1;
for k in 0..total {
    if k > 0 { let _s = print_str(" "); () } else { () };
    let _p = print(vec_get(result, k));
    ()
};
println_str("");

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

## BMB Notes
- Use `set` for ALL variable mutations: `set pos = idx`, `set j = j - 1`
- Find insert position first, then shift elements right, then set the value
```
fn find_insert_pos(v: i64, n: i64, val: i64) -> i64
    post ret >= 0 and ret <= n
= {
    let mut pos = n;
    for idx in 0..n {
        if vec_get(v, idx) > val {
            set pos = idx;
            return pos
        } else { () }
    };
    pos
};

fn main() -> i64 = {
    let n = read_int();
    let v = vec_new();
    for _k in 0..n { let _p = vec_push(v, read_int()); () };
    let val = read_int();
    let pos = find_insert_pos(v, n, val);
    let _p = vec_push(v, 0);
    let mut j = n;
    while j > pos {
        let _s = vec_set(v, j, vec_get(v, j - 1));
        set j = j - 1
    };
    let _s2 = vec_set(v, pos, val);
    let total = n + 1;
    for k in 0..total {
        if k > 0 { let _s3 = print_str(" "); () } else { () };
        let _p2 = print(vec_get(v, k));
        ()
    };
    println_str("");
    let _f = vec_free(v);
    0
};
```
