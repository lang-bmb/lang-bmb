# bmb-algo — Blazing Fast Algorithms

> Up to ~450× faster than pure Python on DP workloads at scale (knapsack(100) at v0.98, 2026-05-12).

High-performance algorithms compiled from [BMB](https://github.com/iyulab/lang-bmb), a language where compile-time contracts eliminate runtime overhead.

## Installation

**Python:**
```bash
pip install bmb-algo
```

**Node.js** (via koffi FFI — no native build required):
```bash
cd ecosystem/bmb-algo/bindings/node && npm install
```
See [`bindings/node/README.md`](bindings/node/README.md) for full Node.js API documentation.

## Benchmarks (vs Pure Python)

Measured at **v0.98** (2026-05-12). Configuration in [`benchmarks/bench_algo.py`](benchmarks/bench_algo.py).

| Algorithm | bmb-algo | Python | Speedup |
|-----------|----------|--------|---------|
| **knapsack(100 items, cap ~1300)** | 50 us | 22.5 ms | **~450×** |
| knapsack(10 items, cap 20) | 3.5 us | 24 us | ~7× |
| prime_count(10000) | 9 us | 448 us | **~49×** |
| edit_distance | 1.7 us | 14.6 us | **~8.5×** |
| nqueens(10) | 1.72 ms | 9.93 ms | **~5.8×** |
| merge_sort(15) | 4.0 us | 11.6 us | **~2.9×** |
| **quicksort(1000)** | 218 us | 548 us | **~2.5×** |
| quicksort(15) | 3.9 us | 3.6 us | ~0.9× ⚠️ |
| fibonacci(30) | 0.4 us | 0.6 us | ~1.5× |

*All timings include ctypes FFI overhead — 100-500-iteration mean after 10-iter warmup. Numbers ≈ approximations (5-10% run-to-run variance).*

### Scaling behavior

BMB's advantage **amplifies with input size**, because FFI call overhead is amortized over more algorithmic work:

| size | knapsack speedup | quicksort speedup |
|------|------------------|-------------------|
| n=10 | ~7× | ~0.9× (FFI-bound) |
| n=30 | ~170× | parity |
| n=100 | **~450×** | ~1.2× |
| n=300 | ~600× | — |
| n=1000 | — | **~2.5×** |

**Recommendation**: use bmb-algo for inputs where algorithmic work ≫ FFI overhead. Below ~100 elements / states, raw Python may match or beat bmb-algo due to ctypes marshalling cost.

### Historical measurements (archived)

`bmb-algo v0.2.0` (2026-03-23) recorded `knapsack 90.7×` and `nqueens(8) 181.6×` vs Python.
The `knapsack 90.7×` is consistent with the scaling table above (achievable at n≥30).
The `nqueens(8) 181.6×` does not reproduce at any tested size with the current `bench_algo.py` baseline — likely a different bench configuration or baseline. See [CHANGELOG.md](CHANGELOG.md) for re-baseline notes.

## Quick Start

```python
import bmb_algo

# Dynamic Programming
bmb_algo.knapsack([2, 3, 4], [3, 4, 5], 7)         # 9
bmb_algo.edit_distance("kitten", "sitting")          # 3
bmb_algo.lcs("ABCBDAB", "BDCAB")                    # 4
bmb_algo.coin_change([1, 5, 11], 15)                 # 3
bmb_algo.lis([10, 9, 2, 5, 3, 7, 101, 18])          # 4
bmb_algo.max_subarray([-2, 1, -3, 4, -1, 2, 1])     # 6

# Graph
bmb_algo.dijkstra([[0, 4, -1], [-1, 0, 2], [-1, -1, 0]], 0)  # [0, 4, 6]
bmb_algo.floyd_warshall([[0, 3, -1], [2, 0, -1], [-1, 7, 0]])
bmb_algo.bfs_count(adj_matrix, source=0)
bmb_algo.topological_sort(adj_matrix)

# Sort
bmb_algo.quicksort([5, 3, 1, 4, 2])                 # [1, 2, 3, 4, 5]
bmb_algo.merge_sort([5, 3, 1, 4, 2])                 # [1, 2, 3, 4, 5]
bmb_algo.heap_sort([5, 3, 1, 4, 2])                  # [1, 2, 3, 4, 5]
bmb_algo.counting_sort([3, 1, 4, 1, 5, 9])           # [1, 1, 3, 4, 5, 9]

# Number Theory
bmb_algo.nqueens(8)                                   # 92
bmb_algo.prime_count(100)                             # 25
bmb_algo.fibonacci(50)                                # 12586269025
bmb_algo.gcd(12, 8)                                   # 4
bmb_algo.modpow(2, 10, 1000)                          # 24
```

## Full API (55 algorithms)

### Dynamic Programming
| Function | Description |
|----------|-------------|
| `knapsack(weights, values, capacity)` | 0/1 knapsack problem |
| `edit_distance(a, b)` | Levenshtein distance |
| `lcs(a, b)` | Longest common subsequence length |
| `max_subarray(arr)` | Kadane's algorithm |
| `coin_change(coins, amount)` | Minimum coins to make amount |
| `lis(arr)` | Longest increasing subsequence length |

### Graph
| Function | Description |
|----------|-------------|
| `dijkstra(adj_matrix, source)` | Shortest paths from source |
| `floyd_warshall(matrix)` | All-pairs shortest paths |
| `bfs_count(adj_matrix, source)` | BFS reachable count |
| `topological_sort(adj_matrix)` | Topological ordering |

### Sort
| Function | Description |
|----------|-------------|
| `quicksort(arr)` | Quicksort (returns new list) |
| `merge_sort(arr)` | Merge sort (returns new list) |
| `heap_sort(arr)` | Heap sort (returns new list) |
| `counting_sort(arr)` | Counting sort (non-negative ints) |
| `shell_sort(arr)` | Shell sort |
| `insertion_sort(arr)` | Insertion sort |
| `selection_sort(arr)` | Selection sort |
| `bubble_sort(arr)` | Bubble sort (with early termination) |

### Search
| Function | Description |
|----------|-------------|
| `binary_search(arr, target)` | Binary search, returns index or -1 |

### Number Theory
| Function | Description |
|----------|-------------|
| `gcd(a, b)` | Greatest common divisor |
| `lcm(a, b)` | Least common multiple |
| `fibonacci(n)` | Nth Fibonacci number |
| `prime_count(n)` | Count primes up to n |
| `nqueens(n)` | N-Queens solution count |
| `modpow(base, exp, mod)` | Modular exponentiation |
| `is_prime(n)` | Primality test |

### Matrix
| Function | Description |
|----------|-------------|
| `matrix_multiply(a, b)` | Matrix multiplication |
| `matrix_transpose(m)` | Transpose matrix |
| `matrix_det(m)` | Determinant (Gaussian elimination) |

### Utility
| Function | Description |
|----------|-------------|
| `djb2_hash(s)` | DJB2 string hash |
| `power_set_size(n)` | 2^n |
| `is_sorted(arr)` | Check if sorted |
| `array_reverse(arr)` | Reverse array |
| `array_rotate(arr, k)` | Rotate left by k |
| `unique_count(sorted_arr)` | Count unique values |
| `prefix_sum(arr)` | Prefix sum array |
| `array_sum(arr)` | Sum of elements |
| `array_min(arr)` / `array_max(arr)` | Min/max element |
| `bit_popcount(x)` | Population count |
| `bit_set(v, pos)` / `bit_clear(v, pos)` / `bit_test(v, pos)` | Bit operations |
| `array_contains(arr, target)` | Membership test |
| `array_index_of(arr, target)` | Find index (-1 if missing) |
| `array_product(arr)` | Product of all elements |
| `subset_sum(arr, target)` | Subset sum check (DP) |

## How?

Written in [BMB](https://github.com/iyulab/lang-bmb) — a language where compile-time contracts prove correctness, then generate code faster than hand-tuned C. Safety isn't a separate goal; it's the natural consequence of pursuing maximum performance.

## License

MIT
