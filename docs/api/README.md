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
| [bmb-http](./bmb-http.md) | HTTP client using curl backend | [View](./bmb-http.md) |
| [bmb-regex](./bmb-regex.md) | Simple regex engine for pattern matching | [View](./bmb-regex.md) |
| [bmb-time](./bmb-time.md) | Monotonic clock, sleep, duration utilities | [View](./bmb-time.md) |
| [bmb-fs](./bmb-fs.md) | Filesystem operations and path utilities | [View](./bmb-fs.md) |
| [bmb-math](./bmb-math.md) | Mathematical functions (trig, power, gcd) | [View](./bmb-math.md) |
| [bmb-collections](./bmb-collections.md) | Data structures (Stack, Heap, Queue) | [View](./bmb-collections.md) |
| [bmb-parse](./bmb-parse.md) | Position-based text parsing | [View](./bmb-parse.md) |

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

This documentation was last updated on 2026-03-22. New modules (time, fs) added manually.
