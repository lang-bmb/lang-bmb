# Jump Game

## Description

Given an array of non-negative integers, where each element represents your maximum jump length from that position, determine if you can reach the last index starting from index 0.

**Input** (stdin):
- First line: n
- Next n lines: one jump value per line

**Output** (stdout):
- `"true"` if you can reach the last index, `"false"` otherwise

## Example

Input:
```
5
2
3
1
1
4
```

Output:
```
true
```

## Constraints

- 1 <= n <= 100000
- 0 <= jumps[i] <= 100000

## Category

Greedy

## BMB Notes
- Track maximum reachable index (greedy approach)
- If current index > max_reach, we're stuck
- At each position, update max_reach = max(max_reach, i + arr[i])

```
fn scan(arr: i64, n: i64, i: i64, max_reach: i64) -> bool =
    if i >= n { true }
    else if i > max_reach { false }
    else {
        let reach = i + vec_get(arr, i);
        let new_max = if reach > max_reach { reach } else { max_reach };
        scan(arr, n, i + 1, new_max)
    };
```
