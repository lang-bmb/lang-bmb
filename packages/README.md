# BMB Official Packages

This directory contains the official BMB standard library packages.

## Package List

### Core Packages

| Package | Version | Description |
|---------|---------|-------------|
| [bmb-core](./bmb-core) | 0.1.0 | Core types and functions (bool, numeric, math) |
| [bmb-traits](./bmb-traits) | 0.14.0 | Core traits (Debug, Clone, PartialEq, Default) |
| [bmb-option](./bmb-option) | 0.14.0 | Generic Option<T> type for optional values |
| [bmb-result](./bmb-result) | 0.14.0 | Generic Result<T, E> type for error handling |

### Data Structure Packages

| Package | Version | Description |
|---------|---------|-------------|
| [bmb-string](./bmb-string) | 0.1.0 | String utility functions |
| [bmb-array](./bmb-array) | 0.1.0 | Fixed-size array utilities |
| [bmb-iter](./bmb-iter) | 0.14.0 | Iterator trait and combinators |

### System Packages

| Package | Version | Description |
|---------|---------|-------------|
| [bmb-io](./bmb-io) | 0.1.0 | File I/O operations |
| [bmb-process](./bmb-process) | 0.1.0 | Process execution utilities |
| [bmb-runtime](./bmb-runtime) | 0.1.0 | Self-hosted replacement for C runtime |

### Testing Packages

| Package | Version | Description |
|---------|---------|-------------|
| [bmb-test](./bmb-test) | 0.1.0 | Test assertion utilities |

## Usage

Add dependencies to your `gotgan.toml`:

```toml
[dependencies]
bmb-core = "0.1.0"
bmb-string = "0.1.0"
bmb-option = "0.14.0"
```

Then import in your BMB code:

```bmb
use bmb_core::abs;
use bmb_string::string_eq;
use bmb_option::Option;

fn main() -> i64 = abs(-42);
```

## Package Structure

Each package follows this structure:

```
<package-name>/
├── gotgan.toml    # Package manifest (or Gotgan.toml)
└── src/
    └── lib.bmb    # Library source
```

## Contract Verification

All functions in these packages have explicit pre/post conditions:

```bmb
pub fn abs(x: i64) -> i64
    post ret >= 0
= if x >= 0 { x } else { 0 - x };
```

This enables:
- Compile-time verification with Z3
- Bounds check elimination
- Documentation of function behavior

## Dependency Graph

```
bmb-core ─────────────────────────────────────────────────┐
    │                                                     │
    ├──► bmb-traits                                       │
    │                                                     │
    ├──► bmb-option ─────┬──► bmb-result                  │
    │                    │                                │
    │                    └──► bmb-iter                    │
    │                                                     │
    ├──► bmb-string ──────────► bmb-test ◄────────────────┤
    │                              ▲                      │
    └──► bmb-array ────────────────┘                      │
                                                          │
bmb-io, bmb-process, bmb-runtime ◄────────────────────────┘
```

## Contributing

These packages are part of the BMB compiler distribution. To contribute:

1. Fork the [lang-bmb/bmb](https://github.com/lang-bmb/bmb) repository
2. Make changes in the `packages/` directory
3. Run `cargo test` to verify
4. Submit a pull request

## License

MIT License - See [LICENSE](../LICENSE) for details.
