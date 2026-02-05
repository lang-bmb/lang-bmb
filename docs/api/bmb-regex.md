# bmb-regex API Reference

Simple regex engine for BMB with backtracking support.

## Overview

This package provides basic regular expression matching for BMB. It implements a backtracking regex engine supporting common patterns.

## Supported Pattern Syntax

| Pattern | Description | Example |
|---------|-------------|---------|
| `.` | Any single character | `a.c` matches "abc", "aXc" |
| `*` | Zero or more of preceding | `ab*c` matches "ac", "abc", "abbc" |
| `+` | One or more of preceding | `ab+c` matches "abc", "abbc" (not "ac") |
| `?` | Zero or one of preceding | `ab?c` matches "ac", "abc" |
| `[abc]` | Character class | `[aeiou]` matches any vowel |
| `[^abc]` | Negated character class | `[^0-9]` matches non-digits |
| `[a-z]` | Character range | `[a-z]` matches lowercase letters |
| `^` | Start anchor | `^hello` matches "hello world" |
| `$` | End anchor | `world$` matches "hello world" |
| `\d` | Digit (0-9) | `\d+` matches "123" |
| `\w` | Word character (a-z, A-Z, 0-9, _) | `\w+` matches "hello_123" |
| `\s` | Whitespace (space, tab, newline) | `\s+` matches "   " |
| `\` | Escape next character | `\.` matches literal "." |

## Main API

### matches

```bmb
pub fn matches(pattern: String, text: String) -> bool
```

Check if pattern matches the **entire** string.

**Example:**
```bmb
matches("hello", "hello")           // true
matches("hello", "hello world")     // false
matches("hello.*", "hello world")   // true
matches("[a-z]+", "hello")          // true
matches("\\d+", "123")              // true
```

### search

```bmb
pub fn search(pattern: String, text: String) -> bool
```

Check if pattern matches **anywhere** in the string.

**Example:**
```bmb
search("world", "hello world")      // true
search("xyz", "hello world")        // false
search("\\d+", "abc123def")         // true
```

### find

```bmb
pub fn find(pattern: String, text: String) -> i64
    post ret >= -1 and ret <= text.len()
```

Find the starting position of the first match. Returns `-1` if not found.

**Example:**
```bmb
find("world", "hello world")        // 6
find("xyz", "hello world")          // -1
find("\\d+", "abc123def")           // 3
```

### matches_start

```bmb
pub fn matches_start(pattern: String, text: String) -> bool
```

Check if pattern matches at the **start** of the string.

**Example:**
```bmb
matches_start("hello", "hello world")  // true
matches_start("world", "hello world")  // false
```

## Utility Functions

### split

```bmb
pub fn split(pattern: String, text: String) -> String
```

Split string by regex pattern. Returns newline-separated parts.

**Example:**
```bmb
split(",", "a,b,c")                 // "a\nb\nc"
split("\\s+", "hello   world")      // "hello\nworld"
```

### replace_first

```bmb
pub fn replace_first(pattern: String, text: String, replacement: String) -> String
```

Replace the first match with a replacement string.

**Example:**
```bmb
replace_first("world", "hello world", "BMB")  // "hello BMB"
replace_first("\\d+", "abc123def", "XXX")     // "abcXXXdef"
```

## Examples

### Email Validation (Simple)

```bmb
use bmb_regex::search;

fn is_valid_email(email: String) -> bool =
    search("[a-zA-Z0-9]+@[a-zA-Z0-9]+\\.[a-z]+", email);

fn main() -> i64 =
    if is_valid_email("user@example.com") { 0 } else { 1 };
```

### Extract Numbers

```bmb
use bmb_regex::find;
use bmb_regex::search;

fn has_number(text: String) -> bool =
    search("\\d+", text);

fn find_first_number_pos(text: String) -> i64 =
    find("\\d+", text);
```

### Pattern Matching for Validation

```bmb
use bmb_regex::matches;

// Validate phone number format (XXX-XXX-XXXX)
fn is_phone_number(text: String) -> bool =
    matches("\\d\\d\\d-\\d\\d\\d-\\d\\d\\d\\d", text);

// Validate identifier (letter followed by letters/digits)
fn is_identifier(text: String) -> bool =
    matches("[a-zA-Z][a-zA-Z0-9]*", text);

// Validate hex color (#RRGGBB)
fn is_hex_color(text: String) -> bool =
    matches("#[0-9a-fA-F][0-9a-fA-F][0-9a-fA-F][0-9a-fA-F][0-9a-fA-F][0-9a-fA-F]", text);
```

### Text Processing

```bmb
use bmb_regex::replace_first;
use bmb_regex::split;

fn normalize_whitespace(text: String) -> String =
    // Replace multiple spaces with single space (simplified)
    replace_first("  +", text, " ");

fn tokenize(text: String) -> String =
    // Split by whitespace
    split("\\s+", text);
```

## Limitations

- No capturing groups
- No alternation (`|`)
- No lookahead/lookbehind
- No backreferences
- Greedy matching only (no lazy quantifiers)

## Performance Notes

This is a backtracking implementation. Patterns with many quantifiers on ambiguous patterns (like `.*.*.*`) may have exponential worst-case behavior. For performance-critical applications, keep patterns simple.

## Contracts

All public functions have contracts verified at compile time:

- `find` guarantees: `post ret >= -1 and ret <= text.len()`

The implementation uses BMB's contract verification to ensure correctness.
