# Topological Sort

Output a valid topological ordering of a DAG using Kahn's algorithm (BFS).

## Input
- First integer: `n` (nodes numbered 0..n-1)
- Second integer: `m` (number of directed edges)
- Next `2*m` integers: `u1 v1 u2 v2 ...` (each pair `u v` means edge u → v)

## Output
Nodes in topological order, space-separated on one line.

## Algorithm hint

Store all edges as two parallel arrays `edge_from[]` and `edge_to[]` (read all edges at once). Compute in-degrees in the same pass. Then use BFS with a front-pointer queue:
1. Push all nodes with in-degree 0 (scanning 0..n in order)
2. Dequeue front node u, add to result
3. Scan all m edges: for each edge (u, v), decrement in-degree of v; if it becomes 0, enqueue v

This is O(n*m) but correct and simple to implement.

## Example

Input:
```
4 4 0 1 0 2 1 3 2 3
```
Output:
```
0 1 2 3
```

Input:
```
3 0
```
Output:
```
0 1 2
```
(no edges — output 0 1 2 in order)

Input:
```
4 2 0 1 2 3
```
Output:
```
0 2 1 3
```

## Constraints

- 1 <= n <= 100
- 0 <= m <= 500
- Graph is a valid DAG (no cycles)

## Category

Algorithm (topological sort / Kahn's BFS)

## BMB Notes
- Use two parallel vecs for edges (`edge_from`, `edge_to`) + in-degree vec
- Queue = vec with `front` index (front-pointer pattern, no vec_pop for queue)
- Scan all m edges each BFS step (O(n*m) — correct and simple)
- Output space-separated with first-flag pattern
```bmb
fn main() -> i64 = {
    let n: i64 = read_int();
    let m: i64 = read_int();
    let edge_from = vec_new();
    let edge_to = vec_new();
    let indeg = vec_new();
    let mut ii: i64 = 0;
    while ii < n { let _p = vec_push(indeg, 0); set ii = ii + 1 };
    let mut i: i64 = 0;
    while i < m {
        let u: i64 = read_int();
        let v: i64 = read_int();
        let _pu = vec_push(edge_from, u);
        let _pv = vec_push(edge_to, v);
        let _si = vec_set(indeg, v, vec_get(indeg, v) + 1);
        set i = i + 1
    };
    let queue = vec_new();
    let mut front: i64 = 0;
    let mut qi: i64 = 0;
    while qi < n {
        if vec_get(indeg, qi) == 0 { let _pq = vec_push(queue, qi) } else { () };
        set qi = qi + 1
    };
    let result = vec_new();
    while front < vec_len(queue) {
        let u: i64 = vec_get(queue, front);
        set front = front + 1;
        let _pr = vec_push(result, u);
        let mut j: i64 = 0;
        while j < m {
            if vec_get(edge_from, j) == u {
                let v: i64 = vec_get(edge_to, j);
                let new_indeg: i64 = vec_get(indeg, v) - 1;
                let _sd = vec_set(indeg, v, new_indeg);
                if new_indeg == 0 { let _pq = vec_push(queue, v) } else { () }
            } else { () };
            set j = j + 1
        }
    };
    let rn = vec_len(result);
    let mut ri: i64 = 0;
    while ri < rn {
        if ri > 0 { let _ps = print_str(" ") } else { () };
        print(vec_get(result, ri));
        set ri = ri + 1
    };
    let _pn = println_str("");
    0
};
```
