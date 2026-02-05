# packages/bmb-array/src/lib.bmb

Auto-generated documentation.

## Table of Contents

- [`contains_i64`](#contains_i64)
- [`contains_i64_from`](#contains_i64_from)
- [`index_of_i64`](#index_of_i64)
- [`index_of_i64_from`](#index_of_i64_from)
- [`count_i64`](#count_i64)
- [`count_i64_from`](#count_i64_from)
- [`sum_i64`](#sum_i64)
- [`sum_i64_from`](#sum_i64_from)
- [`min_i64`](#min_i64)
- [`min_i64_from`](#min_i64_from)
- [`max_i64`](#max_i64)
- [`max_i64_from`](#max_i64_from)
- [`avg_i64`](#avg_i64)
- [`product_i64`](#product_i64)
- [`product_i64_from`](#product_i64_from)
- [`all_positive`](#all_positive)
- [`all_positive_from`](#all_positive_from)
- [`all_non_negative`](#all_non_negative)
- [`all_non_negative_from`](#all_non_negative_from)
- [`any_positive`](#any_positive)
- [`any_positive_from`](#any_positive_from)
- [`any_zero`](#any_zero)
- [`is_sorted_asc`](#is_sorted_asc)
- [`is_sorted_asc_from`](#is_sorted_asc_from)
- [`is_sorted_desc`](#is_sorted_desc)
- [`is_sorted_desc_from`](#is_sorted_desc_from)
- [`all_equal`](#all_equal)
- [`all_equal_from`](#all_equal_from)
- [`is_valid_index`](#is_valid_index)
- [`clamp_index`](#clamp_index)
- [`wrap_index`](#wrap_index)
- [`sum_range`](#sum_range)
- [`sum_range_from`](#sum_range_from)
- [`count_range`](#count_range)
- [`count_range_from`](#count_range_from)

## Functions

### `contains_i64`

```bmb
pub fn contains_i64(arr: [i64; 8], len: i64, val: i64) -> bool
```

Check if array contains value

---

### `contains_i64_from`

```bmb
fn contains_i64_from(arr: [i64; 8], len: i64, val: i64, pos: i64) -> bool
```

---

### `index_of_i64`

```bmb
pub fn index_of_i64(arr: [i64; 8], len: i64, val: i64) -> i64
```

Find index of value, returns -1 if not found

---

### `index_of_i64_from`

```bmb
fn index_of_i64_from(arr: [i64; 8], len: i64, val: i64, pos: i64) -> i64
```

---

### `count_i64`

```bmb
pub fn count_i64(arr: [i64; 8], len: i64, val: i64) -> i64
```

Count occurrences of value

---

### `count_i64_from`

```bmb
fn count_i64_from(arr: [i64; 8], len: i64, val: i64, pos: i64) -> i64
```

---

### `sum_i64`

```bmb
pub fn sum_i64(arr: [i64; 8], len: i64) -> i64
```

Sum all elements

---

### `sum_i64_from`

```bmb
fn sum_i64_from(arr: [i64; 8], len: i64, pos: i64) -> i64
```

---

### `min_i64`

```bmb
pub fn min_i64(arr: [i64; 8], len: i64) -> i64
```

Find minimum value (requires non-empty)

---

### `min_i64_from`

```bmb
fn min_i64_from(arr: [i64; 8], len: i64, current_min: i64, pos: i64) -> i64
```

---

### `max_i64`

```bmb
pub fn max_i64(arr: [i64; 8], len: i64) -> i64
```

Find maximum value (requires non-empty)

---

### `max_i64_from`

```bmb
fn max_i64_from(arr: [i64; 8], len: i64, current_max: i64, pos: i64) -> i64
```

---

### `avg_i64`

```bmb
pub fn avg_i64(arr: [i64; 8], len: i64) -> i64
```

Calculate average (integer division)

---

### `product_i64`

```bmb
pub fn product_i64(arr: [i64; 8], len: i64) -> i64
```

Product of all elements

---

### `product_i64_from`

```bmb
fn product_i64_from(arr: [i64; 8], len: i64, pos: i64) -> i64
```

---

### `all_positive`

```bmb
pub fn all_positive(arr: [i64; 8], len: i64) -> bool
```

Check if all elements are positive

---

### `all_positive_from`

```bmb
fn all_positive_from(arr: [i64; 8], len: i64, pos: i64) -> bool
```

---

### `all_non_negative`

```bmb
pub fn all_non_negative(arr: [i64; 8], len: i64) -> bool
```

Check if all elements are non-negative

---

### `all_non_negative_from`

```bmb
fn all_non_negative_from(arr: [i64; 8], len: i64, pos: i64) -> bool
```

---

### `any_positive`

```bmb
pub fn any_positive(arr: [i64; 8], len: i64) -> bool
```

Check if any element is positive

---

### `any_positive_from`

```bmb
fn any_positive_from(arr: [i64; 8], len: i64, pos: i64) -> bool
```

---

### `any_zero`

```bmb
pub fn any_zero(arr: [i64; 8], len: i64) -> bool
```

Check if any element is zero

---

### `is_sorted_asc`

```bmb
pub fn is_sorted_asc(arr: [i64; 8], len: i64) -> bool
```

Check if array is sorted ascending

---

### `is_sorted_asc_from`

```bmb
fn is_sorted_asc_from(arr: [i64; 8], len: i64, pos: i64) -> bool
```

---

### `is_sorted_desc`

```bmb
pub fn is_sorted_desc(arr: [i64; 8], len: i64) -> bool
```

Check if array is sorted descending

---

### `is_sorted_desc_from`

```bmb
fn is_sorted_desc_from(arr: [i64; 8], len: i64, pos: i64) -> bool
```

---

### `all_equal`

```bmb
pub fn all_equal(arr: [i64; 8], len: i64) -> bool
```

Check if all elements are equal

---

### `all_equal_from`

```bmb
fn all_equal_from(arr: [i64; 8], len: i64, val: i64, pos: i64) -> bool
```

---

### `is_valid_index`

```bmb
pub fn is_valid_index(len: i64, idx: i64) -> bool
```

Check if index is valid

---

### `clamp_index`

```bmb
pub fn clamp_index(len: i64, idx: i64) -> i64
```

Clamp index to valid range

---

### `wrap_index`

```bmb
pub fn wrap_index(len: i64, idx: i64) -> i64
```

Wrap index (modulo behavior)

---

### `sum_range`

```bmb
pub fn sum_range(arr: [i64; 8], start: i64, end: i64) -> i64
```

Sum elements in range [start, end)

---

### `sum_range_from`

```bmb
fn sum_range_from(arr: [i64; 8], pos: i64, end: i64) -> i64
```

---

### `count_range`

```bmb
pub fn count_range(arr: [i64; 8], start: i64, end: i64, val: i64) -> i64
```

Count elements in range [start, end) matching value

---

### `count_range_from`

```bmb
fn count_range_from(arr: [i64; 8], pos: i64, end: i64, val: i64) -> i64
```

---

