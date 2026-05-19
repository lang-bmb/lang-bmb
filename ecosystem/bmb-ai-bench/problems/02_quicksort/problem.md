# Quicksort

## Description

Implement quicksort to sort an array of integers in non-decreasing order.

**Input** (stdin):
- First integer: `n`, the number of elements (0 <= n <= 100000)
- Next `n` integers: the array elements to sort

**Output** (stdout):
- Print the sorted elements separated by spaces, followed by a newline
- If the array is empty (n=0), print an empty line

## Example

Input:
```
5 5 3 1 4 2
```

Output:
```
1 2 3 4 5
```

## Constraints

- All values fit in a 64-bit signed integer
- 0 <= n <= 100000

## Category

Algorithm (sorting)

## BMB Notes
- Use recursive quicksort; swap via tmp variable using `vec_get`/`vec_set`
- Last-element pivot; partition in place; output space-separated via first-flag pattern
```
fn qs(v: i64, lo: i64, hi: i64) -> i64 = {
    if lo >= hi { 0 } else {
        let pivot: i64 = vec_get(v, hi);
        let mut i: i64 = lo - 1; let mut j: i64 = lo;
        while j < hi {
            if vec_get(v, j) <= pivot {
                set i = i + 1;
                let tmp: i64 = vec_get(v, i);
                vec_set(v, i, vec_get(v, j));
                vec_set(v, j, tmp)
            } else { () };
            set j = j + 1
        };
        let tmp: i64 = vec_get(v, i + 1);
        vec_set(v, i + 1, vec_get(v, hi)); vec_set(v, hi, tmp);
        let p: i64 = i + 1;
        let _a = qs(v, lo, p - 1);
        qs(v, p + 1, hi)
    }
};
fn main() -> i64 = {
    let n: i64 = read_int();
    let v = vec_new();
    for _i in 0..n { vec_push(v, read_int()) };
    if n > 0 { let _s = qs(v, 0, n - 1) } else { () };
    let mut first: i64 = 1;
    for i in 0..n {
        if first == 0 { print_str(" ") } else { () };
        print(vec_get(v, i)); set first = 0
    };
    println_str("");
    0
};
