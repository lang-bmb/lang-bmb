# bmb-json-tool

A JSON processing CLI demonstrating BMB language features.

## Usage

```bash
bmb run main.bmb -- <command> <json> [path]
```

## Commands

| Command | Description | Example |
|---------|-------------|---------|
| `type <json>` | Show value type | `type '{"a":1}'` -> object |
| `length <json>` | Array/object/string length | `length '[1,2,3]'` -> 3 |
| `keys <json>` | List object keys | `keys '{"a":1,"b":2}'` -> a, b |
| `validate <json>` | Check if valid JSON | `validate '{}'` -> valid |
| `get <json> <path>` | Extract value at path | `get '{"a":42}' '.a'` -> 42 |

## Path Syntax

| Syntax | Description | Example |
|--------|-------------|---------|
| `.key` | Object field access | `.name` |
| `[index]` | Array element access | `[0]` |

Paths can be chained: `.users[0].name`

## Examples

```bash
# Get value type
bmb run main.bmb -- type '{"name":"alice"}'
# Output: object

# Get array length
bmb run main.bmb -- length '[1,2,3,4,5]'
# Output: 5

# List object keys
bmb run main.bmb -- keys '{"id":1,"name":"bob","age":30}'
# Output:
# id
# name
# age

# Validate JSON
bmb run main.bmb -- validate '{"valid": true}'
# Output: valid

bmb run main.bmb -- validate '{"incomplete":'
# Output: invalid

# Extract nested value
bmb run main.bmb -- get '{"user":{"name":"alice","scores":[90,85,95]}}' '.user.name'
# Output: alice

# Extract array element
bmb run main.bmb -- get '{"items":["a","b","c"]}' '.items[1]'
# Output: b

# Extract deeply nested value
bmb run main.bmb -- get '{"data":[{"x":10},{"x":20}]}' '.data[1].x'
# Output: 20
```

## Features Demonstrated

1. **Contract-based programming** - Pre/postconditions on parsing functions
2. **Recursive descent parsing** - Position-based JSON traversal
3. **Type detection** - Runtime type identification
4. **Path navigation** - Dot and bracket notation parsing
5. **CLI argument parsing** - Positional argument handling
6. **Tail recursion** - Efficient iteration patterns

## Code Structure

| Section | Description |
|---------|-------------|
| Type Detection | JSON value type identification |
| Parsing | Number, boolean, string, null parsing |
| Nested Skipping | Array/object traversal |
| Array Operations | Length, element access |
| Object Operations | Length, field access, key listing |
| Path Navigation | Dot/bracket path parsing |
| Validation | JSON syntax verification |
| Commands | CLI command implementations |

## Supported JSON Types

- `null` - Null value
- `boolean` - true/false
- `number` - Integer values (no float support yet)
- `string` - Quoted strings
- `array` - Ordered collections
- `object` - Key-value pairs

## Running Tests

```bmb
fn main() -> i64 = run_tests();  // Replace main to run tests
```

Expected output: `777` (type tests), `888` (length tests), `999` (navigation tests)
