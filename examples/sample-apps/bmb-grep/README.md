# bmb-grep

A pattern matching tool demonstrating BMB language features.

## Usage

```bash
bmb run main.bmb -- "<pattern>" "<text>" [options]
```

## Options

| Flag | Description |
|------|-------------|
| `-n` | Show line numbers |
| `-c` | Count matches only |
| `-v` | Invert match (show non-matching lines) |
| `-i` | Case insensitive matching |
| `-h`, `--help` | Show help |

## Examples

```bash
# Basic pattern search
bmb run main.bmb -- "hello" "hello world"
# Output: hello world

# Search in multi-line text (use \n for newlines)
bmb run main.bmb -- "error" "line1\nline2 error\nline3"
# Output: line2 error

# Show line numbers
bmb run main.bmb -- "test" "test1\nother\ntest2" -n
# Output:
# 1: test1
# 3: test2

# Case insensitive search
bmb run main.bmb -- "hello" "HELLO World" -i
# Output: HELLO World

# Invert match (show non-matching lines)
bmb run main.bmb -- "error" "ok\nerror\nfine" -v
# Output:
# ok
# fine

# Count matches only
bmb run main.bmb -- "a" "banana" -c
# Output: Matches: 3
```

## Features Demonstrated

1. **Contract-based programming** - Pre/postconditions on search functions
2. **Tail recursion** - Efficient iteration via recursive calls
3. **Pattern matching** - Case-sensitive and case-insensitive search
4. **CLI argument parsing** - Flag and positional argument handling
5. **String processing** - Manual character-by-character parsing
6. **Functional style** - Pure functions with immutable data

## Code Structure

| Section | Description |
|---------|-------------|
| Character Utilities | Case conversion, character classification |
| Pattern Matching | Literal pattern search, contains, count |
| Line Processing | Multi-line text handling |
| Output Formatting | Integer to string, line printing |
| Grep Core Logic | Main search algorithm |
| CLI Parsing | Argument extraction |

## Running Tests

```bmb
fn main() -> i64 = run_tests();  // Replace main to run tests
```

Expected output: `777` (pattern tests), `888` (line tests), `999` (CLI tests)
