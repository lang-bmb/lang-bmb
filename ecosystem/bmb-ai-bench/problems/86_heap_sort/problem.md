# Heap Sort

Given n integers, sort them in ascending order and output space-separated on one line.

## Input
- First integer: n
- Next n integers: the unsorted values

## Output
The n integers sorted ascending, space-separated on one line.

## Example
Input: 5 5 3 1 4 2
Output: 1 2 3 4 5

## BMB Notes
- Use `vec_new()`/`vec_push()`/`vec_get()`/`vec_set()` for the array
- Swap: use a `tmp` variable (no swap builtin)
- Implement sift-down iteratively (while loop) — BMB supports recursion but iterative is cleaner
- Output: space-separated on one line using `print(x)` with space check, then `println_str("")`
- Simpler alternative: just use insertion sort (O(n²) is fine for n≤1000):
```
fn main() -> i64 = {
    let n: i64 = read_int();
    let v = vec_new();
    for _i in 0..n { let _p = vec_push(v, read_int()) };
    // insertion sort
    for i in 1..n {
        let key = vec_get(v, i);
        let mut j: i64 = i - 1;
        while j >= 0 && vec_get(v, j) > key {
            let _s = vec_set(v, j + 1, vec_get(v, j));
            j = j - 1
        };
        let _s = vec_set(v, j + 1, key)
    };
    for i in 0..n {
        if i > 0 { let _s = print_str(" "); () } else { () };
        let _p = print(vec_get(v, i))
    };
    println_str("");
    vec_free(v);
    0
};
```
