# Contract Programming in BMB

> A comprehensive guide to BMB's compile-time contract verification system.

## Table of Contents

1. [What Are Contracts?](#what-are-contracts)
2. [Preconditions](#preconditions)
3. [Postconditions](#postconditions)
4. [Inline Refinement Types](#inline-refinement-types)
5. [Named Contracts](#named-contracts)
6. [Function Attributes](#function-attributes)
7. [Verification Process](#verification-process)
8. [Advanced Patterns](#advanced-patterns)
9. [Common Patterns](#common-patterns)
10. [Debugging Contract Failures](#debugging-contract-failures)

---

## What Are Contracts?

Contracts are **compile-time guarantees** about your code. Unlike runtime assertions that crash when violated, BMB contracts are verified by an SMT solver (Z3) before your program runs.

### The Contract Advantage

| Approach | When Checked | Failure Mode | Performance |
|----------|--------------|--------------|-------------|
| Runtime assertions | Execution | Crash/panic | Runtime cost |
| Unit tests | Test time | Test failure | No production cost |
| **BMB Contracts** | **Compile time** | **Won't compile** | **Zero runtime cost** |

### Basic Example

```bmb
-- This function has a contract: b must not be zero
fn divide(a: i64, b: i64) -> i64
  pre b != 0
= a / b;

fn main() -> i64 =
    divide(10, 2);   -- OK: compiler proves 2 != 0
    -- divide(10, 0); -- ERROR: compiler rejects this call
```

---

## Preconditions

Preconditions (`pre`) specify what must be true **before** a function executes. The compiler verifies that all callers satisfy these conditions.

### Basic Syntax

```bmb
fn function_name(params) -> ReturnType
  pre condition1
  pre condition2
= body;
```

### Examples

```bmb
-- Single precondition
fn sqrt(x: i64) -> i64
  pre x >= 0
= -- implementation;

-- Multiple preconditions
fn array_get(arr: [i64; 8], index: i64) -> i64
  pre index >= 0
  pre index < 8
= arr[index];

-- Complex preconditions
fn binary_search(arr: [i64; 8], len: i64, target: i64) -> i64
  pre len > 0
  pre len <= 8
  pre is_sorted_asc(arr, len)
= -- implementation;
```

### What Preconditions Can Reference

- Function parameters
- Global constants
- Pure functions (functions without side effects)
- Logical operators: `and`, `or`, `not`
- Comparisons: `==`, `!=`, `<`, `<=`, `>`, `>=`
- Arithmetic: `+`, `-`, `*`, `/`, `%`

---

## Postconditions

Postconditions (`post`) specify what will be true **after** a function returns. The compiler verifies that the function body satisfies these guarantees.

### Basic Syntax

```bmb
fn function_name(params) -> ReturnType
  post condition_using_ret
= body;
```

The keyword `ret` refers to the return value.

### Examples

```bmb
-- Return value constraint
fn abs(x: i64) -> i64
  post ret >= 0
= if x >= 0 then x else 0 - x;

-- Multiple postconditions
fn clamp(x: i64, lo: i64, hi: i64) -> i64
  pre lo <= hi
  post ret >= lo
  post ret <= hi
= if x < lo then lo
  else if x > hi then hi
  else x;

-- Relationship postconditions
fn max(a: i64, b: i64) -> i64
  post ret >= a
  post ret >= b
  post ret == a or ret == b
= if a >= b then a else b;
```

### Postconditions with `old`

Use `old(expr)` to refer to a value at function entry (before any modifications):

```bmb
fn increment(counter: &mut i64) -> ()
  post *counter == old(*counter) + 1
= *counter = *counter + 1;
```

---

## Inline Refinement Types

BMB v0.2 introduced inline refinement types—constraints directly in type annotations using the `it` keyword.

### Basic Syntax

```bmb
-- Parameter constraint
fn divide(a: i64, b: i64{it != 0}) -> i64 = a / b;

-- Return constraint
fn abs(x: i64) -> i64{it >= 0} =
    if x >= 0 then x else 0 - x;

-- Multiple constraints
fn clamp(x: i64, lo: i64, hi: i64{it >= lo}) -> i64{it >= lo, it <= hi} =
    if x < lo then lo else if x > hi then hi else x;
```

### The `it` Keyword

`it` refers to the value being constrained:
- In parameters: `it` is the parameter value
- In return types: `it` is the return value

```bmb
-- it = b (parameter)
fn safe_divide(a: i64, b: i64{it != 0}) -> i64 = a / b;

-- it = return value
fn positive_only(x: i64) -> i64{it > 0} =
    if x > 0 then x else 1;
```

### Inline vs Classic Syntax

```bmb
-- Classic syntax (v0.1)
fn divide_classic(a: i64, b: i64) -> i64
  pre b != 0
= a / b;

-- Inline syntax (v0.2) - preferred
fn divide_inline(a: i64, b: i64{it != 0}) -> i64 = a / b;

-- Both are equivalent; inline is more concise
```

---

## Named Contracts

For complex constraints, use named contracts in `where` blocks. Names appear in error messages, making debugging easier.

### Basic Syntax

```bmb
fn function_name(params) -> r: ReturnType
  where {
    constraint_name1: condition1,
    constraint_name2: condition2
  }
= body;
```

Note: Named return (`r: ReturnType`) lets you reference the return value by name.

### Examples

```bmb
-- Named postconditions
fn min(a: i64, b: i64) -> r: i64
  where {
    less_or_equal: r <= a and r <= b,
    is_input: r == a or r == b
  }
= if a <= b then a else b;

-- Complex algorithm contracts
fn binary_search(arr: [i64; 8], len: i64, target: i64) -> r: i64
  where {
    valid_length: len > 0 and len <= 8,
    array_sorted: is_sorted_asc(arr, len),
    result_bounds: r == -1 or (r >= 0 and r < len),
    found_correct: r >= 0 implies arr[r] == target,
    not_found_correct: r == -1 implies not contains_i64(arr, len, target)
  }
= -- implementation;
```

### Error Messages

Named constraints produce clear error messages:

```
Error: Contract violation in `binary_search`
  - `array_sorted`: Could not prove is_sorted_asc(arr, len)
  at main.bmb:42:5
```

---

## Function Attributes

BMB provides attributes that affect verification and optimization.

### @pure

Marks a function as having no side effects. Pure functions can be used in contracts.

```bmb
@pure
fn square(x: i64) -> i64{it >= 0} = x * x;

@pure
fn is_even(n: i64) -> bool = n % 2 == 0;

-- Pure functions can be used in contracts
fn process_even(n: i64) -> i64
  pre is_even(n)
= n / 2;
```

### @decreases

Specifies a decreasing measure for recursive functions, proving termination.

```bmb
@decreases(n)
fn factorial(n: i64{it >= 0}) -> i64{it >= 1} =
    if n <= 1 then 1
    else n * factorial(n - 1);

@decreases(b)
fn power(base: i64, b: i64{it >= 0}) -> i64 =
    if b == 0 then 1
    else base * power(base, b - 1);
```

### @trust

Tells the compiler to trust the function's contracts without verification. Use sparingly.

```bmb
@trust
fn unsafe_array_access(arr: [i64; 8], index: i64) -> i64
  pre index >= 0
  pre index < 8
= arr[index];  -- Compiler trusts this is safe
```

### @inline

Suggests the compiler inline the function, which can help verification.

```bmb
@inline
fn is_valid_index(len: i64, idx: i64) -> bool =
    idx >= 0 and idx < len;
```

---

## Verification Process

### How BMB Verifies Contracts

1. **Parse contracts** into logical formulas
2. **Translate to SMT-LIB2** format
3. **Send to Z3** SMT solver
4. **Analyze result**:
   - `sat` (satisfiable): Contract might be violated → compile error
   - `unsat` (unsatisfiable): Contract is always satisfied → success
   - `unknown`/timeout: → warning or error depending on configuration

### Example Translation

```bmb
fn abs(x: i64) -> i64
  post ret >= 0
= if x >= 0 then x else 0 - x;
```

Becomes (simplified SMT-LIB2):

```smt
(declare-const x Int)
(declare-const ret Int)

; Function body
(assert (= ret (ite (>= x 0) x (- 0 x))))

; Negation of postcondition (we want UNSAT)
(assert (not (>= ret 0)))

(check-sat)  ; Returns UNSAT → contract satisfied
```

### Verification Commands

```bash
# Verify a single file
bmb verify program.bmb

# Verify with verbose output
bmb verify --verbose program.bmb

# Type check only (no SMT verification)
bmb check program.bmb
```

---

## Advanced Patterns

### Quantifiers

Use `forall` and `exists` for array/collection contracts:

```bmb
fn all_positive(arr: [i64; 8], len: i64) -> bool
  pre len >= 0 and len <= 8
  post ret == forall(i in 0..len): arr[i] > 0
= -- implementation;

fn contains(arr: [i64; 8], len: i64, target: i64) -> bool
  pre len >= 0 and len <= 8
  post ret == exists(i in 0..len): arr[i] == target
= -- implementation;
```

### Implication

Use `implies` (or `=>`) for conditional contracts:

```bmb
fn safe_divide(a: i64, b: i64) -> i64
  post b != 0 implies ret == a / b
  post b == 0 implies ret == 0
= if b != 0 then a / b else 0;
```

### State Contracts with `.pre` and `.post`

Reference pre-state and post-state for mutable operations:

```bmb
fn swap(a: &mut i64, b: &mut i64) -> ()
  where {
    a_gets_b: a.post == b.pre,
    b_gets_a: b.post == a.pre
  }
=
    let temp = *a;
    *a = *b;
    *b = temp;
```

### Loop Invariants

Use `@invariant` for loop verification:

```bmb
fn sum_array(arr: [i64; 8], len: i64) -> i64
  pre len >= 0 and len <= 8
=
    var total = 0;
    var i = 0;
    @invariant(i >= 0 and i <= len)
    @invariant(total == sum_range(arr, 0, i))
    while i < len {
        total = total + arr[i];
        i = i + 1;
    };
    total;
```

---

## Common Patterns

### Non-Empty Collections

```bmb
fn first(arr: [i64; 8], len: i64{it > 0}) -> i64
  pre len <= 8
= arr[0];

fn last(arr: [i64; 8], len: i64{it > 0, it <= 8}) -> i64 =
    arr[len - 1];
```

### Bounded Values

```bmb
fn percentage(value: i64{it >= 0, it <= 100}) -> String =
    int_to_string(value) + "%";

fn grade(score: i64{it >= 0, it <= 100}) -> String =
    if score >= 90 then "A"
    else if score >= 80 then "B"
    else if score >= 70 then "C"
    else if score >= 60 then "D"
    else "F";
```

### Option Handling

```bmb
fn unwrap(opt: Option) -> i64
  pre is_some(opt)
= match opt {
    Option::Some(v) => v,
    Option::None => 0  -- Never reached due to precondition
};

fn map_option(opt: Option, f: fn(i64) -> i64) -> Option =
    match opt {
        Option::Some(v) => Option::Some(f(v)),
        Option::None => Option::None
    };
```

### Result Chaining

```bmb
fn try_divide(a: i64, b: i64) -> Result =
    if b == 0 then err(ERR_DIVIDE_BY_ZERO())
    else ok(a / b);

fn safe_compute(x: i64, y: i64{it != 0}) -> i64 =
    let result = try_divide(x, y);
    match result {
        Result::Ok(v) => v,
        Result::Err(_) => 0  -- Never reached due to y != 0
    };
```

---

## Debugging Contract Failures

### Reading Error Messages

```
Error[E0401]: Contract verification failed
  --> src/math.bmb:15:1
   |
15 | fn divide(a: i64, b: i64) -> i64
   |    ^^^^^^ precondition `b != 0` not satisfied
   |
   = note: Call site at main.bmb:42:5
   = note: Z3 found counterexample: b = 0
   = help: Ensure all callers pass non-zero second argument
```

### Common Causes

1. **Missing precondition at call site**
   ```bmb
   fn caller(x: i64) -> i64 =
       divide(10, x);  -- ERROR: x might be 0

   -- Fix: Add constraint
   fn caller(x: i64{it != 0}) -> i64 =
       divide(10, x);  -- OK
   ```

2. **Postcondition not established**
   ```bmb
   fn abs(x: i64) -> i64
     post ret >= 0
   = x;  -- ERROR: x might be negative

   -- Fix: Handle negative case
   fn abs(x: i64) -> i64
     post ret >= 0
   = if x >= 0 then x else 0 - x;  -- OK
   ```

3. **Inconsistent contracts**
   ```bmb
   fn impossible(x: i64) -> i64
     pre x > 0
     pre x < 0  -- ERROR: No x satisfies both
   = x;
   ```

### Debugging Tips

1. **Start simple**: Begin with minimal contracts, add gradually
2. **Check coverage**: Ensure all code paths satisfy postconditions
3. **Use named contracts**: Clear names help identify failures
4. **Verify incrementally**: Run `bmb verify` frequently during development
5. **Read counterexamples**: Z3 provides values that violate contracts

---

## Best Practices

### Do's

- ✅ Start with type-level constraints (refinement types)
- ✅ Use named contracts for complex conditions
- ✅ Keep contracts as simple as possible
- ✅ Document why contracts exist, not what they check
- ✅ Verify frequently during development

### Don'ts

- ❌ Don't use `@trust` unless absolutely necessary
- ❌ Don't write contracts that duplicate type information
- ❌ Don't make contracts so complex Z3 times out
- ❌ Don't ignore verification warnings

### Contract Design Guidelines

1. **Express intent**: Contracts should express what the function needs, not how it works
2. **Minimal sufficient**: Include only necessary constraints
3. **Caller-friendly**: Make preconditions easy for callers to satisfy
4. **Self-documenting**: Good contracts explain the function's requirements

---

## Summary

BMB's contract system provides:

| Feature | Benefit |
|---------|---------|
| Preconditions | Guarantee valid inputs at compile time |
| Postconditions | Guarantee correct outputs at compile time |
| Refinement types | Concise inline constraints |
| Named contracts | Clear error messages |
| SMT verification | Mathematical proof of correctness |
| Zero runtime cost | No performance penalty |

With contracts, bugs are caught at compile time, not in production.

---

*Next: [Language Reference](./LANGUAGE_REFERENCE.md)*
