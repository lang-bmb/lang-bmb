# Count Frequency

## Description

Count how many times a target value appears in an array.

**Input** (stdin):
- First integer: `target`, the value to count
- Second integer: `n`, the number of elements (0 <= n <= 100000)
- Next `n` integers: the array elements

**Output** (stdout):
- Print the count of occurrences of target

## Example

Input:
```
3 7 1 2 3 3 3 4 5
```

Output:
```
3
```

## Constraints

- 0 <= n <= 100000
- All values fit in a 64-bit signed integer

## Category

System (array processing)

## BMB Notes
- Simple linear scan; no array storage needed, just count on the fly
```
let target: i64 = read_int();
let n: i64 = read_int();
let mut count: i64 = 0;
for _i in 0..n {
    let v: i64 = read_int();
    if v == target { set count = count + 1 } else { () }
};
println(count);
0

```
