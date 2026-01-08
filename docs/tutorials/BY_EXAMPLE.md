# BMB By Example

> Learn BMB through practical, runnable examples.

## Table of Contents

1. [Hello World](#hello-world)
2. [Variables and Types](#variables-and-types)
3. [Functions](#functions)
4. [Control Flow](#control-flow)
5. [Collections](#collections)
6. [Contracts](#contracts)
7. [Structs and Enums](#structs-and-enums)
8. [Pattern Matching](#pattern-matching)
9. [Error Handling](#error-handling)
10. [Closures](#closures)
11. [File I/O](#file-io)
12. [Testing](#testing)

---

## Hello World

```bmb
-- The simplest BMB program
fn main() -> i64 =
    println("Hello, World!");
    0;
```

```bash
$ bmb run hello.bmb
Hello, World!
```

---

## Variables and Types

### Immutable Variables

```bmb
fn main() -> i64 =
    let x = 42;          -- i64 inferred
    let name = "BMB";    -- String inferred
    let flag = true;     -- bool inferred

    println(int_to_string(x));
    0;
```

### Mutable Variables

```bmb
fn main() -> i64 =
    var counter = 0;
    counter = counter + 1;
    counter = counter + 1;
    println(int_to_string(counter));  -- prints "2"
    0;
```

### Type Annotations

```bmb
fn main() -> i64 =
    let x: i64 = 42;
    let y: bool = true;
    let arr: [i64; 4] = [1, 2, 3, 4];
    0;
```

---

## Functions

### Basic Functions

```bmb
-- Single expression body
fn square(x: i64) -> i64 = x * x;

-- Multi-expression body
fn area_of_rectangle(width: i64, height: i64) -> i64 =
    let result = width * height;
    result;
```

### Multiple Return Values (via Tuples)

```bmb
fn divmod(a: i64, b: i64) -> (i64, i64) =
    (a / b, a % b);

fn main() -> i64 =
    let (quotient, remainder) = divmod(17, 5);
    println(int_to_string(quotient));   -- 3
    println(int_to_string(remainder));  -- 2
    0;
```

### Recursive Functions

```bmb
fn factorial(n: i64) -> i64 =
    if n <= 1 then 1
    else n * factorial(n - 1);

fn fibonacci(n: i64) -> i64 =
    if n <= 1 then n
    else fibonacci(n - 1) + fibonacci(n - 2);
```

---

## Control Flow

### If Expressions

```bmb
-- If is an expression, not a statement
fn max(a: i64, b: i64) -> i64 =
    if a > b then a else b;

fn sign(n: i64) -> String =
    if n > 0 then "positive"
    else if n < 0 then "negative"
    else "zero";
```

### Nested Conditionals

```bmb
fn grade(score: i64) -> String =
    if score >= 90 then "A"
    else if score >= 80 then "B"
    else if score >= 70 then "C"
    else if score >= 60 then "D"
    else "F";
```

---

## Collections

### Arrays (Fixed Size)

```bmb
fn main() -> i64 =
    let arr = [1, 2, 3, 4, 5];
    let first = arr[0];
    let sum = arr[0] + arr[1] + arr[2];

    -- Using stdlib
    use array::sum_i64;
    let total = sum_i64(arr, 5);
    0;
```

### Array Operations

```bmb
use array::sum_i64;
use array::min_i64;
use array::max_i64;
use array::is_sorted_asc;

fn analyze_data(data: [i64; 8], len: i64) -> i64 =
    let total = sum_i64(data, len);
    let minimum = min_i64(data, len);
    let maximum = max_i64(data, len);
    let sorted = is_sorted_asc(data, len);

    if sorted then total else 0 - 1;
```

---

## Contracts

### Preconditions

```bmb
-- Compiler ensures divisor is never zero
fn safe_divide(a: i64, b: i64) -> i64
  pre b != 0
= a / b;

-- Multiple preconditions
fn array_access(arr: [i64; 8], index: i64) -> i64
  pre index >= 0
  pre index < 8
= arr[index];
```

### Postconditions

```bmb
-- Compiler verifies return value is always non-negative
fn absolute(x: i64) -> i64
  post ret >= 0
= if x >= 0 then x else 0 - x;

-- Complex postcondition
fn clamp(x: i64, lo: i64, hi: i64) -> i64
  pre lo <= hi
  post ret >= lo
  post ret <= hi
= if x < lo then lo
  else if x > hi then hi
  else x;
```

### Inline Refinement Types

```bmb
-- Modern syntax: constraints in type position
fn sqrt_checked(x: i64{it >= 0}) -> i64{it >= 0} =
    -- implementation for non-negative input
    -- compiler knows x >= 0 and result >= 0
    -- ...
```

### Named Contracts

```bmb
fn binary_search(arr: [i64; 8], len: i64, target: i64) -> i64
  where {
    valid_length: len > 0 and len <= 8,
    array_sorted: is_sorted_asc(arr, len)
  }
  post search_result_valid: (ret == -1) or (ret >= 0 and ret < len)
= -- implementation
```

---

## Structs and Enums

### Structs

```bmb
struct Point {
    x: i64,
    y: i64
}

fn distance_squared(p1: Point, p2: Point) -> i64 =
    let dx = p2.x - p1.x;
    let dy = p2.y - p1.y;
    dx * dx + dy * dy;

fn main() -> i64 =
    let origin = Point { x: 0, y: 0 };
    let p = Point { x: 3, y: 4 };
    distance_squared(origin, p);  -- returns 25
```

### Enums

```bmb
enum Color {
    Red,
    Green,
    Blue,
    RGB(i64, i64, i64)
}

fn to_hex(c: Color) -> i64 =
    match c {
        Color::Red => 16711680,     -- 0xFF0000
        Color::Green => 65280,      -- 0x00FF00
        Color::Blue => 255,         -- 0x0000FF
        Color::RGB(r, g, b) => r * 65536 + g * 256 + b
    };
```

---

## Pattern Matching

### Basic Matching

```bmb
fn describe_number(n: i64) -> String =
    match n {
        0 => "zero",
        1 => "one",
        2 => "two",
        _ => "many"
    };
```

### Enum Matching

```bmb
enum Option {
    Some(i64),
    None
}

fn unwrap_or(opt: Option, default: i64) -> i64 =
    match opt {
        Option::Some(value) => value,
        Option::None => default
    };
```

### Guard Patterns

```bmb
fn classify(n: i64) -> String =
    match n {
        x if x < 0 => "negative",
        0 => "zero",
        x if x < 10 => "small positive",
        _ => "large positive"
    };
```

---

## Error Handling

### Result Type

```bmb
use core::result::Result;
use core::result::ok;
use core::result::err;

fn parse_positive(s: String) -> Result =
    let n = parse_int(s);
    if n > 0 then ok(n)
    else err(-1);

fn main() -> i64 =
    let result = parse_positive("42");
    match result {
        Result::Ok(n) => n,
        Result::Err(code) => 0
    };
```

### Safe Operations with Contracts

```bmb
use core::result::safe_divide;

fn compute_ratio(a: i64, b: i64) -> i64 =
    let result = safe_divide(a, b);
    match result {
        Result::Ok(value) => value,
        Result::Err(_) => 0  -- Division by zero handled
    };
```

---

## Closures

### Lambda Syntax

```bmb
fn main() -> i64 =
    -- Basic closure
    let add_one = fn |x: i64| { x + 1 };
    let result = add_one(5);  -- 6

    -- Two parameters
    let add = fn |a: i64, b: i64| { a + b };
    add(3, 4);  -- 7
```

### Closures as Parameters

```bmb
fn apply(f: fn(i64) -> i64, x: i64) -> i64 = f(x);

fn main() -> i64 =
    let double = fn |n: i64| { n * 2 };
    apply(double, 21);  -- 42
```

---

## File I/O

### Reading Files

```bmb
use io::read_file;
use io::IO_SUCCESS;

fn main() -> i64 =
    let content = read_file("input.txt");
    println(content);
    0;
```

### Writing Files

```bmb
use io::write_file;
use io::append_file;

fn main() -> i64 =
    let result = write_file("output.txt", "Hello, BMB!");
    if result == 0 then
        append_file("output.txt", "\nAppended line")
    else
        result;
```

### Error Handling

```bmb
use io::read_file_result;
use io::IO_ERROR_NOT_FOUND;

fn safe_read(path: String) -> String =
    let result = read_file_result(path);
    if result == 0 then read_file(path)
    else "";
```

---

## Testing

### Test Functions

```bmb
use test::assert_eq_i64;
use test::assert_true;

fn test_addition() -> bool =
    assert_eq_i64(1 + 2, 3);

fn test_max() -> bool =
    let result = max(10, 20);
    assert_eq_i64(result, 20);

fn test_contracts() -> bool =
    let clamped = clamp(15, 0, 10);
    assert_true(clamped >= 0) and
    assert_true(clamped <= 10);
```

### Running Tests

```bash
$ bmb test myprogram.bmb
Running 3 tests...
  test_addition ... OK
  test_max ... OK
  test_contracts ... OK

All tests passed!
```

### Test Assertions

```bmb
use test::assert_string_eq;
use test::assert_in_range;
use test::assert_positive;

fn test_string_operations() -> bool =
    assert_string_eq("hello", "hello");

fn test_range() -> bool =
    let x = compute_value();
    assert_in_range(x, 0, 100);

fn test_result() -> bool =
    let n = abs(-42);
    assert_positive(n);
```

---

## Complete Example: FizzBuzz with Contracts

```bmb
-- FizzBuzz with contract verification
fn fizzbuzz(n: i64{it > 0}) -> String
  pre n > 0
  post ret.len() > 0
=
    if n % 15 == 0 then "FizzBuzz"
    else if n % 3 == 0 then "Fizz"
    else if n % 5 == 0 then "Buzz"
    else int_to_string(n);

fn main() -> i64 =
    var i = 1;
    while i <= 20 {
        println(fizzbuzz(i));
        i = i + 1;
    };
    0;
```

---

*Next: [Contract Programming Guide](./CONTRACT_PROGRAMMING.md)*
