# Error Handling in BMB

BMB uses contracts and return values for error handling. There are no exceptions.

## Contracts: Preventing Errors

The primary error handling mechanism is **prevention** via `pre`/`post` conditions:

```bmb
fn divide(a: i64, b: i64) -> i64
    pre b != 0
    post ret * b <= a and ret * b > a - b
= a / b;
```

The `pre` condition makes division-by-zero **impossible** at the call site. The compiler verifies this at compile time — no runtime check needed.

## Return Code Pattern

For operations that can fail, return an error code:

```bmb
// 0 = success, negative = error
fn write_data(path: String, data: String) -> i64
    pre path.len() > 0
    post ret <= 0
= write_file(path, data);

fn main() -> i64 = {
    let result = write_data("output.txt", "hello");
    if result < 0 {
        println(0 - 1);   // error
        1
    } else {
        println(0);        // success
        0
    }
};
```

## Nullable Types (`T?`)

For values that may or may not exist:

```bmb
fn find_index(arr: &[i64; 10], target: i64) -> i64
    post ret >= -1 and ret < 10
= find_index_from(arr, target, 0);

fn find_index_from(arr: &[i64; 10], target: i64, i: i64) -> i64 =
    if i >= 10 { -1 }
    else if arr[i] == target { i }
    else { find_index_from(arr, target, i + 1) };
```

## Option Pattern (stdlib)

The `core::option` module provides structured optional values:

```bmb
use core::option::some;
use core::option::none;
use core::option::is_some;
use core::option::unwrap_or;

fn safe_divide(a: i64, b: i64) -> i64 =
    if b == 0 { none() }
    else { some(a / b) };

fn main() -> i64 = {
    let result = safe_divide(10, 3);
    let value = unwrap_or(result, 0);
    println(value);   // 3
    0
};
```

## Result Pattern (stdlib)

The `core::result` module provides success/error discrimination:

```bmb
use core::result::ok;
use core::result::err;
use core::result::is_ok;
use core::result::unwrap;
```

## Best Practices

1. **Use contracts first** — prevent errors instead of handling them
2. **Return error codes** for I/O and external operations
3. **Use Option/Result** for domain logic that can fail
4. **Document failure modes** in `post` conditions

### Contract as documentation:

```bmb
fn binary_search(arr: &[i64; 100], target: i64, lo: i64, hi: i64) -> i64
    pre lo >= 0 and hi < 100
    post ret >= -1 and ret < 100
= ...;
```

The contracts tell the caller exactly what inputs are valid and what outputs to expect. No separate error handling documentation needed.
