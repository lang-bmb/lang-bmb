# packages/bmb-string/src/lib.bmb

Auto-generated documentation.

## Table of Contents

- [`char_is_whitespace`](#char_is_whitespace)
- [`char_is_digit`](#char_is_digit)
- [`char_is_lower`](#char_is_lower)
- [`char_is_upper`](#char_is_upper)
- [`char_is_alpha`](#char_is_alpha)
- [`char_is_alnum`](#char_is_alnum)
- [`char_to_upper`](#char_to_upper)
- [`char_to_lower`](#char_to_lower)
- [`digit_to_int`](#digit_to_int)
- [`int_to_digit`](#int_to_digit)
- [`contains_char`](#contains_char)
- [`contains_char_from`](#contains_char_from)
- [`starts_with`](#starts_with)
- [`starts_with_check`](#starts_with_check)
- [`ends_with`](#ends_with)
- [`ends_with_check`](#ends_with_check)
- [`index_of_char`](#index_of_char)
- [`index_of_char_from`](#index_of_char_from)
- [`count_char`](#count_char)
- [`count_char_from`](#count_char_from)
- [`find_trim_start`](#find_trim_start)
- [`find_trim_start_from`](#find_trim_start_from)
- [`find_trim_end`](#find_trim_end)
- [`find_trim_end_from`](#find_trim_end_from)
- [`is_blank`](#is_blank)
- [`parse_uint`](#parse_uint)
- [`parse_uint_acc`](#parse_uint_acc)
- [`parse_int`](#parse_int)
- [`is_valid_int`](#is_valid_int)
- [`all_digits`](#all_digits)
- [`string_compare`](#string_compare)
- [`string_compare_from`](#string_compare_from)
- [`string_eq`](#string_eq)
- [`string_eq_from`](#string_eq_from)
- [`digit_char`](#digit_char)
- [`int_to_string`](#int_to_string)
- [`split_first_len`](#split_first_len)
- [`char_count`](#char_count)

## Functions

### `char_is_whitespace`

```bmb
pub fn char_is_whitespace(c: i64) -> bool
```

Check if character code is whitespace (space, tab, newline, CR)

---

### `char_is_digit`

```bmb
pub fn char_is_digit(c: i64) -> bool
```

Check if character code is a digit (0-9)

---

### `char_is_lower`

```bmb
pub fn char_is_lower(c: i64) -> bool
```

Check if character code is lowercase letter (a-z)

---

### `char_is_upper`

```bmb
pub fn char_is_upper(c: i64) -> bool
```

Check if character code is uppercase letter (A-Z)

---

### `char_is_alpha`

```bmb
pub fn char_is_alpha(c: i64) -> bool
```

Check if character code is a letter (a-z or A-Z)

---

### `char_is_alnum`

```bmb
pub fn char_is_alnum(c: i64) -> bool
```

Check if character code is alphanumeric

---

### `char_to_upper`

```bmb
pub fn char_to_upper(c: i64) -> i64
```

Convert lowercase to uppercase

---

### `char_to_lower`

```bmb
pub fn char_to_lower(c: i64) -> i64
```

Convert uppercase to lowercase

---

### `digit_to_int`

```bmb
pub fn digit_to_int(c: i64) -> i64
```

Convert digit character to integer (0-9)

---

### `int_to_digit`

```bmb
pub fn int_to_digit(n: i64) -> i64
```

Convert integer (0-9) to digit character

---

### `contains_char`

```bmb
pub fn contains_char(s: String, c: i64) -> bool
```

Check if string contains character at any position

---

### `contains_char_from`

```bmb
fn contains_char_from(s: String, c: i64, pos: i64) -> bool
```

---

### `starts_with`

```bmb
pub fn starts_with(s: String, prefix: String) -> bool
```

Check if string starts with given prefix

---

### `starts_with_check`

```bmb
fn starts_with_check(s: String, prefix: String, pos: i64) -> bool
```

---

### `ends_with`

```bmb
pub fn ends_with(s: String, suffix: String) -> bool
```

Check if string ends with given suffix

---

### `ends_with_check`

```bmb
fn ends_with_check(s: String, suffix: String, pos: i64) -> bool
```

---

### `index_of_char`

```bmb
pub fn index_of_char(s: String, c: i64) -> i64
```

Find first occurrence of character, returns -1 if not found

---

### `index_of_char_from`

```bmb
fn index_of_char_from(s: String, c: i64, pos: i64) -> i64
```

---

### `count_char`

```bmb
pub fn count_char(s: String, c: i64) -> i64
```

Count occurrences of character

---

### `count_char_from`

```bmb
fn count_char_from(s: String, c: i64, pos: i64) -> i64
```

---

### `find_trim_start`

```bmb
pub fn find_trim_start(s: String) -> i64
```

Find first non-whitespace position

---

### `find_trim_start_from`

```bmb
fn find_trim_start_from(s: String, pos: i64) -> i64
```

---

### `find_trim_end`

```bmb
pub fn find_trim_end(s: String) -> i64
```

Find position after last non-whitespace

---

### `find_trim_end_from`

```bmb
fn find_trim_end_from(s: String, pos: i64) -> i64
```

---

### `is_blank`

```bmb
pub fn is_blank(s: String) -> bool
```

Check if string is empty or only whitespace

---

### `parse_uint`

```bmb
pub fn parse_uint(s: String) -> i64
```

Parse non-negative integer from string (returns -1 on error)

---

### `parse_uint_acc`

```bmb
fn parse_uint_acc(s: String, pos: i64, acc: i64) -> i64
```

---

### `parse_int`

```bmb
pub fn parse_int(s: String) -> i64
```

Parse signed integer from string

---

### `is_valid_int`

```bmb
pub fn is_valid_int(s: String) -> bool
```

Check if string represents a valid integer

---

### `all_digits`

```bmb
fn all_digits(s: String, pos: i64) -> bool
```

---

### `string_compare`

```bmb
pub fn string_compare(a: String, b: String) -> i64
```

Compare two strings lexicographically (-1, 0, 1)

---

### `string_compare_from`

```bmb
fn string_compare_from(a: String, b: String, pos: i64) -> i64
```

---

### `string_eq`

```bmb
pub fn string_eq(a: String, b: String) -> bool
```

Check string equality

---

### `string_eq_from`

```bmb
fn string_eq_from(a: String, b: String, pos: i64) -> bool
```

---

### `digit_char`

```bmb
pub fn digit_char(d: i64) -> String
```

Convert single digit (0-9) to string

---

### `int_to_string`

```bmb
pub fn int_to_string(n: i64) -> String
```

Convert integer to string representation

---

### `split_first_len`

```bmb
pub fn split_first_len(s: String, delim: i64) -> i64
```

Get substring length for split by character

---

### `char_count`

```bmb
pub fn char_count(s: String) -> i64
```

Get character count (same as len but with explicit name)

---

