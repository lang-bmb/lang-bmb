# Palindrome Check

Check if an integer array is a palindrome.

## Input
- First integer: **t** (number of test cases)
- Each test case: **n** followed by **n** integers

## Output
1 if palindrome, 0 if not (one per line, t lines total)

## Example
Input: `3  5 1 2 3 2 1  3 1 2 3  1 42`
- t=3 test cases
- Test 1: n=5, values=[1,2,3,2,1] → palindrome → 1
- Test 2: n=3, values=[1,2,3] → not palindrome → 0
- Test 3: n=1, values=[42] → palindrome → 1

## IMPORTANT: t is the query count

For each test case, first read n, then read n values.

**CRITICAL**: The main function MUST end with `0` (the i64 return value). Do NOT end the function body with `println(ok)` — that returns `()`, not `i64`, causing a type error.

```
fn main() -> i64 = {
    let t: i64 = read_int();
    let mut i: i64 = 0;
    while i < t {
        let n: i64 = read_int();
        let arr = vec_new();
        let mut j: i64 = 0;
        while j < n {
            let _p = vec_push(arr, read_int());
            set j = j + 1;
        };
        let mut ok: i64 = 1;
        let mut k: i64 = 0;
        while k < n / 2 {
            if vec_get(arr, k) != vec_get(arr, n - 1 - k) { set ok = 0 } else { () };
            set k = k + 1;
        };
        println(ok);
        vec_free(arr);
        set i = i + 1;
    };
    0
};
```
