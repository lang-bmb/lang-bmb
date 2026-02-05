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

## Using Standard Library Packages

BMB comes with official packages distributed via the `gotgan` package manager:

### Core Packages

```bmb
-- Basic types and math
use bmb_core::abs;
use bmb_core::min;
use bmb_core::max;

-- String utilities
use bmb_string::starts_with;
use bmb_string::trim;
use bmb_string::split_lines;

-- Array operations
use bmb_array::sum_i64;
use bmb_array::is_sorted_asc;
```

### Data Packages

```bmb
-- JSON parsing and serialization
use bmb_json::parse_json;
use bmb_json::to_json;
use bmb_json::get_field;

fn process_config(json_str: String) -> i64 =
    let obj = parse_json(json_str);
    let name = get_field(obj, "name");
    if name.len() > 0 { 1 } else { 0 };
```

### Network Packages

```bmb
-- HTTP client (uses curl backend)
use bmb_http::http_get;
use bmb_http::http_post;
use bmb_http::get_status_code;

fn fetch_data(url: String) -> String =
    let response = http_get(url);
    let status = get_status_code(response);
    if status == 200 { get_body(response) } else { "" };
```

### Pattern Matching Packages

```bmb
-- Regular expressions
use bmb_regex::matches;
use bmb_regex::search;
use bmb_regex::find;

fn validate_email(email: String) -> bool =
    search("[a-zA-Z0-9]+@[a-zA-Z0-9]+\\.[a-z]+", email);
```

### Available Packages

| Package | Description |
|---------|-------------|
| `bmb-core` | Core types and math functions |
| `bmb-string` | String utilities |
| `bmb-array` | Fixed-size array operations |
| `bmb-io` | File I/O operations |
| `bmb-process` | Process execution |
| `bmb-test` | Test assertions |
| `bmb-json` | JSON parser/serializer |
| `bmb-http` | HTTP client |
| `bmb-regex` | Regular expressions |

See [API Documentation](../api/README.md) for complete reference.

## Practical Example: Config Processor

Here's a complete example combining multiple packages:

```bmb
-- config_processor.bmb
-- A program that reads, validates, and processes a JSON config file

use bmb_io::read_file;
use bmb_io::file_exists;
use bmb_json::parse_json;
use bmb_json::get_field;
use bmb_json::get_element;
use bmb_json::array_length;
use bmb_regex::matches;
use bmb_string::int_to_string;

-- Validate that a URL has correct format
fn is_valid_url(url: String) -> bool =
    matches("https?://[a-zA-Z0-9.-]+", url);

-- Extract config value with validation
fn get_config_url(config: String) -> String
  post ret.len() == 0 or is_valid_url(ret)
=
    let url = get_field(config, "api_url");
    if is_valid_url(url) { url } else { "" };

-- Count enabled features in config
fn count_features(config: String) -> i64
  post ret >= 0
=
    let features = get_field(config, "features");
    array_length(features);

fn main() -> i64 =
    let config_path = "config.json";

    -- Check if config exists
    if not file_exists(config_path) {
        let _ = println("Error: config.json not found");
        1
    } else {
        -- Read and parse config
        let content = read_file(config_path);
        let config = parse_json(content);

        -- Extract and validate
        let url = get_config_url(config);
        let feature_count = count_features(config);

        -- Report
        let _ = println("API URL: " + url);
        let _ = println("Features: " + int_to_string(feature_count));
        0
    };
```

Run it:

```bash
bmb build config_processor.bmb -o processor
./processor
```

## Package Management with Gotgan

BMB uses `gotgan` for package management:

```bash
# Initialize a new project
gotgan init my-project

# Add a dependency
gotgan add bmb-json

# Build the project
gotgan build
```

Example `gotgan.toml`:

```toml
[package]
name = "my-project"
version = "0.1.0"
edition = "2026"

[dependencies]
bmb-json = "0.1.0"
bmb-http = "0.1.0"
bmb-regex = "0.1.0"
```

## Next Steps

1. **[By Example](./BY_EXAMPLE.md)**: Learn BMB through practical examples
2. **[Contract Programming](./CONTRACT_PROGRAMMING.md)**: Deep dive into verification
3. **[From Rust](../scenarios/FROM_RUST.md)**: Migration guide for Rust developers
4. **[Language Reference](../LANGUAGE_REFERENCE.md)**: Complete syntax reference
5. **[API Documentation](../api/README.md)**: Package API reference

## Getting Help

- GitHub Issues: [github.com/lang-bmb/bmb/issues](https://github.com/lang-bmb/bmb/issues)
- Documentation: [bmb.dev](https://bmb.dev) (coming soon)

---

*BMB v0.66 - AI-Native Programming Language*
