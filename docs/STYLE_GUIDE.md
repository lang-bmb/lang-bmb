# BMB Style Guide

> **Version**: v0.64.5
> **Enforced by**: `bmb-fmt` (tools/bmb-fmt/main.bmb)

This document defines the standard code style for BMB projects.

---

## Indentation

- **Use 4 spaces** for indentation
- **No tabs** - use spaces only
- Align continuation lines with the opening delimiter

```bmb
// Good
fn example(a: i64,
           b: i64) -> i64 =
    a + b;

// Bad (tabs)
fn example(a: i64,
	b: i64) -> i64 =
	a + b;
```

---

## Line Length

- **Maximum line length: 100 characters**
- Break long lines at natural boundaries (operators, commas)
- Prefer shorter lines for readability

```bmb
// Good - under 100 chars
fn process(input: String) -> String =
    transform(input);

// Good - long expression broken at operators
let result = some_long_function_name(arg1, arg2) +
    another_function(arg3, arg4);

// Bad - over 100 chars
let result = some_very_long_function_name_that_exceeds_the_limit(argument1, argument2, argument3);
```

---

## Whitespace

### Trailing Whitespace

- **No trailing whitespace** on any line
- Files should end with a single newline

### Blank Lines

- **Two blank lines** between top-level definitions
- **One blank line** between logical sections within functions
- **No multiple consecutive blank lines** (max 2)

```bmb
// Good
fn foo() -> i64 = 42;


fn bar() -> i64 = 43;


// Bad - only one blank line between functions
fn foo() -> i64 = 42;

fn bar() -> i64 = 43;
```

### Spaces Around Operators

- **One space** around binary operators
- **No space** before commas, semicolons
- **One space** after commas, colons

```bmb
// Good
let x = a + b * c;
fn example(a: i64, b: i64) -> i64 = a + b;

// Bad
let x=a+b*c;
let x = a +b* c;
fn example(a:i64,b:i64) -> i64 = a+b;
```

---

## Naming Conventions

### Functions

- **snake_case** for regular functions
- **SCREAMING_CASE** for constant functions (returns compile-time constant)

```bmb
// Regular functions
fn calculate_sum(a: i64, b: i64) -> i64 = a + b;

fn process_input(s: String) -> String = ...;

// Constant functions
fn MAX_SIZE() -> i64 = 1024;

fn BUFFER_CAPACITY() -> i64 = 4096;
```

### Variables

- **snake_case** for all variables
- Use descriptive names (avoid single letters except for indices)

```bmb
// Good
let user_count = 42;
let buffer_size = calculate_size();
for i in 0..n { ... }

// Bad
let uc = 42;
let x = calculate_size();
```

### Types

- **PascalCase** for types (structs, enums, type aliases)

```bmb
// Good
struct UserProfile { ... }
enum MessageType { ... }
type StringList = Array<String>;

// Bad
struct user_profile { ... }
enum message_type { ... }
```

---

## Braces

### Block Expressions

- Opening brace on the same line as the statement
- Closing brace on its own line
- Indent contents by 4 spaces

```bmb
// Good
fn main() -> i64 = {
    let x = 42;
    x
};

// Bad - opening brace on new line
fn main() -> i64 =
{
    let x = 42;
    x
};
```

### Single-Line Expressions

- Prefer single-line for short expressions
- Use braces when needed for clarity

```bmb
// Good - short expressions on one line
fn add(a: i64, b: i64) -> i64 = a + b;

fn is_positive(n: i64) -> bool = n > 0;

// Good - longer expressions use block
fn complex() -> i64 = {
    let a = compute_a();
    let b = compute_b();
    a + b
};
```

---

## If/Else

- Use consistent style for if/else chains
- Short conditions can be single-line
- Complex conditions should be multi-line

```bmb
// Good - short conditions
if x > 0 { x } else { 0 - x };

// Good - complex conditions
if condition {
    long_expression_here()
} else if other_condition {
    another_expression()
} else {
    default_value()
};
```

---

## Comments

### Documentation Comments

- Use `///` for documentation comments
- Place before the item being documented
- Write in complete sentences

```bmb
/// Calculates the factorial of n.
/// Returns 1 for n <= 0.
fn factorial(n: i64) -> i64 =
    if n <= 1 { 1 }
    else { n * factorial(n - 1) };
```

### Regular Comments

- Use `//` for inline comments
- Place above the code being explained
- Keep comments up-to-date with code

```bmb
// Calculate the buffer size based on input length
let buffer_size = input.len() * 2 + HEADER_SIZE();

// Handle edge case for empty input
if input.len() == 0 { return default_value(); };
```

### Section Comments

- Use comment blocks for major sections
- Surround with equal signs for visibility

```bmb
// ============================================================
// String Utilities
// ============================================================

fn trim(s: String) -> String = ...;
fn concat(a: String, b: String) -> String = ...;
```

---

## Contracts

### Preconditions

- Place `pre` before the function body
- Use for input validation

```bmb
fn get_element(arr: Array<i64>, idx: i64) -> i64
    pre idx >= 0 and idx < arr.len()
= arr.at(idx);
```

### Postconditions

- Place `post` after `pre` (if present)
- Use for output guarantees

```bmb
fn abs(n: i64) -> i64
    post result >= 0
= if n < 0 { 0 - n } else { n };
```

---

## Function Organization

- Order functions by dependency (callers before callees when possible)
- Group related functions together
- Place `main()` at the end of the file

```bmb
// Configuration
fn CONFIG_VALUE() -> i64 = 42;

// Utilities (called by main logic)
fn helper() -> i64 = ...;

// Main logic
fn process() -> i64 = helper() + CONFIG_VALUE();

// Entry point
fn main() -> i64 = process();
```

---

## File Organization

1. Module documentation (header comment)
2. Imports (`use` statements)
3. Constants (SCREAMING_CASE functions)
4. Type definitions (structs, enums)
5. Utility functions
6. Main logic functions
7. Entry point (`main`)

```bmb
// Module: Example
// Description: This module demonstrates proper file organization.

// ============================================================
// Constants
// ============================================================

fn MAX_SIZE() -> i64 = 1024;

// ============================================================
// Types
// ============================================================

struct Config {
    size: i64,
    name: String,
}

// ============================================================
// Utilities
// ============================================================

fn helper() -> i64 = ...;

// ============================================================
// Main Logic
// ============================================================

fn process(config: Config) -> i64 = ...;

// ============================================================
// Entry Point
// ============================================================

fn main() -> i64 = {
    let config = Config { size: MAX_SIZE(), name: "example" };
    process(config)
};
```

---

## Running the Formatter

```bash
# Format a file (prints to stdout)
bmb run tools/bmb-fmt/main.bmb <file.bmb>

# Check if file needs formatting (CI mode)
bmb run tools/bmb-fmt/main.bmb --check <file.bmb>
```

---

## Exceptions

These conventions may be relaxed in:

- **Generated code**: Machine-generated code may have different formatting
- **Performance-critical sections**: Where specific layout is required
- **External API compatibility**: When matching external naming conventions

Document any exceptions with a comment explaining the reason.
