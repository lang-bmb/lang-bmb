# Knapsack 01

0/1 Knapsack: maximize total value without exceeding capacity.

## Input
- First integer: n (number of items)
- Second integer: W (capacity)
- Next 2n integers: **weight1 value1 weight2 value2 ...** (interleaved pairs)

## Output
Maximum achievable value.

## Example
Input: `3 50 10 60 20 100 30 120`
- n=3, W=50
- Item 0: weight=10, value=60
- Item 1: weight=20, value=100
- Item 2: weight=30, value=120
- Best: items 0+1, total weight=30, total value=220 → output 220

## IMPORTANT: Reading Order

Read items as interleaved (weight, value) pairs:

```
let n: i64 = read_int();
let W: i64 = read_int();
let weights: i64 = vec_new();
let values: i64 = vec_new();
let mut i: i64 = 0;
while i < n {
    vec_push(weights, read_int());  // weight first
    vec_push(values, read_int());   // value second
    set i = i + 1;
};
```

## DP Implementation

Standard 1D DP approach (O(n*W) time, O(W) space):

```
// dp[w] = max value achievable with capacity exactly w
let dp: i64 = vec_new();
let mut w: i64 = 0;
while w <= W { vec_push(dp, 0); set w = w + 1; };

let mut item: i64 = 0;
while item < n {
    let wt: i64 = vec_get(weights, item);
    let vl: i64 = vec_get(values, item);
    // traverse w from W down to wt (avoid using item twice)
    set w = W;
    while w >= wt {
        let old: i64 = vec_get(dp, w);
        let new_val: i64 = vec_get(dp, w - wt) + vl;
        if new_val > old { vec_set(dp, w, new_val) } else { () };
        set w = w - 1;
    };
    set item = item + 1;
};
println(vec_get(dp, W))
```
