# bmb-algo — Blazing Fast Algorithms

> Up to ~245× faster than pure Python on DP workloads (knapsack(100), median of 5 runs at v0.98, 2026-05-12). Scales to ~300× at n=300.

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

Measured at **v0.98** (2026-05-12), **median of 5 runs**. Reproduce with `python benchmarks/bench_algo.py --runs=5`.

| Algorithm | bmb-algo | Python | Speedup (median) | spread |
|-----------|----------|--------|------------------|--------|
| **knapsack(100 items, cap ~1300)** | 42 us | 10.2 ms | **~243×** | 235-257× |
| knapsack(10 items, cap 20) | 2.1 us | 12.9 us | ~6.2× | 6.0-6.5× |
| prime_count(10000) | 11 us | 329 us | **~31×** | 30-31× |
| edit_distance¹ | 1.4 us | 9.3 us | **~6.6×** | 4.9-7.0× |
| nqueens(10) | 1.74 ms | 6.90 ms | **~4.0×** | 3.9-4.0× |
| **quicksort(1000)** | 100 us | 488 us | **~4.9×** | 4.8-4.9× |
| merge_sort(15) | 2.1 us | 6.5 us | **~3.2×** | 3.0-3.2× |
| fibonacci(30) | 0.23 us | 0.51 us | ~2.2× | 2.1-2.3× |
| quicksort(15) | 2.0 us | 3.4 us | ~1.7× | 1.6-1.7× |

*All timings include ctypes FFI overhead — 50-500-iteration mean per sample, 10-iter warmup. Inter-run variance ~5% on stable runs; `edit_distance` shows occasional outliers (1 of 5 measured 4.87× — Python L1-cache miss susceptibility).*

¹ `edit_distance`/`lcs` were since reimplemented with rolling 2-row DP — correctness identical, large inputs ~1.7–2.5× faster. The small-input row above predates that change; rerun `bench_algo.py --runs=5` to refresh.

> **How to read these numbers.** "vs pure Python" is an anchor — any compiled language beats
> CPython, so the multiplier alone is not the value proposition. The meaningful comparison is
> against the ecosystem incumbents (NumPy / Cython): on DP/combinatorial kernels NumPy can't
> vectorize (e.g. knapsack) bmb-algo is competitive-to-faster, and for bulk reductions the
> [zero-copy input path](#zero-copy-inputs-numpy--arrayarray) reaches/beats NumPy's own routines.
> Incumbent-relative measurements: see the lang-bmb `claudedocs/measurements/` AL-3 report.

### Scaling behavior

BMB's advantage **amplifies with input size**, because FFI call overhead is amortized over more algorithmic work:

| size | knapsack speedup | quicksort speedup |
|------|------------------|-------------------|
| n=10 | ~29× | — |
| n=15 | — | ~1.6× (FFI overhead 큰 비중) |
| n=30 | ~119× | — |
| n=50 | — | ~3.0× |
| n=100 | **~246×** | ~3.0× |
| n=300 | **~306×** | — |
| n=500 | — | ~4.4× |
| n=1000 | — | **~4.7×** |

Reproduce with `python benchmarks/bench_algo.py --runs=5 --scaling` (adds ~30s).

**Recommendation**: use bmb-algo for inputs where algorithmic work ≫ FFI overhead. For DP workloads (knapsack, edit_distance), the advantage is already material at n≈10 and grows rapidly with state count. For sorting, the crossover where BMB clearly wins is around n≈30–50.

### Historical measurements (archived)

`bmb-algo v0.2.0` (2026-03-23) recorded `knapsack 90.7×` and `nqueens(8) 181.6×` vs Python.
The `knapsack 90.7×` reproduces in scale (current median-of-5 puts knapsack(n=30) at ~119×, which is in the ballpark of the v0.2.0 number).
The `nqueens(8) 181.6×` does not reproduce against the current `py_nqueens` baseline (n=8/10/12 all measure ~4-8× speedup) — likely a different bench configuration in v0.2.0.

A more recent intermediate measurement on 2026-05-12 (Cycle 2754) reported `knapsack(100) ~450×` and `quicksort(15) ~0.9× SLOW`. **Neither reproduces against the current median-of-5 baseline** — the Cycle 2754 sample was n=2 and appears to have hit unusual system conditions. The numbers in the table above (median of 5 back-to-back runs on the same machine, within 5% inter-run variance) are the durable baseline. See [CHANGELOG.md](CHANGELOG.md) for full re-baseline notes.

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

### Zero-copy inputs (NumPy / `array.array`)

Array-taking **read-only** functions (`array_sum`, `array_min/max`, `max_subarray`, `lis`,
`coin_change`, `knapsack`, `binary_search`, `is_sorted`, `unique_count`, `subset_sum`,
`array_product`, `array_contains`, `array_index_of`) accept a Python `list`, a NumPy `int64`
array, or an `array.array('q')`. NumPy/`array` inputs are passed **zero-copy** — their buffer
address goes straight to the native code with no per-element marshalling:

```python
import numpy as np, bmb_algo
a = np.random.randint(0, 1000, size=1_000_000, dtype=np.int64)
bmb_algo.array_sum(a)        # zero-copy: ~500× faster than passing a Python list,
                             # and faster than numpy's own a.sum() on this workload
```

For million-element arrays the list path is dominated by marshalling; the zero-copy path removes
it entirely. Non-int64 or non-contiguous NumPy arrays are converted with a single contiguous copy
(still far cheaper than element-by-element). Read-only functions never mutate the caller's buffer.
Sorting functions (which return a new list) keep the copy path and are unaffected.

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
