# bmb-compute

High-performance compute functions powered by [BMB](https://github.com/iyulab/lang-bmb) — Node.js FFI bindings.

## Installation

```bash
npm install bmb-compute koffi
```

Requires the `bmb_compute` native shared library. See [Getting the native library](#getting-the-native-library).

## Getting the native library

**Option A — Download from GitHub Releases (recommended):**
1. Go to [lang-bmb Releases](https://github.com/iyulab/lang-bmb/releases)
2. Download `bmb-libs-<your-platform>.zip` from the latest release
3. Place `bmb_compute.dll` / `libbmb_compute.so` / `libbmb_compute.dylib` next to `index.js`

**Option B — Build from source:**
```bash
cd /path/to/lang-bmb
cargo build --release
./target/release/bmb build ecosystem/bmb-compute/src/lib.bmb --shared -o ecosystem/bmb-compute/bmb_compute
```

## Functions

| Function | Description |
|----------|-------------|
| `abs(n)` | Absolute value |
| `min(a, b)` | Minimum |
| `max(a, b)` | Maximum |
| `clamp(n, lo, hi)` | Clamp to range |
| `sign(n)` | Sign (-1/0/1) |
| `ipow(base, exp)` | Integer power |
| `isqrt(n)` | Integer square root |
| `factorial(n)` | Factorial |
| `rand_next(seed)` | XorShift64* next value |
| `rand_u64(seed)` | Uniform u64 |
| `rand_range(seed, lo, hi)` | Random in range |
| `sum(arr)` | Array sum (scaled ×1000) |
| `mean(arr)` | Mean (scaled ×1000) |
| `variance(arr)` | Variance (scaled ×1000) |
| `median(arr)` | Median (scaled ×1000) |
| `dot_product(a, b)` | Dot product |
| `dist_squared(a, b)` | Squared distance |
| `weighted_sum(vals, weights)` | Weighted sum (scaled) |
| `vec_sum(arr)` | Vector sum |
| `vec_max(arr)` | Vector maximum |
| `vec_min(arr)` | Vector minimum |
| `vec_count_positive(arr)` | Count positive elements |
| `vec_count_negative(arr)` | Count negative elements |
| `vec_count_zero(arr)` | Count zero elements |
| `vec_range(arr)` | max - min |
| `vec_argmax(arr)` | Index of maximum |
| `vec_argmin(arr)` | Index of minimum |

## License

MIT
