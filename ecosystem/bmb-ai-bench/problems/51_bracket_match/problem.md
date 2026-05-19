# Bracket Match

Check if a bracket sequence is balanced. Brackets are given as ASCII codes.

## Input
- First integer: t
- Each test case: n followed by n ASCII codes
  - 40='(' 41=')' 91='[' 93=']' 123='{' 125='}'

## Output
For each test case: 1 if balanced, 0 if not (one per line)

## Notes
- Empty (n=0) is balanced
- Pairs: '(' ')'; '[' ']'; '{' '}'

## Example
`1 4 40 91 93 41` -> "([])" -> 1

## BMB Notes
- Use `vec_new()` / `vec_push(stack, v)` / `vec_pop(stack)` / `vec_len(stack)` for stack operations
- `||` works for boolean OR: `if c == 41 || c == 93 || c == 125 { ... }`
- For bitwise operations use `band`/`bor`/`bxor` (not `&`/`|`/`^`)
- Read input with `read_int()` for each number

## BMB Sketch
```
fn solve_case(n: i64) -> i64 = {
    let stack = vec_new();
    let mut i = 0;
    let mut ok = 1;
    loop {
        if i >= n { break } else { () };
        let c = read_int();
        if c == 40 || c == 91 || c == 123 {
            let _p = vec_push(stack, c);
            ()
        } else {
            if vec_len(stack) == 0 {
                set ok = 0;
                ()
            } else {
                let top = vec_pop(stack);
                if (c == 41 && top != 40) || (c == 93 && top != 91) || (c == 125 && top != 123) {
                    set ok = 0;
                    ()
                } else { () }
            }
        };
        set i = i + 1
    };
    if ok == 1 && vec_len(stack) == 0 { 1 } else { 0 }
};
```
