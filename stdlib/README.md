# BMB Standard Library

> v0.6.0 Leaf: Foundation for BMB standard library

## Design Principles (AI-Native)

| Principle | Description |
|-----------|-------------|
| **Contract-First** | All functions have explicit pre/post conditions |
| **Zero Ambiguity** | No implicit conversions or default behaviors |
| **Verification** | Every constraint is SMT-verifiable |
| **Explicit Types** | Specialized types until generics are available |

## Structure

```
stdlib/
├── core/
│   ├── num.bmb       # Numeric operations (abs, min, max, clamp)
│   ├── bool.bmb      # Boolean operations
│   ├── option.bmb    # Option type (specialized for i64)
│   └── result.bmb    # Result type (specialized for i64, String)
├── string/           # String operations (v0.6.1)
└── collections/      # Collection types (v0.6.2)
```

## Module Status

| Module | Functions | Status |
|--------|-----------|--------|
| core::num | 4 | v0.6.0 |
| core::bool | 3 | v0.6.0 |
| core::option | 5 | v0.6.0 |
| core::result | 5 | v0.6.0 |
| string | 15 | v0.6.1 |
| collections | 15 | v0.6.2 |

## Usage

```bmb
-- Import specific functions
use core::num::abs;
use core::option::Option;

fn main() -> i64 =
    let x = abs(-42);
    let opt = Option::Some(x);
    match opt {
        Option::Some(v) => v,
        Option::None => 0
    };
```

## Generics Note

Current implementation uses type-specialized versions:
- `Option` = `Option_i64` (holds i64)
- `Result` = `Result_i64_String` (Ok: i64, Err: String)

Full generics (`Option<T>`, `Result<T, E>`) planned for v0.6.1+.

## Contract Patterns

### Preconditions (pre)
```bmb
fn divide(a: i64, b: i64) -> i64
  pre b != 0
= a / b;
```

### Postconditions (post)
```bmb
fn abs(x: i64) -> i64
  post ret >= 0
  post (x >= 0 and ret == x) or (x < 0 and ret == 0 - x)
= if x >= 0 then x else 0 - x;
```

### Combined Contracts
```bmb
fn clamp(x: i64, lo: i64, hi: i64) -> i64
  pre lo <= hi
  post ret >= lo and ret <= hi
  post (x < lo and ret == lo) or (x > hi and ret == hi) or ret == x
= if x < lo then lo else if x > hi then hi else x;
```
