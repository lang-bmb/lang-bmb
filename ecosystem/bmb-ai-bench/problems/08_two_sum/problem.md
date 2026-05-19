# Two Sum

## Description

Given an array of integers and a target sum, find two distinct indices whose elements sum to the target. Use a brute-force O(n²) approach.

**Input** (stdin):
- First integer: `target`, the target sum
- Second integer: `n`, the number of elements (2 <= n <= 1000)
- Next `n` integers: the array elements

**Output** (stdout):
- Print the two 0-based indices (smaller index first), separated by a space
- A valid pair is guaranteed to exist

## Example

Input:
```
9 4 2 7 11 15
```

Output:
```
0 1
```

(arr[0] + arr[1] = 2 + 7 = 9)

## Constraints

- 2 <= n <= 1000
- Exactly one valid pair exists
- All values fit in a 64-bit signed integer

## Category

Algorithm (search)

## BMB Notes
- O(n²) brute force: nested loops, break out when pair found using a `found` flag
- Read target, then n, then all elements into a vec
```
let target: i64 = read_int();
let n: i64 = read_int();
let v = vec_new();
for _i in 0..n { vec_push(v, read_int()) };
let mut ri: i64 = 0; let mut rj: i64 = 0; let mut found: i64 = 0;
let mut i: i64 = 0;
while i < n {
    let mut j: i64 = i + 1;
    while j < n {
        if found == 0 {
            if vec_get(v, i) + vec_get(v, j) == target {
                set ri = i; set rj = j; set found = 1
            } else { () }
        } else { () };
        set j = j + 1
    };
    set i = i + 1
};
print(ri); print_str(" "); println(rj);
0

```
