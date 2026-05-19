# Run Length Encode

Given an array of n integers, compute its run-length encoding.

## Input
- First integer: n
- Next n integers: the array

## Output
- First line: number of runs (consecutive equal-value groups)
- Then for each run: `value count` on **one line** (value and count space-separated)

## IMPORTANT: Output Format

Each run is ONE line: value and count separated by a single space.
```
// CORRECT — one line per run (use print + print_str + println):
print(val); print_str(" "); println(count);   // prints "1 2" or "2 3"

// WRONG — two separate lines:
println(val);    // "1" on one line
println(count);  // "2" on next line — WRONG

// WRONG — format("{} {}", ...) does NOT substitute {} placeholders:
println_str(format("{} {}", val, count));  // prints literal "{} {}" — WRONG
```

## Example
Input: `5 1 1 2 2 2` → array = [1,1,2,2,2]
Output:
```
2
1 2
2 3
```
(2 runs: "1 2" means value=1 count=2; "2 3" means value=2 count=3)

## Constraints
- 1 <= n <= 100000
- All values fit in a 64-bit signed integer

## Category

Algorithm (run-length encoding)
