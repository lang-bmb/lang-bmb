# Zigzag Print

Arrange n items into rows of w. Print odd rows left-to-right, even rows right-to-left.

## Input
- First integer: n
- Second integer: w (row width)
- Next n integers: items in order

## Output
Each row space-separated on its own line. Row 1 forward, row 2 backward, etc.

## Example
`6 3 1 2 3 4 5 6` -> row1 forward [1,2,3], row2 backward [6,5,4]
Output:
```
1 2 3
6 5 4
```

## BMB Notes

**CRITICAL**: Use `set j = j + 1` (NOT `j = j + 1`). BMB requires `set` for ALL variable updates.
**CRITICAL**: Use `if row % 2 == 0 { ... } else { ... }` directly. Do NOT assign to a bool and compare with `== 1`.
  - WRONG: `let is_even = row % 2 == 0; if is_even == 1 { ... }` — type error (bool vs i64)
  - CORRECT: `if row % 2 == 0 { ... } else { ... }`

```
fn main() -> i64 = {
    let n: i64 = read_int();
    let w: i64 = read_int();
    let v = vec_new();
    let mut k: i64 = 0;
    while k < n {
        let _p = vec_push(v, read_int());
        set k = k + 1
    };
    let mut row: i64 = 0;
    let mut i: i64 = 0;
    while i < n {
        let actual_end: i64 = if i + w < n { i + w } else { n };
        if row % 2 == 0 {
            let mut j: i64 = i;
            while j < actual_end {
                if j > i { print_str(" ") } else { () };
                print(vec_get(v, j));
                set j = j + 1
            }
        } else {
            let mut j: i64 = actual_end - 1;
            let mut first: i64 = 1;
            while j >= i {
                if first == 0 { print_str(" ") } else { () };
                print(vec_get(v, j));
                set first = 0;
                set j = j - 1
            }
        };
        println_str("");
        set i = i + w;
        set row = row + 1
    };
    0
};
```
