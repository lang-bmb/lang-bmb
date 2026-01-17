# Contract-Based Verification in BMB

> How BMB's contract system eliminates entire classes of bugs at compile time

## Overview

BMB's contract system provides static verification of program correctness through preconditions, postconditions, and invariants. Unlike runtime assertions, these contracts are verified at compile time using SMT solvers (Z3), catching bugs before the code ever runs.

## Core Concepts

### Preconditions (`pre`)

Preconditions specify what must be true before a function executes:

```bmb
fn divide(a: i64, b: i64) -> i64
  pre b != 0  // Caller must ensure b is never zero
= a / b;

fn sqrt(n: i64) -> i64
  pre n >= 0  // Cannot compute sqrt of negative number
= sqrt_impl(n);
```

The compiler verifies at every call site that the precondition is satisfied.

### Postconditions (`post`)

Postconditions specify what the function guarantees upon return:

```bmb
fn abs(x: i64) -> i64
  post ret >= 0  // Result is always non-negative
= if x < 0 { 0 - x } else { x };

fn max(a: i64, b: i64) -> i64
  post ret >= a and ret >= b  // Result is at least as large as both inputs
= if a > b { a } else { b };
```

The compiler verifies that the function body actually establishes the postcondition.

### Refinement Types (`where`)

Types can be refined with predicates:

```bmb
type NonZero = i64 where self != 0;
type Positive = i64 where self > 0;
type Percentage = i64 where self >= 0 and self <= 100;

fn safe_divide(a: i64, b: NonZero) -> i64 = a / b;  // Division is always safe
```

## Real-World Examples

### Array Bounds Checking

```bmb
fn vec_get(v: Vec<i64>, idx: i64) -> i64
  pre idx >= 0
  pre idx < vec_len(v)
= vec_get_unchecked(v, idx);

fn safe_access(data: Vec<i64>, i: i64) -> i64
  pre i >= 0
  pre i < vec_len(data)
= {
    // Compiler proves bounds are satisfied
    let value = vec_get(data, i);
    value * 2
};
```

### Resource Management

```bmb
fn file_read(handle: i64) -> String
  pre handle != 0  // Must be valid handle
= file_read_impl(handle);

fn process_file(path: String) -> i64 = {
    let handle = file_open(path);
    if handle == 0 {
        0 - 1  // Error: file not found
    } else {
        let content = file_read(handle);  // Verified: handle != 0
        let result = process(content);
        let closed = file_close(handle);
        result
    }
};
```

### State Machine Verification

```bmb
// Connection states as type aliases
type Disconnected = i64 where self == 0;
type Connected = i64 where self == 1;
type Authenticated = i64 where self == 2;

fn connect(state: Disconnected) -> Connected = 1;
fn authenticate(state: Connected, token: String) -> Authenticated = 2;
fn send_message(state: Authenticated, msg: String) -> i64 = {
    // Can only send when authenticated - verified at compile time
    send_impl(msg)
};
```

## Contract Composition

Contracts compose naturally through function calls:

```bmb
fn clamp(x: i64, lo: i64, hi: i64) -> i64
  pre lo <= hi
  post ret >= lo and ret <= hi
= max(lo, min(hi, x));

fn normalize(values: Vec<i64>) -> Vec<i64>
  pre vec_len(values) > 0
  post vec_len(ret) == vec_len(values)
= {
    let min_val = find_min(values);
    let max_val = find_max(values);
    let range = max_val - min_val;
    if range == 0 { values }
    else { map_normalize(values, min_val, range) }
};
```

## Trust Annotations

For interfacing with external code or performance-critical sections:

```bmb
@trust  // Skip verification for this function
fn ffi_call(ptr: i64) -> i64 = external_function(ptr);

@check  // Force runtime check even if statically verified
fn debug_validate(x: i64) -> i64
  pre x > 0
= x;
```

## Verification Workflow

```bash
# Type check without verification
bmb check file.bmb

# Full verification with Z3
bmb verify file.bmb

# Show verification details
bmb verify file.bmb --verbose

# List unverified contracts
bmb q fn --unverified
```

## Benefits Over Traditional Approaches

| Approach | When Checked | Coverage | Performance |
|----------|--------------|----------|-------------|
| Unit Tests | Runtime | Sample cases | Test overhead |
| Assertions | Runtime | All executions | Runtime overhead |
| BMB Contracts | Compile time | All possible inputs | Zero runtime cost |

## Common Patterns

### Option Type Contracts

```bmb
fn unwrap_or(opt: Option<i64>, default: i64) -> i64 =
    match opt {
        Some(v) => v,
        None => default
    };

fn unwrap(opt: Option<i64>) -> i64
  pre opt != None  // Caller must ensure Some
= match opt {
    Some(v) => v,
    None => 0  // Unreachable, but type-safe
};
```

### Loop Invariants

```bmb
fn sum_to_n(n: i64) -> i64
  pre n >= 0
  post ret == n * (n + 1) / 2
= sum_iter(n, 0, 0);

fn sum_iter(n: i64, i: i64, acc: i64) -> i64
  pre i >= 0 and i <= n + 1
  pre acc == i * (i - 1) / 2  // Loop invariant
= if i > n { acc } else { sum_iter(n, i + 1, acc + i) };
```

## Integration with AI Development

BMB contracts provide explicit specifications that AI code generators can target:

```bmb
// Specification for AI to implement
fn binary_search(arr: Vec<i64>, target: i64) -> i64
  pre is_sorted(arr)
  post ret == -1 or (ret >= 0 and ret < vec_len(arr) and vec_get(arr, ret) == target)
= /* AI generates implementation */;
```

The compiler then verifies the AI-generated code meets the specification.

## Next Steps

- [Systems Programming](SYSTEMS.md) - Memory safety with contracts
- [Performance](PERFORMANCE.md) - Zero-cost contracts
- [From Rust](FROM_RUST.md) - Migration guide
