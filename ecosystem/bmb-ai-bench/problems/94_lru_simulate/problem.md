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
- Maintain a vec where most-recently-used is at the end (index vec_len-1).
- On hit: remove from current position (`vec_remove`), push to end.
- On miss: if full, remove index 0 (LRU), push new page; else just push. Increment faults.
- BMB has NO `break`. Use linear scan storing the last found index:
```
fn main() -> i64 = {
    let capacity: i64 = read_int();
    let n: i64 = read_int();
    let cache = vec_new();
    let mut faults: i64 = 0;
    for _i in 0..n {
        let page: i64 = read_int();
        let len: i64 = vec_len(cache);
        let mut found: i64 = -1;
        for j in 0..len {
            if vec_get(cache, j) == page { set found = j } else { () }
        };
        if found >= 0 {
            let tmp: i64 = vec_remove(cache, found);
            let _p = vec_push(cache, tmp)
        } else {
            if vec_len(cache) >= capacity {
                let _evict = vec_remove(cache, 0)
            } else { () };
            let _p = vec_push(cache, page);
            set faults = faults + 1
        }
    };
    println(faults);
    0
};
```
