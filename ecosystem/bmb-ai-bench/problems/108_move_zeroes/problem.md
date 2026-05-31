# Move Zeroes

## Description

Given an array, move all zeros to the end while maintaining the relative order of non-zero elements.

**Input** (stdin):
- First integer: `n`, array size (1 <= n <= 10000)
- Next `n` integers: array elements

**Output** (stdout):
- Space-separated elements after moving zeroes to end

## Example

Input:
```
5 0 1 0 3 12
```

Output:
```
1 3 12 0 0
```

## Constraints

- 1 <= n <= 10000
- All values fit in i64

## Category

Algorithm (array)

## BMB Notes
- Two-pass: collect non-zeros, then append zeros
- Or in-place with write pointer pattern
```
fn main() -> i64 = {
    let n: i64 = read_int();
    let v = vec_new(); let out = vec_new();
    for _i in 0..n { let x = read_int(); vec_push(v, x) };
    for i in 0..n { let x = vec_get(v, i); if x != 0 { vec_push(out, x) } else { () } };
    let nz = vec_len(out);
    for _i in nz..n { vec_push(out, 0) };
    for i in 0..n {
        if i > 0 { print_str(" ") } else { () };
        print(vec_get(out, i))
    };
    println_str("");
    vec_free(v); vec_free(out);
    0
};
```
