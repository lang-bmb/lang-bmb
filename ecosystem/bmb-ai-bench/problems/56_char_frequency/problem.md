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
// CORRECT — one line per entry:
println_str(format("{} {}", val, count));   // prints "1 2"

// WRONG — separate lines:
println(val);    // "1" → WRONG
println(count);  // "2" → WRONG
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
