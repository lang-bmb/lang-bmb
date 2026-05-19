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

**CRITICAL**: op=3 stats output is ONE line: `total_bytes count` space-separated. Use `print(total); print_str(" "); println(count)`.

- Use two parallel vecs: `sizes` (allocation sizes) and `active` (1=live, 0=freed)
- Allocation ID = index in these vecs (0-indexed, monotone increasing)
- op=2 does NOT print anything (just marks active[id]=0)

```
fn main() -> i64 = {
    let n: i64 = read_int();
    let sizes = vec_new();
    let active = vec_new();
    let mut next_id: i64 = 0;
    let mut i: i64 = 0;
    while i < n {
        let op: i64 = read_int();
        if op == 1 {
            let sz: i64 = read_int();
            let _p1 = vec_push(sizes, sz);
            let _p2 = vec_push(active, 1);
            println(next_id);
            set next_id = next_id + 1
        } else {
            if op == 2 {
                let id: i64 = read_int();
                let _s = vec_set(active, id, 0)
            } else {
                let mut total: i64 = 0;
                let mut cnt: i64 = 0;
                let mut j: i64 = 0;
                while j < next_id {
                    if vec_get(active, j) == 1 {
                        set total = total + vec_get(sizes, j);
                        set cnt = cnt + 1
                    } else { () };
                    set j = j + 1
                };
                print(total); print_str(" "); println(cnt)
            }
        };
        set i = i + 1
    };
    0
};
```
