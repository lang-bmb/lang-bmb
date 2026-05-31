# Best Time to Buy and Sell Stock

## Description

Given an array of stock prices, find the maximum profit from a single buy and sell. You must buy before you sell. If no profit is possible, return 0.

**Input** (stdin):
- First integer: `n`, number of days (1 <= n <= 100000)
- Next `n` integers: price on each day

**Output** (stdout):
- Print the maximum profit (0 if no profit possible)

## Example

Input:
```
7 7 1 5 3 6 4 9
```

Output:
```
8
```

(Buy at 1, sell at 9)

## Constraints

- 1 <= n <= 100000
- All prices fit in i64 and are >= 0

## Category

Algorithm (greedy)

## BMB Notes
- Track minimum price seen so far and maximum profit
- One pass: for each price, update max_profit and min_price
```
fn main() -> i64 = {
    let n: i64 = read_int();
    let v = vec_new();
    for _i in 0..n { let x = read_int(); vec_push(v, x) };
    let mut min_price = vec_get(v, 0);
    let mut max_profit = 0;
    for i in 1..n {
        let p = vec_get(v, i);
        if p - min_price > max_profit { max_profit = p - min_price } else { () };
        if p < min_price { min_price = p } else { () }
    };
    println(max_profit);
    vec_free(v);
    0
};
```
