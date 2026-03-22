# bmb-compute — Numeric Computation

Math, statistics, random numbers, and vector operations compiled from [BMB](https://github.com/iyulab/lang-bmb).

## Functions (20)

### Math
- `abs(x)`, `min(a, b)`, `max(a, b)`, `clamp(x, lo, hi)`, `sign(x)`
- `ipow(base, exp)` — Integer power
- `sqrt(n)` — Integer square root (Newton's method)
- `factorial(n)` — Factorial (up to 20!)

### Statistics
- `sum(arr)` — Sum of elements
- `mean_scaled(arr)` — Mean x 1000 (3 decimal places)
- `min_val(arr)`, `max_val(arr)` — Min/max
- `range_val(arr)` — Range (max - min)
- `variance_scaled(arr)` — Variance x 1000000 (6 decimal places)

### Random (XorShift64*)
- `rand_seed(seed)` — Initialize PRNG
- `rand_next(state)` — Next state
- `rand_pos(state)` — Positive random value
- `rand_range(state, max)` — Random in [0, max)

### Vector
- `dot_product(a, b)` — Dot product
- `dist_squared(a, b)` — Euclidean distance squared

## Quick Start

```python
import bmb_compute

bmb_compute.sqrt(144)       # 12
bmb_compute.factorial(10)   # 3628800
bmb_compute.ipow(2, 20)     # 1048576

bmb_compute.sum([10, 20, 30])       # 60
bmb_compute.mean_scaled([10, 20, 30])  # 20000 (= 20.000)

bmb_compute.dot_product([1,2,3], [4,5,6])  # 32
bmb_compute.dist_squared([0,0], [3,4])     # 25
```

Powered by [BMB](https://github.com/iyulab/lang-bmb) — *Performance > Everything*.
