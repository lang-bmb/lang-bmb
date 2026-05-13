# Registry Pattern

## Description

Implement a simple key-value registry supporting set, get, and count operations.

The registry stores integer key→value pairs. Keys are unique (setting an existing key overwrites its value). Implement three operations:

- **Op 1 (set)**: `1 k v` — store value `v` under key `k`. If key already exists, overwrite.
- **Op 2 (get)**: `2 k` — print the value for key `k`, or -1 if not found
- **Op 3 (count)**: `3` — print the number of distinct keys currently stored

**Input** (stdin):
- Integer `n`: total number of commands
- For each command:
  - `1 k v`: set operation (key k, value v)
  - `2 k`: get operation (key k)
  - `3`: count operation

**Output** (stdout):
- For each get command: print the value or -1
- For each count command: print the count
- One output per get/count command

## Example

Input:
```
3 1 1 100 2 1 3
```

Output:
```
100
1
```

Explanation:
- Command 1: set(k=1, v=100) — registry: {1→100}
- Command 2: get(k=1) → print 100
- Command 3: count → print 1 (one key stored)

Another example:

Input:
```
2 2 5 3
```

Output:
```
-1
0
```

Explanation:
- get(k=5) → not found → print -1
- count → 0 (registry is empty)

## Constraints

- 1 ≤ n ≤ 1000
- -10,000,000,000 ≤ k, v ≤ 10,000,000,000
- At most 1000 distinct keys
- Operations are processed in order

## Key Notes

- Op 1 reads two integers (key and value), op 2 reads one integer (key), op 3 reads no integers
- Set overwrites existing key — count does NOT increase for duplicates
- Count reflects the current number of distinct keys at the time of the query

## Category

Data structures / simulation / registry
