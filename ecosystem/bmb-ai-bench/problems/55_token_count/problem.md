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
- BMB has no built-in set or hash map for integers. Use linear search to check if a value appeared before.
```
// For each element, check all previous elements:
for i in 0..n {
    let val = vec_get(v, i);
    let mut found: i64 = 0;
    for j in 0..i {
        if vec_get(v, j) == val { found = 1; break } else { () }
    };
    if found == 0 { count = count + 1 } else { () }
}
```
