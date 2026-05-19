# LRU Simulate

LRU cache simulation: count total cache misses.

## Input
- First integer: capacity
- Second integer: n
- Next n integers: memory addresses accessed

## Output
Total cache misses (single integer).

## Algorithm
On each access: if in cache (hit), mark as MRU; if not (miss), add to cache, evict LRU if full.

## Example
`3 7 1 2 3 4 1 2 5` -> 7 misses (items 1,2,3 evicted before re-access)

## BMB Notes
- Maintain a vec of capacity where most-recently-used is at the end.
- On hit: remove from current position, append to end (shift elements).
- On miss: if full, remove index 0 (LRU), append new page; else just append.
- BMB has no built-in linked list; use a vec with linear search and shift.
```
let cache = vec_new();
let mut faults: i64 = 0;
for _i in 0..n {
    let page: i64 = read_int();
    let len: i64 = vec_len(cache);
    let mut found: i64 = -1;
    for j in 0..len {
        if vec_get(cache, j) == page { found = j; break } else { () }
    };
    // on hit: shift page to end; on miss: evict LRU if full, append
};
println(faults);
```
