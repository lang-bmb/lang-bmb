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
