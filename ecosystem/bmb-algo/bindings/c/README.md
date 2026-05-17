# bmb-algo — C Bindings

C bindings for the bmb-algo library via direct shared library linking.

## Requirements

- GCC or Clang (C99+)
- `bmb_algo.dll` / `libbmb_algo.so` / `libbmb_algo.dylib` in the parent directory (`../../`)

## Build & Run

```bash
# Build example and test programs
make

# Run the example
make run-example

# Run the test suite
make run-test
```

### Manual build

```bash
# Windows
gcc -O2 -I../../include -o example example.c -L../.. -l:bmb_algo.dll

# Linux
gcc -O2 -I../../include -o example example.c -L../.. -lbmb_algo -Wl,-rpath,../..

# macOS
gcc -O2 -I../../include -o example example.c -L../.. -lbmb_algo -Wl,-rpath,../..
```

## Usage

```c
#include "bmb_algo.h"
#include <stdio.h>

int main(void) {
    bmb_ffi_begin();

    // Scalar functions: direct call
    int64_t f10 = bmb_fibonacci(10);         // 55
    int64_t g   = bmb_gcd(48, 18);           // 6

    // Array functions: cast pointer to int64_t
    int64_t arr[] = {5, 3, 8, 1, 9};
    bmb_quicksort((int64_t)arr, 5);          // sorts in-place

    // String functions: use BMB string API
    void *s1 = bmb_ffi_cstr_to_string("ABCBDAB");
    void *s2 = bmb_ffi_cstr_to_string("BDCAB");
    int64_t lcs = bmb_lcs(s1, s2);          // 4
    bmb_ffi_free_string(s1);
    bmb_ffi_free_string(s2);

    // Check for errors
    if (bmb_ffi_has_error()) {
        fprintf(stderr, "Error: %s\n", bmb_ffi_error_message());
        bmb_ffi_end();
        return 1;
    }

    bmb_ffi_end();
    printf("fib(10)=%lld  gcd=%lld  lcs=%lld\n",
           (long long)f10, (long long)g, (long long)lcs);
    return 0;
}
```

## API Conventions

### FFI safety

All calls must be wrapped with `bmb_ffi_begin()` / `bmb_ffi_end()`:

```c
bmb_ffi_begin();
// ... calls ...
if (bmb_ffi_has_error()) { /* handle */ }
bmb_ffi_end();
```

### Array arguments

Functions that operate on arrays accept `int64_t arr` where `arr` is a
`int64_t *` cast to `int64_t`. The array must be caller-allocated and
contain `int64_t` elements:

```c
int64_t data[] = {5, 3, 1, 4, 2};
bmb_quicksort((int64_t)data, 5);
```

### String arguments

Functions that take or return BMB strings use opaque `void *` handles.
Always free strings obtained from string functions:

```c
void *s = bmb_ffi_cstr_to_string("hello");
// use s ...
bmb_ffi_free_string(s);
```

## Function Reference

See `../../include/bmb_algo.h` for the full list of 55 functions.

### Math & Number Theory
`bmb_gcd`, `bmb_lcm`, `bmb_fibonacci`, `bmb_prime_count`, `bmb_modpow`,
`bmb_nqueens`, `bmb_power_set_size`, `bmb_algo_is_prime`, `bmb_digit_sum`,
`bmb_is_palindrome_num`

### Sorting
`bmb_quicksort`, `bmb_merge_sort`, `bmb_heap_sort`, `bmb_counting_sort`,
`bmb_insertion_sort`, `bmb_selection_sort`, `bmb_bubble_sort`, `bmb_shell_sort`

### Search & DP
`bmb_binary_search`, `bmb_kth_smallest`, `bmb_two_sum`, `bmb_subset_sum`,
`bmb_coin_change`, `bmb_lis`, `bmb_max_subarray`, `bmb_sorted_intersect_count`,
`bmb_array_mode`

### Array Utilities
`bmb_array_sum`, `bmb_array_min`, `bmb_array_max`, `bmb_array_contains`,
`bmb_array_index_of`, `bmb_unique_count`, `bmb_array_fill`, `bmb_array_reverse`,
`bmb_array_rotate`, `bmb_is_sorted`, `bmb_prefix_sum`, `bmb_array_product`

### Bit Operations
`bmb_bit_popcount`, `bmb_bit_set`, `bmb_bit_clear`, `bmb_bit_test`, `bmb_bit_toggle`

### Matrix
`bmb_matrix_multiply`, `bmb_matrix_transpose`, `bmb_matrix_det`

### Graph
`bmb_dijkstra`, `bmb_bfs_count`, `bmb_floyd_warshall`, `bmb_topological_sort`,
`bmb_knapsack`

### Strings
`bmb_lcs`, `bmb_edit_distance`, `bmb_djb2_hash`
