# Trapping Rain Water

## Description

Given n non-negative integers representing an elevation map, compute how much water it can trap after raining.

**Input** (stdin):
- First line: n
- Next n lines: one height per line

**Output** (stdout):
- Total units of water trapped

## Example

Input:
```
12
0
1
0
2
1
0
1
3
2
1
2
1
```

Output:
```
6
```

## Constraints

- 1 <= n <= 100000
- 0 <= heights[i] <= 10000

## Category

Two Pointers

## BMB Notes
- Two-pointer approach: lo from left, hi from right
- Track max heights seen from each side
- Water at position = max_side - current_height (when current < max)

```
fn trap(arr: i64, n: i64, lo: i64, hi: i64, lo_max: i64, hi_max: i64, total: i64) -> i64 =
    if lo >= hi { total }
    else {
        let lv = vec_get(arr, lo);
        let rv = vec_get(arr, hi);
        if lv <= rv {
            let new_max = if lv > lo_max { lv } else { lo_max };
            let water = if new_max > lv { new_max - lv } else { 0 };
            trap(arr, n, lo + 1, hi, new_max, hi_max, total + water)
        } else {
            let new_max = if rv > hi_max { rv } else { hi_max };
            let water = if new_max > rv { new_max - rv } else { 0 };
            trap(arr, n, lo, hi - 1, lo_max, new_max, total + water)
        }
    };
```
