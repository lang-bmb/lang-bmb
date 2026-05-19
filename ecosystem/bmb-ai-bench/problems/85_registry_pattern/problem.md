# Registry Pattern

Implement a simple key-value registry supporting set, get, and count operations.

The registry stores integer key→value pairs. Keys are unique (setting an existing key overwrites its value).

- **Op 1 (set)**: `1 k v` — store value `v` under key `k`. Overwrite if key exists.
- **Op 2 (get)**: `2 k` — print the value for key `k`, or -1 if not found
- **Op 3 (count)**: `3` — print the number of distinct keys currently stored

## Input

- Integer `n`: total number of commands
- For each command:
  - `1 k v`: set operation (reads 2 integers: key k, value v)
  - `2 k`: get operation (reads 1 integer: key k)
  - `3`: count operation (reads no extra integers)

## Output

- For each get command: print the value or -1
- For each count command: print the count
- One output per get/count command

## Example

Input: `3 1 1 100 2 1 3`
Output:
```
100
1
```
- Command 1: set(k=1, v=100) — registry: {1→100}
- Command 2: get(k=1) → 100
- Command 3: count → 1 (one key stored)

Input: `2 2 5 3`
Output:
```
-1
0
```
- get(k=5) → not found → -1
- count → 0 (empty)

## IMPORTANT: Parallel Array Implementation

BMB's `str_hashmap` uses string keys. For integer keys, use parallel arrays:
- One `keys` vector to store keys
- One `vals` vector to store corresponding values
- A `size` counter for distinct keys

```
fn reg_get(keys: i64, vals: i64, size: i64, k: i64) -> i64 = {
    let mut i: i64 = 0;
    let mut result: i64 = -1;
    while i < size {
        if vec_get(keys, i) == k { set result = vec_get(vals, i) } else { () };
        set i = i + 1;
    };
    result
};

fn reg_set(keys: i64, vals: i64, size_ref: i64, k: i64, v: i64) -> i64 = {
    // find if key exists
    let mut i: i64 = 0;
    let mut found: i64 = -1;
    while i < vec_get(size_ref, 0) {
        if vec_get(keys, i) == k { set found = i } else { () };
        set i = i + 1;
    };
    if found >= 0 {
        vec_set(vals, found, v)
    } else {
        let _pk = vec_push(keys, k);
        let _pv = vec_push(vals, v);
        vec_set(size_ref, 0, vec_get(size_ref, 0) + 1)
    }
};
```

Or simpler: use linear scan with a `count` variable, checking each key in order.

## Key Notes

- Op 1 reads **two** integers (k then v); op 2 reads **one** integer (k); op 3 reads **none**
- **Op 1 (set): NO output** — do not print anything for set operations
- Set overwrites existing key — count does NOT increase for duplicate sets
- Count reflects distinct keys at query time (not total set operations)

**CRITICAL**: Only op 2 (get) and op 3 (count) produce output. Op 1 (set) produces NO output at all. Do not call `println` inside the set branch.

## Constraints

- 1 ≤ n ≤ 1000
- At most 1000 distinct keys
- Operations are processed in order
