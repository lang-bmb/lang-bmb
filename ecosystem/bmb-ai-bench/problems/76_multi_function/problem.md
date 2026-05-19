# Multi Function

Given n integers, output 5 statistics.

## Input
- First integer: n
- Next n integers: the array

## Output (5 lines)
1. Sum
2. Minimum
3. Maximum
4. abs(minimum): absolute value of the minimum element
5. sign(sum): 1 if sum>0, -1 if sum<0, 0 if sum=0

## Example
`5 1 -2 3 -4 5` -> sum=3, min=-4, max=5, abs(min)=4, sign(3)=1

## BMB Notes

**CRITICAL**: Use `set` for ALL variable updates. `sum = sum + x` does NOT work — must be `set sum = sum + x`.

```
fn main() -> i64 = {
    let n: i64 = read_int();
    let v = vec_new();
    let mut i: i64 = 0;
    while i < n {
        let _p = vec_push(v, read_int());
        set i = i + 1
    };
    let mut sum: i64 = 0;
    let mut mn: i64 = vec_get(v, 0);
    let mut mx: i64 = vec_get(v, 0);
    let mut j: i64 = 0;
    while j < n {
        let x: i64 = vec_get(v, j);
        set sum = sum + x;
        if x < mn { set mn = x } else { () };
        if x > mx { set mx = x } else { () };
        set j = j + 1
    };
    println(sum);
    println(mn);
    println(mx);
    println(abs(mn));
    println(if sum > 0 { 1 } else if sum < 0 { -1 } else { 0 });
    0
};
```
