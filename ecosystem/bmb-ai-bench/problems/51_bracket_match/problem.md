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

## BMB Notes

**CRITICAL**: BMB has NO `loop` keyword and NO `break`. Use `while i < n { ... }` instead.
**CRITICAL**: After processing all brackets, check `vec_len(stack) == 0` — unclosed brackets mean NOT balanced.

```
fn main() -> i64 = {
    let t: i64 = read_int();
    let mut ti: i64 = 0;
    while ti < t {
        let n: i64 = read_int();
        let stack = vec_new();
        let mut i: i64 = 0;
        let mut ok: i64 = 1;
        while i < n {
            let c: i64 = read_int();
            if c == 40 || c == 91 || c == 123 {
                let _p = vec_push(stack, c)
            } else {
                if vec_len(stack) == 0 {
                    set ok = 0
                } else {
                    let top: i64 = vec_pop(stack);
                    if (c == 41 && top != 40) || (c == 93 && top != 91) || (c == 125 && top != 123) {
                        set ok = 0
                    } else { () }
                }
            };
            set i = i + 1
        };
        if ok == 1 && vec_len(stack) == 0 { println(1) } else { println(0) };
        set ti = ti + 1
    };
    0
};
```
