# packages/bmb-io/src/lib.bmb

Auto-generated documentation.

## Table of Contents

- [`read_file`](#read_file)
- [`read_file_result`](#read_file_result)
- [`write_file`](#write_file)
- [`append_file`](#append_file)
- [`file_exists`](#file_exists)
- [`file_size`](#file_size)
- [`IO_SUCCESS`](#IO_SUCCESS)
- [`IO_ERROR_NOT_FOUND`](#IO_ERROR_NOT_FOUND)
- [`IO_ERROR_PERMISSION`](#IO_ERROR_PERMISSION)
- [`IO_ERROR_EXISTS`](#IO_ERROR_EXISTS)
- [`IO_ERROR_INVALID`](#IO_ERROR_INVALID)
- [`IO_ERROR_NO_SPACE`](#IO_ERROR_NO_SPACE)
- [`IO_ERROR_READ_ONLY`](#IO_ERROR_READ_ONLY)
- [`IO_ERROR_UNKNOWN`](#IO_ERROR_UNKNOWN)
- [`is_valid_path`](#is_valid_path)
- [`find_extension`](#find_extension)
- [`find_last_char`](#find_last_char)
- [`find_last_char_from`](#find_last_char_from)

## Functions

### `read_file`

```bmb
pub fn read_file(path: String) -> String
```

---

### `read_file_result`

```bmb
pub fn read_file_result(path: String) -> i64
```

---

### `write_file`

```bmb
pub fn write_file(path: String, content: String) -> i64
```

---

### `append_file`

```bmb
pub fn append_file(path: String, content: String) -> i64
```

---

### `file_exists`

```bmb
pub fn file_exists(path: String) -> i64
```

---

### `file_size`

```bmb
pub fn file_size(path: String) -> i64
```

---

### `IO_SUCCESS`

```bmb
pub fn IO_SUCCESS() -> i64
```

Success

---

### `IO_ERROR_NOT_FOUND`

```bmb
pub fn IO_ERROR_NOT_FOUND() -> i64
```

File not found (ENOENT)

---

### `IO_ERROR_PERMISSION`

```bmb
pub fn IO_ERROR_PERMISSION() -> i64
```

Permission denied (EACCES)

---

### `IO_ERROR_EXISTS`

```bmb
pub fn IO_ERROR_EXISTS() -> i64
```

File already exists (EEXIST)

---

### `IO_ERROR_INVALID`

```bmb
pub fn IO_ERROR_INVALID() -> i64
```

Invalid argument (EINVAL)

---

### `IO_ERROR_NO_SPACE`

```bmb
pub fn IO_ERROR_NO_SPACE() -> i64
```

No space left (ENOSPC)

---

### `IO_ERROR_READ_ONLY`

```bmb
pub fn IO_ERROR_READ_ONLY() -> i64
```

Read-only filesystem (EROFS)

---

### `IO_ERROR_UNKNOWN`

```bmb
pub fn IO_ERROR_UNKNOWN() -> i64
```

Generic/unknown error

---

### `is_valid_path`

```bmb
pub fn is_valid_path(path: String) -> i64
```

Check if path looks valid (basic validation)

---

### `find_extension`

```bmb
pub fn find_extension(path: String) -> i64
```

Find file extension (position of last '.')
Returns: Position of '.', or -1 if no extension

---

### `find_last_char`

```bmb
fn find_last_char(s: String, c: i64) -> i64
```

---

### `find_last_char_from`

```bmb
fn find_last_char_from(s: String, c: i64, pos: i64) -> i64
```

---

