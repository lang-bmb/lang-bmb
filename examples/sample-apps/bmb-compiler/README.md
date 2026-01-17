# bmb-compiler

A mini expression language compiler demonstrating compiler construction in BMB.

## Usage

```bash
bmb run main.bmb -- "<expression>"
```

## Language Features

### Literals
- **Numbers**: `42`, `-17`, `0`

### Operators

| Category | Operators | Precedence |
|----------|-----------|------------|
| Comparison | `==`, `!=`, `<`, `>`, `<=`, `>=` | Lowest |
| Additive | `+`, `-` | Medium |
| Multiplicative | `*`, `/`, `%` | High |
| Unary | `-` (negation) | Highest |

### Grouping
- **Parentheses**: `(expr)` for explicit grouping

### Conditionals
- **If-then-else**: `if cond then expr1 else expr2`

## Examples

```bash
# Basic arithmetic
bmb run main.bmb -- "2 + 3"
# Output: 5

# Operator precedence
bmb run main.bmb -- "2 + 3 * 4"
# Output: 14

# Parentheses
bmb run main.bmb -- "(2 + 3) * 4"
# Output: 20

# Comparison
bmb run main.bmb -- "10 > 5"
# Output: 1

bmb run main.bmb -- "3 == 4"
# Output: 0

# Modulo
bmb run main.bmb -- "17 % 5"
# Output: 2

# Negation
bmb run main.bmb -- "-42 + 50"
# Output: 8

# Conditional
bmb run main.bmb -- "if 5 > 3 then 100 else 0"
# Output: 100

bmb run main.bmb -- "if 2 > 7 then 1 else -1"
# Output: -1
```

## Compiler Architecture

```
Source Code
    │
    ▼
┌─────────┐     Tokenizes input into tokens
│  Lexer  │     (numbers, operators, keywords)
└────┬────┘
     │
     ▼
┌─────────┐     Builds Abstract Syntax Tree
│ Parser  │     (recursive descent)
└────┬────┘
     │
     ▼
┌───────────┐   Evaluates AST to produce result
│ Evaluator │   (tree-walking interpreter)
└───────────┘
     │
     ▼
  Result
```

## Implementation Details

### Token Encoding

Tokens are encoded as single integers:
```
token = type * 1000000 + value * 1000 + end_pos
```

### AST Node Encoding

AST nodes are encoded as:
```
binary_node = op * 10000000 + left * 1000 + right
literal_node = type * 10000000 + value
```

### Parser Result

Parser functions return packed results:
```
result = value * 1000 + new_position
```

## Features Demonstrated

1. **Lexer Implementation**
   - Token types and encoding
   - Whitespace skipping
   - Multi-character operators (`==`, `<=`, etc.)
   - Keyword recognition (`if`, `then`, `else`)

2. **Recursive Descent Parser**
   - Operator precedence handling
   - Left-associativity
   - Parenthesized expressions
   - Error handling

3. **Tree-Walking Evaluator**
   - Binary operations
   - Comparison operators
   - Conditional expressions
   - Division by zero handling

4. **BMB Language Features**
   - Tail-recursive functions
   - Pattern matching via conditionals
   - Contract-based design
   - Functional programming style

## Code Structure

| Component | Functions |
|-----------|-----------|
| **Token Types** | `tok_*()` constants |
| **Lexer** | `next_token()`, `parse_number()`, `parse_ident()` |
| **AST Nodes** | `node_*()` constants, `make_*()` constructors |
| **Parser** | `parse_expr()`, `parse_comparison()`, etc. |
| **Evaluator** | `eval_node()` |
| **CLI** | `main()`, `show_usage()` |

## Running Tests

```bmb
fn main() -> i64 = run_tests();  // Replace main to run tests
```

Expected output: `777` (lexer tests), `888` (parser tests), `999` (evaluator tests)

## Limitations

- Variables are recognized but not fully implemented
- Only integer arithmetic (no floating point)
- No user-defined functions
- Limited error messages

## Extension Ideas

1. Add variable bindings with `let x = expr; body`
2. Implement bytecode compilation
3. Add function definitions
4. Support floating-point numbers
5. Add more operators (bitwise, logical)
