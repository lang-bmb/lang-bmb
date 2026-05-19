# Majority Element

Find the element appearing more than n/2 times. Output -1 if none.

## Input
- First integer: **t** (number of test cases)
- Each test case: **n** followed by **n** integers

## Output
Majority element or -1 (one per line, t lines total)

## Example
Input: `3  5 1 1 1 2 3  3 1 2 3  1 42`
- t=3 test cases
- Test 1: n=5, values=[1,1,1,2,3] → majority=1 (appears 3 > 2.5 times) → 1
- Test 2: n=3, values=[1,2,3] → no majority → -1
- Test 3: n=1, values=[42] → majority=42 → 42

## IMPORTANT: t is the query count

**CRITICAL**: Do NOT discard t. You MUST loop t times using `while i < t`.
**CRITICAL**: `let arr = vec_new()` — NO type annotation. `let arr: i64 = vec_new()` is WRONG.
**CRITICAL**: `let _p = vec_push(arr, read_int())` — vec_push result must be bound.

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
            set j = j + 1
        };
        let mut majority: i64 = -1;
        let mut k: i64 = 0;
        while k < n {
            let candidate: i64 = vec_get(arr, k);
            let mut cnt: i64 = 0;
            let mut m: i64 = 0;
            while m < n {
                if vec_get(arr, m) == candidate { set cnt = cnt + 1 } else { () };
                set m = m + 1
            };
            if cnt * 2 > n { set majority = candidate } else { () };
            set k = k + 1
        };
        println(majority);
        set i = i + 1
    };
    0
};
```
