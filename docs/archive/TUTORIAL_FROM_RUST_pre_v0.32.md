# BMB for Rust Developers

> A migration guide for Rust developers transitioning to BMB.

## Overview

BMB shares many concepts with Rust but has key differences in syntax and philosophy. This guide helps Rust developers leverage their existing knowledge.

## Quick Comparison

| Feature | Rust | BMB |
|---------|------|-----|
| Comments | `//` | `--` |
| Functions | `fn foo() -> T { body }` | `fn foo() -> T = body;` |
| Mutability | `let mut x` | `var x` |
| If expression | `if cond { a } else { b }` | `if cond then a else b` |
| Match | `match x { ... }` | `match x { ... }` (similar) |
| Traits | `trait` | `trait` (similar) |
| Contracts | None (use asserts) | `pre`, `post`, `where` |
| Null safety | `Option<T>` | `Option` (i64 specialized) |
| Error handling | `Result<T, E>` | `Result` (i64 specialized) |

---

## Syntax Mapping

### Functions

**Rust:**
```rust
fn add(a: i64, b: i64) -> i64 {
    a + b
}

fn complex_function(x: i64) -> i64 {
    let y = x * 2;
    let z = y + 10;
    z
}
```

**BMB:**
```bmb
fn add(a: i64, b: i64) -> i64 = a + b;

fn complex_function(x: i64) -> i64 =
    let y = x * 2;
    let z = y + 10;
    z;
```

**Key Differences:**
- Use `=` instead of `{ }` for function body
- Expression-based: last expression is return value
- Semicolon after final expression

### Variables

**Rust:**
```rust
let x = 42;           // immutable
let mut y = 10;       // mutable
y = y + 1;
```

**BMB:**
```bmb
let x = 42;           -- immutable
var y = 10;           -- mutable
y = y + 1;
```

**Key Difference:** Use `var` instead of `let mut`

### Control Flow

**Rust:**
```rust
// If expression
let max = if a > b { a } else { b };

// Match
let result = match n {
    0 => "zero",
    1 => "one",
    _ => "other",
};
```

**BMB:**
```bmb
-- If expression
let max = if a > b then a else b;

-- Match
let result = match n {
    0 => "zero",
    1 => "one",
    _ => "other"
};
```

**Key Difference:** Use `then` instead of `{ }` in if expressions

### Structs

**Rust:**
```rust
struct Point {
    x: i64,
    y: i64,
}

impl Point {
    fn new(x: i64, y: i64) -> Self {
        Point { x, y }
    }

    fn distance(&self, other: &Point) -> i64 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        dx * dx + dy * dy
    }
}
```

**BMB:**
```bmb
struct Point {
    x: i64,
    y: i64
}

fn Point_new(x: i64, y: i64) -> Point =
    Point { x: x, y: y };

fn Point_distance(self: Point, other: Point) -> i64 =
    let dx = self.x - other.x;
    let dy = self.y - other.y;
    dx * dx + dy * dy;
```

**Key Differences:**
- No `impl` blocks (functions are standalone)
- No `Self` type alias
- Explicit `self` parameter

### Enums

**Rust:**
```rust
enum Option<T> {
    Some(T),
    None,
}

enum Result<T, E> {
    Ok(T),
    Err(E),
}
```

**BMB:**
```bmb
-- BMB currently uses specialized types (no generics yet)
enum Option {
    Some(i64),
    None
}

enum Result {
    Ok(i64),
    Err(i64)
}
```

**Key Difference:** No generics yet; use type-specialized versions

### Pattern Matching

**Rust:**
```rust
match opt {
    Some(x) if x > 0 => x,
    Some(_) => 0,
    None => -1,
}
```

**BMB:**
```bmb
match opt {
    Option::Some(x) if x > 0 => x,
    Option::Some(_) => 0,
    Option::None => -1
}
```

**Key Differences:**
- Use `::` for enum variant paths
- No trailing comma required

### Closures

**Rust:**
```rust
let add = |a, b| a + b;
let double = |x: i64| x * 2;
```

**BMB:**
```bmb
let add = fn |a: i64, b: i64| { a + b };
let double = fn |x: i64| { x * 2 };
```

**Key Differences:**
- Use `fn |params| { body }` syntax
- Type annotations required

---

## Contracts vs Asserts

The biggest conceptual difference is BMB's compile-time contract verification.

### Rust (Runtime Assertions)

```rust
fn divide(a: i64, b: i64) -> i64 {
    assert!(b != 0, "Division by zero!");  // Runtime check
    a / b
}

fn binary_search(arr: &[i64], target: i64) -> Option<usize> {
    debug_assert!(is_sorted(arr));  // Debug-only runtime check
    // ...
}
```

### BMB (Compile-Time Verification)

```bmb
fn divide(a: i64, b: i64) -> i64
  pre b != 0  -- Compile-time verification!
= a / b;

fn binary_search(arr: [i64; 8], len: i64, target: i64) -> i64
  pre len > 0 and len <= 8
  pre is_sorted_asc(arr, len)  -- Compiler verifies caller passes sorted array
  post (ret == -1) or (ret >= 0 and ret < len)
= -- implementation
```

