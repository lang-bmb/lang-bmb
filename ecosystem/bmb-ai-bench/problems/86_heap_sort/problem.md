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
- `&&` and `||` work in BMB with short-circuit semantics (safe for boundary checks)
- Use bubble sort or selection sort (O(n²) is fine for n≤1000):
```
fn main() -> i64 = {
    let n: i64 = read_int();
    let v = vec_new();
    for _i in 0..n { let _p = vec_push(v, read_int()) };
    // bubble sort — no && needed
    for i in 0..n {
        for j in 0..(n - i - 1) {
            if vec_get(v, j) > vec_get(v, j + 1) {
                let tmp: i64 = vec_get(v, j);
                let _s1 = vec_set(v, j, vec_get(v, j + 1));
                let _s2 = vec_set(v, j + 1, tmp)
            }
        }
    };
    for i in 0..n {
        if i > 0 { let _s = print_str(" "); () } else { () };
        let _p = print(vec_get(v, i))
    };
    println_str("");
    let _f = vec_free(v);
    0
};
