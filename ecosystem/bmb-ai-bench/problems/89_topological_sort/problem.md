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
