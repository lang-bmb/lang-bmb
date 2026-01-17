# Migrating from Rust to BMB

> A guide for Rust developers transitioning to BMB

## Overview

If you know Rust, you'll find BMB familiar yet refreshingly simple. BMB takes Rust's safety philosophy but replaces the borrow checker with contract-based verification—achieving similar guarantees through different means.

## Syntax Comparison

### Functions

```rust
// Rust
fn max(a: i64, b: i64) -> i64 {
    if a > b { a } else { b }
}
```

```bmb
// BMB - expression-based, no explicit return
fn max(a: i64, b: i64) -> i64 = if a > b { a } else { b };
```

### Variables

```rust
// Rust
let x = 5;
let mut y = 10;
y = 20;
```

```bmb
// BMB - all bindings are immutable, use new bindings
let x = 5;
let y = 10;
let y = 20;  // Shadowing creates new binding
```

### Control Flow

```rust
// Rust
fn classify(n: i64) -> &'static str {
    if n < 0 {
        "negative"
    } else if n == 0 {
        "zero"
    } else {
        "positive"
    }
}
```

```bmb
// BMB - identical structure
fn classify(n: i64) -> String =
    if n < 0 { "negative" }
    else if n == 0 { "zero" }
    else { "positive" };
```

### Pattern Matching

```rust
// Rust
match opt {
    Some(v) => v * 2,
    None => 0,
}
```

```bmb
// BMB - same syntax
match opt {
    Some(v) => v * 2,
    None => 0
}
```

### Structs

```rust
// Rust
struct Point {
    x: i64,
    y: i64,
}

impl Point {
    fn new(x: i64, y: i64) -> Self {
        Point { x, y }
    }

    fn distance(&self) -> i64 {
        self.x * self.x + self.y * self.y
    }
}
```

```bmb
// BMB - no impl blocks, free functions with struct as first param
struct Point {
    x: i64,
    y: i64
}

fn point_new(x: i64, y: i64) -> Point = Point { x: x, y: y };

fn point_distance(p: Point) -> i64 = p.x * p.x + p.y * p.y;
```

### Enums

```rust
// Rust
enum Option<T> {
    Some(T),
    None,
}
```

```bmb
// BMB - same syntax
enum Option<T> {
    Some(T),
    None
}
```

## Key Differences

### No Borrow Checker → Contracts

Rust enforces memory safety through ownership and borrowing. BMB uses contracts:

```rust
// Rust - compiler tracks ownership
fn process(data: Vec<i64>) -> i64 {
    data[0]  // Borrow checker ensures data is valid
}
```

```bmb
// BMB - contracts specify requirements
fn process(data: Vec<i64>) -> i64
  pre vec_len(data) > 0  // Explicit precondition
= vec_get(data, 0);
```

### No Lifetimes

BMB doesn't have lifetimes. Memory management is explicit:

```rust
// Rust
fn longest<'a>(x: &'a str, y: &'a str) -> &'a str {
    if x.len() > y.len() { x } else { y }
}
```

```bmb
// BMB - returns owned value
fn longest(x: String, y: String) -> String =
    if x.len() > y.len() { x } else { y };
```

### Explicit Memory Management

Instead of RAII, BMB uses explicit allocation/deallocation with contracts:

```rust
// Rust - automatic cleanup
{
    let v = vec![1, 2, 3];
} // v dropped here
```

```bmb
// BMB - explicit management
fn with_buffer(f: fn(i64) -> i64) -> i64 = {
    let buf = malloc(100);
    let result = f(buf);
    let freed = free(buf);
    result
};
```

### No Traits → Functions

```rust
// Rust
trait Display {
    fn display(&self) -> String;
}

impl Display for Point {
    fn display(&self) -> String {
        format!("({}, {})", self.x, self.y)
    }
}
```

```bmb
// BMB - convention-based naming
fn point_display(p: Point) -> String = {
    let sb = sb_new();
    let s1 = sb_push(sb, "(");
    let s2 = sb_push(sb, int_to_string(p.x));
    let s3 = sb_push(sb, ", ");
    let s4 = sb_push(sb, int_to_string(p.y));
    let s5 = sb_push(sb, ")");
    sb_build(sb)
};
```

## Ownership Patterns → Contract Patterns

### Rust's Option Unwrap

```rust
// Rust
fn unwrap<T>(opt: Option<T>) -> T {
    match opt {
        Some(v) => v,
        None => panic!("unwrap on None"),
    }
}
```

```bmb
// BMB - contract ensures caller passes Some
fn unwrap(opt: Option<i64>) -> i64
  pre opt != None  // Compile-time verified
= match opt {
    Some(v) => v,
    None => 0  // Unreachable, but type-safe
};
```

### Rust's Bounds Checking

```rust
// Rust
fn get<T>(v: &[T], i: usize) -> &T {
    &v[i]  // Panics if out of bounds
}
```

