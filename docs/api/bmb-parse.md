# bmb-parse — Parse Module API

Position-based text parsing utilities for structured text processing.

## Core Parsing

| Function | Signature | Description |
|----------|-----------|-------------|
| `skip_ws(s, pos)` | `(String, i64) -> i64` | Skip whitespace from position |
| `skip_until(s, pos, ch)` | `(String, i64, i64) -> i64` | Skip until character found |
| `read_word(s, pos)` | `(String, i64) -> String` | Read word (non-whitespace) |
| `read_int(s, pos)` | `(String, i64) -> i64` | Parse integer at position |
| `read_until(s, pos, ch)` | `(String, i64, i64) -> String` | Read until delimiter |

## Field Extraction

| Function | Signature | Description |
|----------|-----------|-------------|
| `field(s, n, sep)` | `(String, i64, i64) -> String` | Extract nth field by separator |
| `field_count(s, sep)` | `(String, i64) -> i64` | Count fields |

## Pattern Matching

| Function | Signature | Description |
|----------|-----------|-------------|
| `starts_with_at(s, prefix, pos)` | `(String, String, i64) -> bool` | Check prefix at position |
| `find_str(s, needle, pos)` | `(String, String, i64) -> i64` | Find substring |

All functions use **position-based** parsing — pass the current position and get back the new position. This enables zero-allocation parsing.
