# Getting Started with BMB

> Learn BMB in 15 minutes: Installation, Hello World, and your first contract-verified program.

## What is BMB?

BMB (Bare-Metal-Banter) is an **AI-Native programming language** designed for:

- **Contract-based verification**: Pre/post conditions verified at compile time
- **Native performance**: Targets LLVM IR for C/Rust-level speed
- **AI code generation**: Syntax optimized for AI understanding and generation
- **Memory safety**: Ownership and borrowing model (Rust-influenced)

## Prerequisites

- Rust toolchain (for now - will be removed in future versions)
- LLVM 15+ (for native compilation)
- Z3 SMT solver (for contract verification)

## Installation

### Option 1: Build from Source

```bash
# Clone repository
git clone https://github.com/iyulab/lang-bmb.git
cd lang-bmb

# Build compiler
cargo build --release

# Add to PATH (Unix)
export PATH="$PATH:$(pwd)/target/release"

# Add to PATH (Windows PowerShell)
$env:PATH += ";$(pwd)\target\release"
```

### Option 2: Pre-built Binary

```bash
# Download latest release
# Visit: https://github.com/iyulab/lang-bmb/releases
```

### Verify Installation

```bash
bmb --version
# Output: bmb 0.33.x
```

## Hello World

Create a file `hello.bmb`:

```bmb
-- hello.bmb
-- BMB uses line comments with --

fn main() -> i64 =
    println("Hello, BMB!");
    0;
```

Run it:

```bash
bmb run hello.bmb
# Output: Hello, BMB!
```

## Basic Syntax

### Functions

BMB functions are expression-based. The last expression is the return value:

```bmb
-- Simple function with expression body
fn add(a: i64, b: i64) -> i64 = a + b;

-- Multi-line function with let bindings
fn calculate(x: i64) -> i64 =
    let doubled = x * 2;
    let squared = doubled * doubled;
    squared + 1;
```

### Variables

```bmb
-- Immutable binding (default)
let x = 42;

-- Mutable binding
var y = 10;
y = y + 1;  -- OK: y is mutable
```

### Control Flow

```bmb
-- If expression (not statement)
fn max(a: i64, b: i64) -> i64 =
    if a > b then a else b;

-- Match expression
fn describe(n: i64) -> String =
    match n {
        0 => "zero",
        1 => "one",
        _ => "many"
    };
```

### Types

| Type | Description | Example |
|------|-------------|---------|
| `i64` | 64-bit signed integer | `42`, `-1` |
| `bool` | Boolean | `true`, `false` |
| `String` | UTF-8 string | `"hello"` |
| `[T; N]` | Fixed-size array | `[1, 2, 3]` |
| `(T1, T2)` | Tuple | `(1, "a")` |

## Your First Contract

BMB's killer feature is **compile-time contract verification**. Contracts are checked by the Z3 SMT solver before your code runs.

### Preconditions

Preconditions (`pre`) specify what must be true before a function runs:

```bmb
fn divide(a: i64, b: i64) -> i64
  pre b != 0  -- Compile error if caller might pass 0
= a / b;
```

### Postconditions

Postconditions (`post`) specify what will be true after a function returns:

```bmb
fn abs(x: i64) -> i64
  post ret >= 0  -- Compiler verifies this is always true
= if x >= 0 then x else 0 - x;
```

### Inline Refinement Types (v0.2+)

Modern BMB syntax puts constraints directly in type annotations:

```bmb
-- Parameter constraint: b must not be zero
fn divide(a: i64, b: i64{it != 0}) -> i64 = a / b;

-- Return constraint: result is always non-negative
fn abs(x: i64) -> i64{it >= 0} =
    if x >= 0 then x else 0 - x;

-- Combined constraints
fn clamp(x: i64, lo: i64, hi: i64{it >= lo}) -> i64{it >= lo, it <= hi} =
    if x < lo then lo
    else if x > hi then hi
    else x;
```

### Named Contracts with `where`

For complex constraints, use named contracts:

```bmb
fn binary_search(arr: [i64; 8], len: i64, target: i64) -> i64
  where {
    valid_len: len > 0 and len <= 8,
    sorted: is_sorted_asc(arr, len)
  }
  post (ret == -1) or (ret >= 0 and ret < len)
= -- implementation
```

## Running and Building

### Run with Interpreter

```bash
bmb run program.bmb
```

### Type Check Only

```bash
bmb check program.bmb
```

### Verify Contracts

```bash
bmb verify program.bmb
```

### Build Native Executable

```bash
bmb build program.bmb
./program
```

### Interactive REPL

```bash
bmb repl
> let x = 42;
> x + 1
43
```

## Using the Standard Library

BMB comes with a standard library of 231 symbols:

```bmb
-- Import specific functions
use string::starts_with;
use array::sum_i64;
use core::num::abs;

-- Use in your code
fn process(s: String) -> i64 =
    if starts_with(s, "BMB") then 1 else 0;
```

Available modules:
- `core::num` - Numeric operations (abs, min, max, clamp)
- `core::bool` - Boolean operations (implies, xor)
- `core::option` - Optional values (is_some, unwrap)
- `core::result` - Error handling (is_ok, safe_divide)
- `string` - String utilities (44 functions)
- `array` - Array operations (35 functions)
- `parse` - Parsing utilities (31 functions)
- `test` - Test assertions (47 functions)

See [stdlib/README.md](../stdlib/README.md) for complete API documentation.

## Next Steps

1. **[By Example](./BY_EXAMPLE.md)**: Learn BMB through practical examples
2. **[Contract Programming](./CONTRACT_PROGRAMMING.md)**: Deep dive into verification
3. **[From Rust](./FROM_RUST.md)**: Migration guide for Rust developers
4. **[Language Reference](./LANGUAGE_REFERENCE.md)**: Complete syntax reference

## Getting Help

- GitHub Issues: [github.com/iyulab/lang-bmb/issues](https://github.com/iyulab/lang-bmb/issues)
- Documentation: [bmb.dev](https://bmb.dev) (coming soon)

---

*BMB v0.33.1 - AI-Native Programming Language*
