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
