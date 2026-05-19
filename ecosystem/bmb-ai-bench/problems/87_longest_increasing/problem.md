# Longest Increasing

Find the length of the Longest Strictly Increasing Subsequence (LIS).

## Input
- First integer: n
- Next n integers: the sequence

## Output
Length of the longest strictly increasing subsequence.

## Examples
- `6 10 9 2 5 3 7` → LIS=[2,5,7] or [2,3,7], length=3
- `5 1 2 3 4 5` → 5

## BMB Notes
- O(n²) DP: `dp[i]` = LIS length ending at index i; initialize all to 1
- Use `set` for mutable variable assignment; `vec_get`/`vec_set` for vec access

## O(n²) DP Implementation

```
let n: i64 = read_int();
let arr = vec_new();
let mut i: i64 = 0;
while i < n {
    vec_push(arr, read_int());
    set i = i + 1;
};

// dp[i] = length of LIS ending at index i
let dp = vec_new();
let mut j: i64 = 0;
while j < n { vec_push(dp, 1); set j = j + 1; };

let mut result: i64 = 1;
let mut outer: i64 = 1;
while outer < n {
    let mut inner: i64 = 0;
    while inner < outer {
        if vec_get(arr, inner) < vec_get(arr, outer) {
            let candidate: i64 = vec_get(dp, inner) + 1;
            if candidate > vec_get(dp, outer) {
                vec_set(dp, outer, candidate)
            } else { () }
        } else { () };
        set inner = inner + 1;
    };
    if vec_get(dp, outer) > result { set result = vec_get(dp, outer) } else { () };
    set outer = outer + 1;
};
println(result);
0
```
