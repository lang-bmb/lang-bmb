# BMB Lint Rules

> **Version**: v0.64.6
> **Enforced by**: `bmb-lint` (tools/bmb-lint/main.bmb)

This document describes the lint rules checked by `bmb-lint`.

---

## Rule Categories

| Category | Prefix | Description |
|----------|--------|-------------|
| **Warning** | W | Potential issues that should be addressed |
| **Info** | I | Informational notes (not errors) |

---

## Warning Rules

### W001: Line Too Long

**Severity**: Warning
**Default Threshold**: 100 characters

Lines exceeding the maximum length are harder to read and may cause horizontal scrolling.

```bmb
// Bad - W001
let very_long_variable_name = some_function_with_long_name(argument_one, argument_two, argument_three);

// Good - split across lines
let very_long_variable_name = some_function_with_long_name(
    argument_one,
    argument_two,
    argument_three
);
```

**Configuration**: The default threshold is 100 characters.

---

### W002: Trailing Whitespace

**Severity**: Warning

Trailing whitespace is invisible and serves no purpose. It can cause issues with version control diffs.

```bmb
// Bad - W002 (spaces at end of line)
let x = 42;

// Good
let x = 42;
```

**Fix**: Remove all whitespace at the end of lines.

---

### W003: FIXME Comment

**Severity**: Warning

FIXME comments indicate known issues that should be resolved before release.

```bmb
// Bad - W003
// FIXME: This crashes on empty input
fn process(s: String) -> String = s;

// Good - fix the issue or document as a known limitation
fn process(s: String) -> String =
    if s.len() == 0 { "" } else { s };
```

**Fix**: Address the FIXME or convert to a documented limitation.

---

### W004: XXX Comment

**Severity**: Warning

XXX comments typically mark code that needs attention or review.

```bmb
// Bad - W004
// XXX: Is this correct?
fn calculate(n: i64) -> i64 = n * 2;

// Good - resolve the question
fn calculate(n: i64) -> i64 = n * 2;  // Doubles the input
```

**Fix**: Resolve the concern and remove the XXX marker.

---

### W005: Tab Character

**Severity**: Warning

Tab characters cause inconsistent display across editors. Use spaces instead.

```bmb
// Bad - W005 (tab used for indentation)
fn foo() -> i64 = {
	let x = 42;  // tab used here
	x
};

// Good - use 4 spaces
fn foo() -> i64 = {
    let x = 42;
    x
};
```

**Fix**: Replace tabs with 4 spaces.

---

## Info Rules

### I001: TODO Comment

**Severity**: Info

TODO comments are reminders for future work. They're informational, not errors.

```bmb
// I001 - informational
// TODO: Add support for negative numbers
fn abs(n: i64) -> i64 = n;  // Current implementation is incomplete
```

**Guidance**: TODOs are acceptable during development but should be tracked and eventually resolved.

---

### I002: Possible Magic Number

**Severity**: Info

Large numeric literals (3+ digits) may be "magic numbers" that should be named constants.

```bmb
// I002 - possible magic number
fn get_buffer_size() -> i64 = 4096;

// Better - named constant
fn BUFFER_SIZE() -> i64 = 4096;

fn get_buffer_size() -> i64 = BUFFER_SIZE();
```

**Exceptions**:
- Constants defined in constant functions (fn NAME() -> i64 = N)
- Common values (0, 1, 10, 100)
- Bit masks and sizes

---

## Built-in Compiler Warnings

In addition to `bmb-lint` rules, the BMB compiler (`bmb check`) reports:

| Code | Description |
|------|-------------|
| `unused_binding` | Variable declared but never used |
| `unused_function` | Function defined but never called |
| `missing_postcondition` | Function lacks a postcondition |

---

## Running the Linter

```bash
# Lint a file (text output)
bmb run tools/bmb-lint/main.bmb <file.bmb>

# Lint a file (JSON output for tools)
bmb run tools/bmb-lint/main.bmb --json <file.bmb>
```

---

## Output Format

### Text Output

```
Linting: file.bmb

W001|42|line too long (105 > 100)
W002|67|trailing whitespace
I001|89|TODO comment found
```

Format: `<code>|<line>|<message>`

### JSON Output

```json
[{"warning":"W001|42|line too long (105 > 100)"},{"warning":"W002|67|trailing whitespace"}]
```

---

## Suppressing Warnings

Currently, `bmb-lint` does not support inline suppression. All warnings are reported.

**Workaround**: Filter output in CI scripts if needed.

---

## CI Integration

For CI pipelines, use the exit code to determine success:

```bash
# Returns 0 if no warnings, 1 if any warnings found
bmb run tools/bmb-lint/main.bmb file.bmb > /dev/null && echo "Passed" || echo "Warnings found"
```

---

## Recommended Practices

### Pre-commit Hook

Run `bmb-lint` before committing:

```bash
#!/bin/bash
# .git/hooks/pre-commit
bmb run tools/bmb-lint/main.bmb --json changed_files.bmb
```

### CI Pipeline

Include lint checks in CI:

```yaml
- name: Lint Check
  run: bmb run tools/bmb-lint/main.bmb --json src/*.bmb
```

---

## Future Rules

Planned rules for future versions:

| Rule | Description |
|------|-------------|
| W010 | Missing function documentation |
| W011 | Complex function (cyclomatic complexity) |
| W012 | Deeply nested code |
| W013 | Unused imports |
| I010 | Function exceeds recommended length |
