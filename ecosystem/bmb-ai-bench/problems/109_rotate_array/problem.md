# Rotate Array

## Description

Given an array of n integers, rotate the array to the right by k steps.

**Input** (stdin):
- First integer: `n`, array size (1 <= n <= 10000)
- Second integer: `k`, rotation steps (0 <= k)
- Next `n` integers: array elements

**Output** (stdout):
- Space-separated elements after rotation

## Example

Input:
```
7 3 1 2 3 4 5 6 7
```

Output:
```
5 6 7 1 2 3 4
```

## Constraints

- 1 <= n <= 10000
- 0 <= k (effective rotation is k mod n)

## Category

Algorithm (array)

## BMB Notes
- Effective rotation: `kk = k % n`
- Output: last `kk` elements, then first `n-kk` elements
```
fn main() -> i64 = {
    let n: i64 = read_int(); let k: i64 = read_int();
    let v = vec_new();
    for _i in 0..n { let x = read_int(); vec_push(v, x) };
    let kk = k % n;
    let mut first = 1;
    for i in n-kk..n {
        if first == 0 { print_str(" ") } else { set first = 0 };
        print(vec_get(v, i))
    };
    for i in 0..n-kk {
        print_str(" "); print(vec_get(v, i))
    };
    println_str("");
    vec_free(v);
    0
};
```
