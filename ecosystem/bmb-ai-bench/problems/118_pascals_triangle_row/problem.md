# Pascal's Triangle Row N

## Description

Given a non-negative integer n, print the nth row (0-indexed) of Pascal's triangle, one element per line.

**Input** (stdin):
- One integer n (0 <= n <= 1000)

**Output** (stdout):
- n+1 lines, the kth line is C(n, k)

## Example

Input:
```
4
```

Output:
```
1
4
6
4
1
```

## Category

Dynamic Programming

## BMB Notes
- Build row in-place: start with [1, 0, 0, ..., 0]
- Each step updates from right to left: row[i] += row[i-1]
- Use `vec_set` / `vec_get` for in-place update

```
fn update_row(row: i64, n: i64, i: i64) -> () = {
    if i > 0 {
        let j = n - i;
        vec_set(row, j, vec_get(row, j) + vec_get(row, j - 1))
    }
};
```
