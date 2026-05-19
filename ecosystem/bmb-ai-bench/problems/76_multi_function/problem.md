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
- `abs(x)` is a built-in function (works for i64)
- Sign: `if sum > 0 { 1 } else if sum < 0 { -1 } else { 0 }`
```
let mut sum: i64 = 0;
let mut mn: i64 = vec_get(v, 0);
let mut mx: i64 = vec_get(v, 0);
for i in 0..n {
    let x: i64 = vec_get(v, i);
    sum = sum + x;
    if x < mn { mn = x } else { () };
    if x > mx { mx = x } else { () }
};
println(sum);
println(mn);
println(mx);
println(abs(mn));
println(if sum > 0 { 1 } else if sum < 0 { -1 } else { 0 });
```
