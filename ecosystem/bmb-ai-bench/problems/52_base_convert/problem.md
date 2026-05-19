# Base Convert

Convert decimal n to base b. Output each digit as its decimal value, concatenated.

## Input
- First integer: **t** (number of test cases)
- Each test case: **number base** (two integers)

## Output
For each: digits of (number in base b), each digit printed as decimal, concatenated (one per line)

## Notes
- Special case: 0 → "0"
- For base 16: digit 15 prints as "15" (two chars), not 'F'

## Example
- 10 in base 2 → digits [1,0,1,0] → "1010"
- 255 in base 16 → digits [15,15] → "1515"

## IMPORTANT: t is the query count

```
let t: i64 = read_int();
let mut i: i64 = 0;
while i < t {
    let number: i64 = read_int();
    let base: i64 = read_int();
    // convert number to base, collect digits in reverse, then output concatenated
    // Use str_concat to build the result string
    set i = i + 1;
};
0
```

## String Building Hint

Use `str_concat(a, b)` to concatenate and `format("{}", digit)` to convert digit to string.
Collect digits by repeatedly doing `number % base`, then reverse.