### Modern BMB Syntax (Inline Refinement)

```bmb
-- Constraints directly in type position
fn divide(a: i64, b: i64{it != 0}) -> i64 = a / b;

-- Multiple constraints
fn clamp(x: i64, lo: i64, hi: i64{it >= lo}) -> i64{it >= lo, it <= hi} =
    if x < lo then lo else if x > hi then hi else x;
```

---

## Memory Model Comparison

### Rust

```rust
fn process(data: &Vec<i64>) -> i64 {    // Borrow
    data.iter().sum()
}

fn consume(data: Vec<i64>) -> i64 {      // Ownership transfer
    data.into_iter().sum()
}

fn modify(data: &mut Vec<i64>) {         // Mutable borrow
    data.push(42);
}
```

### BMB

BMB has a similar ownership model but with simplified syntax:

```bmb
fn process(data: &[i64; 8]) -> i64 =     -- Immutable borrow
    sum_i64(data, 8);

fn consume(data: own [i64; 8]) -> i64 =  -- Ownership transfer
    sum_i64(data, 8);

fn modify(data: &mut [i64; 8]) -> () =   -- Mutable borrow
    data[0] = 42;
```

**Note:** BMB's memory model is still evolving. Current focus is on fixed-size arrays.

---

## Error Handling

### Rust

```rust
fn divide(a: i64, b: i64) -> Result<i64, &'static str> {
    if b == 0 {
        Err("division by zero")
    } else {
        Ok(a / b)
    }
}

fn main() {
    match divide(10, 2) {
        Ok(result) => println!("{}", result),
        Err(e) => println!("Error: {}", e),
    }
}
```

### BMB

```bmb
use core::result::Result;
use core::result::ok;
use core::result::err;
use core::result::ERR_DIVIDE_BY_ZERO;

fn divide(a: i64, b: i64) -> Result =
    if b == 0 then err(ERR_DIVIDE_BY_ZERO())
    else ok(a / b);

fn main() -> i64 =
    match divide(10, 2) {
        Result::Ok(result) => println(int_to_string(result)),
        Result::Err(_) => println("Error!")
    };
    0;
```

### Or Use Contracts (Preferred)

```bmb
-- Compiler ensures b != 0 at all call sites
fn divide(a: i64, b: i64{it != 0}) -> i64 = a / b;
```

---

## Traits

### Rust

```rust
trait Display {
    fn display(&self) -> String;
}

impl Display for Point {
    fn display(&self) -> String {
        format!("({}, {})", self.x, self.y)
    }
}
```

### BMB

```bmb
trait Display {
    fn display(self: Self) -> String;
}

impl Display for Point {
    fn display(self: Point) -> String =
        "(" + int_to_string(self.x) + ", " + int_to_string(self.y) + ")";
}
```

---

## Migration Checklist

### Syntax Changes

- [ ] Replace `//` comments with `--`
- [ ] Replace `fn foo() -> T { body }` with `fn foo() -> T = body;`
- [ ] Replace `let mut` with `var`
- [ ] Replace `if cond { a } else { b }` with `if cond then a else b`
- [ ] Replace `|x| expr` closures with `fn |x: T| { expr }`
- [ ] Add `::` prefix to enum variants

### Conceptual Changes

- [ ] Convert runtime `assert!` to compile-time `pre`/`post`
- [ ] Replace generic types with specialized versions (temporary)
- [ ] Use stdlib functions from appropriate modules

### Testing

- [ ] Replace `#[test]` with `fn test_*() -> bool`
- [ ] Replace `assert_eq!` with `assert_eq_i64` or similar
- [ ] Run `bmb test` instead of `cargo test`

---

## Common Gotchas

### 1. Expression vs Statement

**Rust:** Blocks are expressions
```rust
let x = {
    let a = 1;
    let b = 2;
    a + b  // No semicolon = return value
};
```

**BMB:** Everything is an expression
```bmb
let x =
    let a = 1;
    let b = 2;
    a + b;  -- Semicolon required, still returns value
```

### 2. Type Inference

BMB has less type inference than Rust. Explicit types are often needed:

```bmb
-- May need type annotation
let arr: [i64; 4] = [1, 2, 3, 4];
let f: fn(i64) -> i64 = fn |x: i64| { x + 1 };
```

### 3. No Generic Collections (Yet)

Rust's `Vec<T>`, `HashMap<K, V>` don't exist yet in BMB. Use fixed arrays:

```bmb
-- Instead of Vec<i64>
let data: [i64; 8] = [0, 0, 0, 0, 0, 0, 0, 0];
let len = 3;  -- Track length separately
```

### 4. No Lifetime Annotations

BMB's borrow checker doesn't require explicit lifetimes:

```rust
// Rust: explicit lifetime
fn longest<'a>(x: &'a str, y: &'a str) -> &'a str { ... }
```

```bmb
-- BMB: no lifetime annotation needed
fn longest(x: &String, y: &String) -> &String = ...;
```

---

*Next: [Contract Programming Guide](./CONTRACT_PROGRAMMING.md)*
