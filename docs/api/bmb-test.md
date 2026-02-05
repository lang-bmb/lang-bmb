# packages/bmb-test/src/lib.bmb

Auto-generated documentation.

## Table of Contents

- [`assert_true`](#assert_true)
- [`assert_false`](#assert_false)
- [`assert_eq_i64`](#assert_eq_i64)
- [`assert_ne_i64`](#assert_ne_i64)
- [`assert_lt_i64`](#assert_lt_i64)
- [`assert_le_i64`](#assert_le_i64)
- [`assert_gt_i64`](#assert_gt_i64)
- [`assert_ge_i64`](#assert_ge_i64)
- [`assert_in_range`](#assert_in_range)
- [`assert_positive`](#assert_positive)
- [`assert_non_negative`](#assert_non_negative)
- [`assert_negative`](#assert_negative)
- [`assert_zero`](#assert_zero)
- [`assert_non_zero`](#assert_non_zero)
- [`assert_eq_bool`](#assert_eq_bool)
- [`assert_truthy`](#assert_truthy)
- [`assert_falsy`](#assert_falsy)
- [`string_eq_local`](#string_eq_local)
- [`string_eq_from`](#string_eq_from)
- [`starts_with_local`](#starts_with_local)
- [`starts_with_check`](#starts_with_check)
- [`ends_with_local`](#ends_with_local)
- [`ends_with_check`](#ends_with_check)
- [`contains_char_local`](#contains_char_local)
- [`contains_char_from`](#contains_char_from)
- [`char_is_whitespace`](#char_is_whitespace)
- [`is_blank_local`](#is_blank_local)
- [`find_trim_start`](#find_trim_start)
- [`find_trim_start_from`](#find_trim_start_from)
- [`assert_string_eq`](#assert_string_eq)
- [`assert_string_ne`](#assert_string_ne)
- [`assert_starts_with`](#assert_starts_with)
- [`assert_ends_with`](#assert_ends_with)
- [`assert_contains_char`](#assert_contains_char)
- [`assert_empty`](#assert_empty)
- [`assert_not_empty`](#assert_not_empty)
- [`assert_blank`](#assert_blank)
- [`assert_not_blank`](#assert_not_blank)
- [`assert_string_len`](#assert_string_len)
- [`contains_i64_local`](#contains_i64_local)
- [`contains_i64_from`](#contains_i64_from)
- [`is_sorted_asc_local`](#is_sorted_asc_local)
- [`is_sorted_asc_from`](#is_sorted_asc_from)
- [`is_sorted_desc_local`](#is_sorted_desc_local)
- [`is_sorted_desc_from`](#is_sorted_desc_from)
- [`all_equal_local`](#all_equal_local)
- [`all_equal_from`](#all_equal_from)
- [`all_positive_local`](#all_positive_local)
- [`all_positive_from`](#all_positive_from)
- [`sum_i64_local`](#sum_i64_local)
- [`sum_i64_from`](#sum_i64_from)
- [`assert_array_contains`](#assert_array_contains)
- [`assert_array_not_contains`](#assert_array_not_contains)
- [`assert_sorted_asc`](#assert_sorted_asc)
- [`assert_sorted_desc`](#assert_sorted_desc)
- [`assert_all_equal`](#assert_all_equal)
- [`assert_all_positive`](#assert_all_positive)
- [`assert_array_sum`](#assert_array_sum)
- [`assert_array_len`](#assert_array_len)
- [`assert_all2`](#assert_all2)
- [`assert_all3`](#assert_all3)
- [`assert_any2`](#assert_any2)
- [`assert_any3`](#assert_any3)
- [`assert_xor`](#assert_xor)
- [`assert_implies`](#assert_implies)
- [`count_passed`](#count_passed)
- [`count_passed_from`](#count_passed_from)
- [`count_failed`](#count_failed)
- [`all_passed`](#all_passed)
- [`any_failed`](#any_failed)

## Functions

### `assert_true`

```bmb
pub fn assert_true(cond: bool) -> bool
```

Assert condition is true

---

### `assert_false`

```bmb
pub fn assert_false(cond: bool) -> bool
```

Assert condition is false

---

### `assert_eq_i64`

```bmb
pub fn assert_eq_i64(actual: i64, expected: i64) -> bool
```

Assert two i64 values are equal

---

### `assert_ne_i64`

```bmb
pub fn assert_ne_i64(actual: i64, expected: i64) -> bool
```

Assert two i64 values are not equal

---

### `assert_lt_i64`

```bmb
pub fn assert_lt_i64(actual: i64, expected: i64) -> bool
```

Assert actual < expected

---

### `assert_le_i64`

```bmb
pub fn assert_le_i64(actual: i64, expected: i64) -> bool
```

Assert actual <= expected

---

### `assert_gt_i64`

```bmb
pub fn assert_gt_i64(actual: i64, expected: i64) -> bool
```

Assert actual > expected

---

### `assert_ge_i64`

```bmb
pub fn assert_ge_i64(actual: i64, expected: i64) -> bool
```

Assert actual >= expected

---

### `assert_in_range`

```bmb
pub fn assert_in_range(val: i64, min: i64, max: i64) -> bool
```

Assert value is in range [min, max]

---

### `assert_positive`

```bmb
pub fn assert_positive(val: i64) -> bool
```

Assert value is positive (> 0)

---

### `assert_non_negative`

```bmb
pub fn assert_non_negative(val: i64) -> bool
```

Assert value is non-negative (>= 0)

---

### `assert_negative`

```bmb
pub fn assert_negative(val: i64) -> bool
```

Assert value is negative (< 0)

---

### `assert_zero`

```bmb
pub fn assert_zero(val: i64) -> bool
```

Assert value is zero

---

### `assert_non_zero`

```bmb
pub fn assert_non_zero(val: i64) -> bool
```

Assert value is non-zero

---

### `assert_eq_bool`

```bmb
pub fn assert_eq_bool(actual: bool, expected: bool) -> bool
```

Assert two booleans are equal

---

### `assert_truthy`

```bmb
pub fn assert_truthy(val: bool) -> bool
```

Assert boolean is truthy

---

### `assert_falsy`

```bmb
pub fn assert_falsy(val: bool) -> bool
```

Assert boolean is falsy

---

### `string_eq_local`

```bmb
fn string_eq_local(a: String, b: String) -> bool
```

---

### `string_eq_from`

```bmb
fn string_eq_from(a: String, b: String, pos: i64) -> bool
```

---

### `starts_with_local`

```bmb
fn starts_with_local(s: String, prefix: String) -> bool
```

---

### `starts_with_check`

```bmb
fn starts_with_check(s: String, prefix: String, pos: i64) -> bool
```

---

### `ends_with_local`

```bmb
fn ends_with_local(s: String, suffix: String) -> bool
```

---

### `ends_with_check`

```bmb
fn ends_with_check(s: String, suffix: String, pos: i64) -> bool
```

---

### `contains_char_local`

```bmb
fn contains_char_local(s: String, c: i64) -> bool
```

---

### `contains_char_from`

```bmb
fn contains_char_from(s: String, c: i64, pos: i64) -> bool
```

---

### `char_is_whitespace`

```bmb
fn char_is_whitespace(c: i64) -> bool
```

---

### `is_blank_local`

```bmb
fn is_blank_local(s: String) -> bool
```

---

### `find_trim_start`

```bmb
fn find_trim_start(s: String) -> i64
```

---

### `find_trim_start_from`

```bmb
fn find_trim_start_from(s: String, pos: i64) -> i64
```

---

### `assert_string_eq`

```bmb
pub fn assert_string_eq(actual: String, expected: String) -> bool
```

Assert two strings are equal

---

### `assert_string_ne`

```bmb
pub fn assert_string_ne(actual: String, expected: String) -> bool
```

Assert two strings are not equal

---

### `assert_starts_with`

```bmb
pub fn assert_starts_with(s: String, prefix: String) -> bool
```

Assert string starts with prefix

---

### `assert_ends_with`

```bmb
pub fn assert_ends_with(s: String, suffix: String) -> bool
```

Assert string ends with suffix

---

### `assert_contains_char`

```bmb
pub fn assert_contains_char(s: String, c: i64) -> bool
```

Assert string contains character

---

### `assert_empty`

```bmb
pub fn assert_empty(s: String) -> bool
```

Assert string is empty

---

### `assert_not_empty`

```bmb
pub fn assert_not_empty(s: String) -> bool
```

Assert string is not empty

---

### `assert_blank`

```bmb
pub fn assert_blank(s: String) -> bool
```

Assert string is blank (empty or whitespace only)

---

### `assert_not_blank`

```bmb
pub fn assert_not_blank(s: String) -> bool
```

Assert string is not blank

---

### `assert_string_len`

```bmb
pub fn assert_string_len(s: String, expected_len: i64) -> bool
```

Assert string has expected length

---

### `contains_i64_local`

```bmb
fn contains_i64_local(arr: [i64; 8], len: i64, val: i64) -> bool
```

---

### `contains_i64_from`

```bmb
fn contains_i64_from(arr: [i64; 8], len: i64, val: i64, pos: i64) -> bool
```

---

### `is_sorted_asc_local`

```bmb
fn is_sorted_asc_local(arr: [i64; 8], len: i64) -> bool
```

---

### `is_sorted_asc_from`

```bmb
fn is_sorted_asc_from(arr: [i64; 8], len: i64, pos: i64) -> bool
```

---

### `is_sorted_desc_local`

```bmb
fn is_sorted_desc_local(arr: [i64; 8], len: i64) -> bool
```

---

### `is_sorted_desc_from`

```bmb
fn is_sorted_desc_from(arr: [i64; 8], len: i64, pos: i64) -> bool
```

---

### `all_equal_local`

```bmb
fn all_equal_local(arr: [i64; 8], len: i64) -> bool
```

---

### `all_equal_from`

```bmb
fn all_equal_from(arr: [i64; 8], len: i64, val: i64, pos: i64) -> bool
```

---

### `all_positive_local`

```bmb
fn all_positive_local(arr: [i64; 8], len: i64) -> bool
```

---

### `all_positive_from`

```bmb
fn all_positive_from(arr: [i64; 8], len: i64, pos: i64) -> bool
```

---

### `sum_i64_local`

```bmb
fn sum_i64_local(arr: [i64; 8], len: i64) -> i64
```

---

### `sum_i64_from`

```bmb
fn sum_i64_from(arr: [i64; 8], len: i64, pos: i64) -> i64
```

---

### `assert_array_contains`

```bmb
pub fn assert_array_contains(arr: [i64; 8], len: i64, val: i64) -> bool
```

Assert array contains value

---

### `assert_array_not_contains`

```bmb
pub fn assert_array_not_contains(arr: [i64; 8], len: i64, val: i64) -> bool
```

Assert array does not contain value

---

### `assert_sorted_asc`

```bmb
pub fn assert_sorted_asc(arr: [i64; 8], len: i64) -> bool
```

Assert array is sorted ascending

---

### `assert_sorted_desc`

```bmb
pub fn assert_sorted_desc(arr: [i64; 8], len: i64) -> bool
```

Assert array is sorted descending

---

### `assert_all_equal`

```bmb
pub fn assert_all_equal(arr: [i64; 8], len: i64) -> bool
```

Assert all array elements are equal

---

### `assert_all_positive`

```bmb
pub fn assert_all_positive(arr: [i64; 8], len: i64) -> bool
```

Assert all array elements are positive

---

### `assert_array_sum`

```bmb
pub fn assert_array_sum(arr: [i64; 8], len: i64, expected: i64) -> bool
```

Assert array sum equals expected

---

### `assert_array_len`

```bmb
pub fn assert_array_len(len: i64, expected: i64) -> bool
```

Assert array length (logical length, not fixed size)

---

### `assert_all2`

```bmb
pub fn assert_all2(a: bool, b: bool) -> bool
```

Assert all conditions are true (and2)

---

### `assert_all3`

```bmb
pub fn assert_all3(a: bool, b: bool, c: bool) -> bool
```

Assert all conditions are true (and3)

---

### `assert_any2`

```bmb
pub fn assert_any2(a: bool, b: bool) -> bool
```

Assert at least one condition is true (or2)

---

### `assert_any3`

```bmb
pub fn assert_any3(a: bool, b: bool, c: bool) -> bool
```

Assert at least one condition is true (or3)

---

### `assert_xor`

```bmb
pub fn assert_xor(a: bool, b: bool) -> bool
```

Assert exactly one of two conditions is true

---

### `assert_implies`

```bmb
pub fn assert_implies(a: bool, b: bool) -> bool
```

Assert implication: if a then b

---

### `count_passed`

```bmb
pub fn count_passed(results: [i64; 8], len: i64) -> i64
```

Count passing tests from array of results

---

### `count_passed_from`

```bmb
fn count_passed_from(results: [i64; 8], len: i64, pos: i64) -> i64
```

---

### `count_failed`

```bmb
pub fn count_failed(results: [i64; 8], len: i64) -> i64
```

Count failing tests from array of results

---

### `all_passed`

```bmb
pub fn all_passed(results: [i64; 8], len: i64) -> bool
```

Check if all tests passed

---

### `any_failed`

```bmb
pub fn any_failed(results: [i64; 8], len: i64) -> bool
```

Check if any test failed

---

