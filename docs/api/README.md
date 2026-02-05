# BMB Standard Library API Reference

This directory contains auto-generated API documentation for BMB standard library packages.

## Package Documentation

| Package | Description | Doc |
|---------|-------------|-----|
| [bmb-core](./bmb-core.md) | Core types and functions (bool, numeric, math) | [View](./bmb-core.md) |
| [bmb-string](./bmb-string.md) | String utility functions | [View](./bmb-string.md) |
| [bmb-io](./bmb-io.md) | File I/O operations | [View](./bmb-io.md) |
| [bmb-array](./bmb-array.md) | Fixed-size array utilities | [View](./bmb-array.md) |
| [bmb-test](./bmb-test.md) | Test assertion utilities | [View](./bmb-test.md) |
| [bmb-process](./bmb-process.md) | Process execution utilities | [View](./bmb-process.md) |
| [bmb-json](./bmb-json.md) | JSON parser and serializer | [View](./bmb-json.md) |

## Generation

Documentation is generated using `bmb-doc`:

```bash
./target/release/bmb run tools/bmb-doc/main.bmb -- <source-file.bmb>
```

## Contract Documentation

All functions in these packages have explicit pre/post conditions documented. For example:

```bmb
/// Absolute value of an integer
pub fn abs(x: i64) -> i64
    post ret >= 0
= if x >= 0 { x } else { 0 - x };
```

The `post ret >= 0` contract is verified at compile-time and documents that the return value is always non-negative.

## Updates

This documentation was last generated on 2026-02-05 using `bmb-doc` v0.64.
