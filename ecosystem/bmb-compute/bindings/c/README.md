# bmb-compute — C Bindings

C bindings for the bmb-compute library via direct shared library linking.

## Requirements

- GCC or Clang (C99+)
- `bmb_compute.dll` / `libbmb_compute.so` / `libbmb_compute.dylib` in the parent directory (`../../`)

## Build & Run

```bash
make          # build test and example
make run-test # run test suite (56 tests)
make run-example
```

### Manual build

```bash
# Windows
gcc -O2 -I../../include -o example example.c -L../.. -l:bmb_compute.dll

# Linux / macOS
gcc -O2 -I../../include -o example example.c -L../.. -lbmb_compute -Wl,-rpath,../..
```

## Usage

```c
#include "bmb_compute.h"
#include <stdio.h>

int main(void) {
    bmb_ffi_begin();

    printf("%lld\n", (long long)bmb_factorial(10));    // 3628800
    printf("%lld\n", (long long)bmb_sqrt(144));        // 12
    printf("%lld\n", (long long)bmb_c_clamp(15,0,10)); // 10

    int64_t a[] = {1, 2, 3}, b[] = {4, 5, 6};
    printf("%lld\n", (long long)bmb_dot_product((int64_t)a, (int64_t)b, 3)); // 32

    if (bmb_ffi_has_error()) {
        fprintf(stderr, "%s\n", bmb_ffi_error_message());
        bmb_ffi_end();
        return 1;
    }
    bmb_ffi_end();
    return 0;
}
```

## API Notes

### Scaled return values

Several statistical functions return integers scaled to avoid floating point:

| Function | Scale | Example |
|----------|-------|---------|
| `bmb_mean_scaled` | × 1000 | mean=30 → 30000 |
| `bmb_median_scaled` | × 1000 | requires sorted input |
| `bmb_variance_scaled` | × 1000000 | |
| `bmb_lerp_scaled(a,b,t)` | t ∈ [0,1000] | t=500 → midpoint |
| `bmb_moving_avg_scaled` | × 1000 | sliding window |

### Moving average window

`bmb_moving_avg_scaled` uses a forward sliding window of size `k`:
- `out[0]` = avg(arr[0..k-1]) × 1000
- `out[i]` = avg(arr[i..i+k-1]) × 1000

### Array pointer convention

Pass array pointers cast to `int64_t`:

```c
int64_t data[] = {10, 20, 30};
int64_t s = bmb_sum((int64_t)data, 3); // 60
```

## Function Reference

See `../../include/bmb_compute.h` for all 33 functions.

### Scalar math
`bmb_c_abs`, `bmb_c_min`, `bmb_c_max`, `bmb_c_clamp`, `bmb_sign`,
`bmb_ipow`, `bmb_sqrt`, `bmb_factorial`,
`bmb_c_is_power_of_two`, `bmb_c_next_power_of_two`, `bmb_lerp_scaled`

### Statistics (array)
`bmb_sum`, `bmb_mean_scaled`, `bmb_median_scaled`, `bmb_variance_scaled`,
`bmb_c_min_val`, `bmb_c_max_val`, `bmb_range_val`,
`bmb_magnitude_squared`, `bmb_weighted_sum`

### Vector ops
`bmb_dot_product`, `bmb_dist_squared`,
`bmb_vec_add`, `bmb_vec_sub`, `bmb_vec_scale`, `bmb_map_square`, `bmb_array_copy`

### Prefix / moving
`bmb_cumsum`, `bmb_moving_avg_scaled`

### RNG (LCG-style, stateless)
`bmb_rand_seed`, `bmb_rand_next`, `bmb_rand_pos`, `bmb_rand_range`
