# Memory Pool

Memory pool with alloc, free, and stats operations.

## Input
- First integer: n (operations)
- Operations:
  - op=1 (alloc): read size; print next allocation ID (0-indexed); track allocation
  - op=2 (free): read id; free that allocation
  - op=3 (stats): print `total_bytes count` (live allocations only)

## Output
op=1 prints ID; op=3 prints stats.

## Example
`5 1 100 1 200 3 2 0 3` -> alloc(100)->0, alloc(200)->1, stats->300 2, free(0), stats->200 1

## BMB Notes
- Use two parallel vecs: `sizes` (allocation sizes) and `active` (1=live, 0=freed)
- Allocation ID = index in these vecs (0-indexed, monotone increasing)
```
let sizes = vec_new();
let active = vec_new();
let mut next_id: i64 = 0;
// op=1 alloc(sz): push sz to sizes, push 1 to active, println(next_id), next_id++
// op=2 free(id): vec_set(active, id, 0)
// op=3 stats: scan all, sum sizes[j] where active[j]==1, count them
```
