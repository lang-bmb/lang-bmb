# Char Frequency

Output the frequency of each distinct value in sorted order.

## Input
- First integer: n
- Next n integers: the array

## Output
- First line: number of distinct values
- Then for each distinct value ascending: `value count` on **one line** (space-separated)

## IMPORTANT: Output Format

Each entry is ONE line: value and count separated by a single space. No leading/trailing spaces.
```
// CORRECT — one line per entry (use print + print_str + println):
print(val); print_str(" "); println(count);   // prints "1 2"

// WRONG — separate lines:
println(val);    // "1" → WRONG
println(count);  // "2" → WRONG

// WRONG — format("{} {}", ...) prints literal "{} {}" not values:
println_str(format("{} {}", val, count));  // WRONG
```

## Example
Input: `5 1 2 1 3 2` → 3 distinct values; 1 appears 2×, 2 appears 2×, 3 appears 1×
Output:
```
3
1 2
2 2
3 1
```
("1 2" = value 1 count 2, "2 2" = value 2 count 2, "3 1" = value 3 count 1)

## Constraints
- 1 <= n <= 100000
- All values fit in a 64-bit signed integer

## Category

Algorithm (frequency table)

## BMB Notes
- BMB has no hash map for integers; sort the input array, then count runs
- Use insertion sort to sort, then scan for runs of equal values
- Output: distinct count first, then `value count` per line using `format`
```
let n: i64 = read_int();
let arr = vec_new();
for _i in 0..n { vec_push(arr, read_int()) };
// insertion sort
for i in 1..n {
    let key: i64 = vec_get(arr, i);
    let mut j: i64 = i - 1;
    while j >= 0 && vec_get(arr, j) > key {
        vec_set(arr, j+1, vec_get(arr, j));
        j = j - 1
    };
    vec_set(arr, j+1, key)
};
// count distinct
let mut distinct: i64 = 0;
let keys = vec_new(); let counts = vec_new();
let mut i: i64 = 0;
while i < n {
    let v: i64 = vec_get(arr, i);
    let mut cnt: i64 = 1;
    while i + cnt < n && vec_get(arr, i + cnt) == v { cnt = cnt + 1 };
    vec_push(keys, v); vec_push(counts, cnt);
    distinct = distinct + 1;
    i = i + cnt
};
println(distinct);
for idx in 0..distinct {
    print(vec_get(keys, idx));
    print_str(" ");
    println(vec_get(counts, idx))
};
0
```
