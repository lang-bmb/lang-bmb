# bmb-calc

A simple command-line calculator demonstrating BMB language features.

## Usage

```bash
bmb run main.bmb -- "<command> <args>"
```

## Commands

### Arithmetic (Binary)
| Command | Description | Example |
|---------|-------------|---------|
| `add a b` | Addition | `add 5 3` → 8 |
| `sub a b` | Subtraction | `sub 10 4` → 6 |
| `mul a b` | Multiplication | `mul 6 7` → 42 |
| `div a b` | Division | `div 20 4` → 5 |
| `mod a b` | Modulo | `mod 17 5` → 2 |
| `pow a b` | Power | `pow 2 10` → 1024 |

### Math Functions (Unary)
| Command | Description | Example |
|---------|-------------|---------|
| `sqrt n` | Square root | `sqrt 16` → 4 |
| `abs n` | Absolute value | `abs -5` → 5 |
| `fac n` | Factorial | `fac 5` → 120 |
| `fib n` | Fibonacci | `fib 10` → 55 |
| `prime n` | Is prime? (1=yes) | `prime 17` → 1 |

### Comparison (Binary)
| Command | Description | Example |
|---------|-------------|---------|
| `min a b` | Minimum | `min 3 7` → 3 |
| `max a b` | Maximum | `max 3 7` → 7 |
| `gcd a b` | Greatest common divisor | `gcd 48 18` → 6 |
| `lcm a b` | Least common multiple | `lcm 4 6` → 12 |

## Examples

```bash
# Basic arithmetic
bmb run main.bmb -- "add 100 200"
# Output: 300

# Power calculation
bmb run main.bmb -- "pow 2 8"
# Output: 256

# Check if prime
bmb run main.bmb -- "prime 97"
# Output: 1

# Get help
bmb run main.bmb -- "help"
```

## Features Demonstrated

1. **Contract-based programming** - Pre/postconditions on math functions
2. **Expression-based syntax** - Everything returns a value
3. **Tail-recursive algorithms** - Efficient iteration via recursion
4. **String parsing** - Manual parsing without regex
5. **CLI argument handling** - Using `arg_count()` and `get_arg()`

## Running Tests

The file includes unit tests that can be invoked by calling `run_tests()`:

```bmb
fn main() -> i64 = run_tests();  // Replace main to run tests
```

Expected output: `777` (math tests), `888` (parse tests), `999` (command tests)