```bmb
// BMB - contract prevents out-of-bounds
fn get(v: Vec<i64>, i: i64) -> i64
  pre i >= 0
  pre i < vec_len(v)
= vec_get(v, i);
```

### Rust's Resource Safety

```rust
// Rust - File automatically closed on drop
use std::fs::File;
let file = File::open("test.txt")?;
// file closed when scope ends
```

```bmb
// BMB - explicit close required
fn process_file(path: String) -> i64 = {
    let handle = file_open(path);
    if handle <= 0 { 0 - 1 }
    else {
        let content = file_read(handle);
        let result = parse(content);
        let closed = file_close(handle);  // Explicit
        result
    }
};
```

## Migration Tool

BMB provides a Rust-to-BMB converter:

```bash
# Convert Rust files
node tools/rust_to_bmb.mjs path/to/*.rs --stats   # Preview
node tools/rust_to_bmb.mjs path/to/*.rs --apply   # Convert

# What it handles:
# - fn, struct, enum, impl
# - match expressions
# - Option, Result types
# - Basic control flow
# - Comments
```

### Example Conversion

```rust
// Input: Rust
impl Vec<T> {
    pub fn push(&mut self, value: T) {
        // ...
    }

    pub fn len(&self) -> usize {
        self.length
    }
}
```

```bmb
// Output: BMB (after conversion + manual refinement)
fn vec_push(v: i64, value: i64) -> i64
  pre v != 0
= vec_push_impl(v, value);

fn vec_len(v: i64) -> i64
  pre v != 0
  post ret >= 0
= vec_len_impl(v);
```

## When to Add Contracts

Focus contracts on:

1. **Function boundaries** - Document expected inputs/outputs
2. **Unsafe operations** - Memory access, division, array indexing
3. **Critical invariants** - State machine transitions, resource validity

```bmb
// Good: contracts on public API
fn binary_search(arr: Vec<i64>, target: i64) -> i64
  pre is_sorted(arr)
  post ret == -1 or vec_get(arr, ret) == target
= binary_search_impl(arr, target, 0, vec_len(arr));

// Internal helper - fewer contracts needed
fn binary_search_impl(arr: Vec<i64>, target: i64, lo: i64, hi: i64) -> i64 =
    if lo >= hi { -1 }
    else {
        let mid = lo + (hi - lo) / 2;
        let v = vec_get(arr, mid);
        if v == target { mid }
        else if v < target { binary_search_impl(arr, target, mid + 1, hi) }
        else { binary_search_impl(arr, target, lo, mid) }
    };
```

## Performance Comparison

| Benchmark | Rust | BMB | Notes |
|-----------|------|-----|-------|
| fibonacci(45) | 1.66s | 1.63s | BMB 2% faster |
| mandelbrot | 42ms | 39ms | BMB 7% faster |
| spectral_norm | 44ms | 39ms | BMB 11% faster |

BMB achieves comparable or better performance because:
- No runtime borrow checking overhead
- Contracts have zero runtime cost
- Same LLVM backend as Rust

## Common Pitfalls

### 1. Forgetting Expression-Based Semantics

```rust
// Rust habit
fn foo() -> i64 {
    let x = compute();
    return x + 1;
}
```

```bmb
// BMB way - last expression is the result
fn foo() -> i64 = {
    let x = compute();
    x + 1  // No semicolon, no return
};
```

### 2. Using Mutable Variables

```rust
// Rust
let mut sum = 0;
for i in 0..n {
    sum += i;
}
```

```bmb
// BMB - use recursion or fold
fn sum_to_n(n: i64) -> i64 = sum_iter(n, 0);

fn sum_iter(n: i64, acc: i64) -> i64 =
    if n <= 0 { acc }
    else { sum_iter(n - 1, acc + n) };
```

### 3. Expecting Automatic Cleanup

```bmb
// Must explicitly free resources
fn process() -> i64 = {
    let buf = malloc(100);
    let result = work(buf);
    let f = free(buf);  // Don't forget!
    result
};
```

## Getting Started

1. **Install BMB**
   ```bash
   cargo install bmb
   ```

2. **Convert a small Rust file**
   ```bash
   node tools/rust_to_bmb.mjs my_file.rs --apply
   ```

3. **Add contracts gradually**
   ```bmb
   fn my_function(x: i64) -> i64
     pre x > 0       // Add when you understand the constraints
     post ret >= x   // Add when you know the guarantees
   = /* implementation */;
   ```

4. **Verify with the compiler**
   ```bash
   bmb verify my_file.bmb
   ```

## Next Steps

- [Contracts](CONTRACTS.md) - Master contract-based programming
- [Systems](SYSTEMS.md) - Low-level BMB patterns
- [Performance](PERFORMANCE.md) - Optimization techniques
