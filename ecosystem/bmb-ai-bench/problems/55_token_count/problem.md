# Token Count

Count the number of distinct values in an array of n integers.

## Input
- First integer: n
- Next n integers: the array

## Output
Count of distinct values (one integer)

## Example
`5 1 2 3 2 1` -> 3 distinct (1,2,3)

## BMB Notes

**CRITICAL**: BMB has NO `break` statement. To exit early from a loop, use a flag variable with `set`.
**CRITICAL**: Use `set count = count + 1` NOT `count = count + 1`. All variable updates need `set`.

```
fn main() -> i64 = {
    let n: i64 = read_int();
    let v = vec_new();
    let mut i: i64 = 0;
    while i < n {
        let _p = vec_push(v, read_int());
        set i = i + 1
    };
    let mut count: i64 = 0;
    let mut idx: i64 = 0;
    while idx < n {
        let val: i64 = vec_get(v, idx);
        let mut found: i64 = 0;
        let mut j: i64 = 0;
        while j < idx {
            if vec_get(v, j) == val { set found = 1 } else { () };
            set j = j + 1
        };
        if found == 0 { set count = count + 1 } else { () };
        set idx = idx + 1
    };
    println(count);
    0
};
```
