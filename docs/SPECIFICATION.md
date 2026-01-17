# BMB Language Specification

**Version**: v0.32
**Date**: 2026-01-10
**Status**: Final Draft

---
## 1. Design Principles

### 1.1 Priority

| Priority | Principle | Description |
|----------|-----------|-------------|
| **P0** | **Performance** | No syntax that prevents optimization. Contracts enable check elimination. |
| **P0** | **Correctness** | No implicit/ambiguous behavior. Same syntax = same meaning. |
| **P1** | **LLM Efficiency** | Maximize code generation accuracy through universal conventions. |

### 1.2 P0 Rules (Non-negotiable)

| Rule | Description | Violation Example |
|------|-------------|-------------------|
| Compile-time verification | Provable at compile time → Must be enforced | Bounds check without contract |
| Explicit behavior | No hidden conversions or control flow | Deref coercion, `?` operator |
| Unambiguous parsing | Same token = Same meaning | N/A (satisfied by Rust base) |
| Single representation | One concept = One syntax | `T?` and `Option<T>` both allowed |

### 1.3 P1 Rules (Balanced)

| Rule | Description | Application |
|------|-------------|-------------|
| Universal over Rust-specific | Prefer widely adopted conventions | `T?` over `Option<T>` |
| Rust when universal | Use Rust syntax when it's the standard | `<T>`, `match`, `&&` |
| Modern over historical | Prefer current standards over legacy | `T?` (2011+) over `Option<T>` (2010) |
| LLM data coverage | Consider training data availability | `Result<T,E>` (rich Rust data) |

---

## 2. Lexical Structure

### 2.1 Comments

```bmb
// Single line comment

/*
   Block comment
   Nesting allowed: /* nested */
*/
```

### 2.2 Keywords

| Category | Keywords |
|----------|----------|
| Declarations | `fn`, `let`, `mut`, `type`, `struct`, `enum`, `trait`, `impl`, `mod`, `use`, `pub` |
| Contracts | `pre`, `post`, `invariant`, `where`, `pure`, `trust` |
| Control | `if`, `else`, `match`, `while`, `for`, `in`, `loop`, `return`, `break`, `continue` |
| Logical | `and`, `or`, `not` |
| Values | `true`, `false`, `None`, `Some`, `Ok`, `Err` |
| Types | `Self`, `self` |
| Special | `as`, `move`, `todo` |

### 2.3 Operators

| Category | Operators | Notes |
|----------|-----------|-------|
| Arithmetic | `+` `-` `*` `/` `%` | Contract required for overflow safety |
| Overflow | `+%` `-%` `*%` | Wrapping (mod 2^n) |
| Overflow | `+\|` `-\|` `*\|` | Saturating (clamp to bounds) |
| Overflow | `+?` `-?` `*?` | Checked (returns `T?`) |
| Comparison | `==` `!=` `<` `>` `<=` `>=` | |
| Logical | `&&` `\|\|` `!` | Symbolic form |
| Logical | `and` `or` `not` | Keyword form (equivalent) |
| Bitwise | `band` `bor` `bxor` `bnot` | **Distinct from `&`/`\|`** |
| Shift | `<<` `>>` | |
| Reference | `&` `&mut` `*` | Unambiguous (prefix only) |
| Other | `=` `->` `=>` `::` `.` `,` `;` `:` `?` | `?` is type suffix only |

### 2.4 Operator Design Rationale

**Bitwise operators use keywords (`band`/`bor`/`bxor`/`bnot`)**:
- P0-Correct: `&` is reference operator, `|` is used in patterns
- No context-dependent parsing
- Clear distinction: `a band b` (bitwise) vs `&a` (reference)

**Logical operators allow both forms**:
- `&&`/`||`/`!` for Rust compatibility
- `and`/`or`/`not` for contract readability
- No ambiguity: both are binary/unary operators

---

## 3. Type System

### 3.1 Primitive Types

| Type | Description |
|------|-------------|
| `i8`, `i16`, `i32`, `i64`, `i128` | Signed integers |
| `u8`, `u16`, `u32`, `u64`, `u128` | Unsigned integers |
| `isize`, `usize` | Pointer-sized integers |
| `f32`, `f64` | IEEE 754 floating point |
| `bool` | Boolean (`true`, `false`) |
| `char` | Unicode scalar value |
| `()` | Unit type |

### 3.2 Compound Types

| Type | Syntax | Description |
|------|--------|-------------|
| Array | `[T; N]` | Fixed-size array |
| Slice | `&[T]` | Dynamically-sized view |
| Tuple | `(T, U, V)` | Heterogeneous fixed-size |
| Reference | `&T`, `&mut T` | Immutable/mutable borrow |
| Pointer | `*const T`, `*mut T` | Raw pointers |

### 3.3 Nullable Types

**Decision**: `T?` syntax only (single representation)

```bmb
// Nullable type
let x: i32? = Some(42);
let y: i32? = None;

// Non-null (default)
let z: i32 = 42;
```

**Methods**:
```bmb
let x: i32? = Some(42);

x.is_some()       // bool
x.is_none()       // bool
x.unwrap()        // i32 (requires pre x.is_some())
x.unwrap_or(0)    // i32
x.map(|v| v + 1)  // i32?
```

**Rationale**:
- P0-Correct: One concept = One syntax (no `Option<T>` alias)
- P1: `T?` is universal (Kotlin, Swift, TypeScript, C#, Dart)
- FFI: `T?` maps to Rust `Option<T>` at boundary

### 3.4 Result Type

**Decision**: `Result<T, E>` (Rust compatible)

```bmb
fn parse(s: &str) -> Result<i32, ParseError> {
    // ...
}
```

### 3.5 Generics

```bmb
fn max<T: Ord>(a: T, b: T) -> T
  post ret >= a and ret >= b
= if a > b { a } else { b };

struct Pair<T, U> {
    first: T,
    second: U,
}

enum Result<T, E> {
    Ok(T),
    Err(E),
}
```

### 3.6 Refinement Types

```bmb
type NonZero = i64 where self != 0;
type Percentage = f64 where self >= 0.0 and self <= 100.0;
type NonEmpty<T> = [T] where self.len() > 0;
```

### 3.7 Lifetimes

```bmb
// Single input reference: automatic
fn first(arr: &[i32]) -> &i32
  pre arr.len() > 0
= &arr[0];

// Multiple input references: explicit required
fn longer<'a>(x: &'a str, y: &str) -> &'a str = x;
```

---

## 4. Functions

### 4.1 Declaration

```bmb
// Expression body: entire expression is return value
fn add(a: i32, b: i32) -> i32 = a + b;

// Block body: explicit return required
fn add(a: i32, b: i32) -> i32 {
    return a + b;
}

// Expression body with control flow
fn abs(x: i32) -> i32 = if x >= 0 { x } else { -x };
```

### 4.2 Explicit Return Rule

**P0-Correct**: `return` required in block bodies `{}`

```bmb
// ✓ Correct
fn foo() -> i32 {
    return 42;
}

// ✓ Correct (expression body)
fn bar() -> i32 = 42;

// ✗ Error: missing return
fn baz() -> i32 {
    42
}
```

**Rationale**: Semicolon should not silently change return type.

### 4.3 Pure Functions

```bmb
pure fn square(x: i64) -> i64 = x * x;

// Compiler guarantees:
// - No side effects
// - Same input → same output
// - Safe for CSE, memoization, reordering
```

**Constraint**: Only `pure` functions allowed in contracts.

### 4.4 Closures

```bmb
let add_one = |x: i32| x + 1;

let complex = |x: i32, y: i32| {
    let sum = x + y;
    return sum * 2;
};
```

### 4.5 Closure Types

**Decision**: Simplified `fn(T) -> U` syntax for all callable types.

```bmb
// Function type annotation
fn apply(f: fn(i32) -> i32, x: i32) -> i32 = f(x);

// Works with closures, function pointers, and function items
let double = |x: i32| x * 2;
apply(double, 5);  // 10
apply(add_one, 5); // 6 (where add_one is a fn)
```

**Rationale**:
- P0-Correct: One syntax for callable types
- P1: Mirrors parameter declaration syntax `(T) -> U`
- No `Fn`/`FnMut`/`FnOnce` trait distinction at type level
- Capture semantics determined by usage (automatic `move` inference)

---

## 5. Contract System

### 5.1 Preconditions

```bmb
fn divide(a: i64, b: i64) -> i64
  pre b != 0
= a / b;

fn get(arr: &[i32], idx: usize) -> i32
  pre idx < arr.len()
= arr[idx];
```

### 5.2 Postconditions

```bmb
fn abs(x: i64) -> i64
  post ret >= 0
  post ret == x or ret == -x
= if x >= 0 { x } else { -x };
```

### 5.3 Loop Invariants

```bmb
fn sum(arr: &[i32]) -> i32 {
    let mut total = 0;
    let mut i = 0;
    while i < arr.len()
      invariant i <= arr.len()
    {
        total = total + arr[i];
        i = i + 1;
    }
    return total;
}
```

### 5.4 Quantifiers

```bmb
fn binary_search(arr: &[i32], target: i32) -> usize?
  pre forall i: 0..arr.len()-1. arr[i] <= arr[i+1]
  post ret.is_none() implies forall i: 0..arr.len(). arr[i] != target
  post ret.is_some() implies arr[ret.unwrap()] == target
{
    // ...
}
```

### 5.5 Trust Annotation

```bmb
#[trust("FFI call to verified C library")]
fn external_sqrt(x: f64) -> f64;

#[trust("performance critical, manually verified")]
fn unsafe_get(arr: &[i32], idx: usize) -> i32 = arr[idx];
```

**Design Principle**: BMB compiled code contains NO runtime contract checks. All contracts are either proven by SMT at compile-time or trusted by the programmer via `@trust`.

---

## 6. Operators Detail

### 6.1 Logical Operators

Both forms are equivalent and interchangeable:

| Symbolic | Keyword | Meaning |
|----------|---------|---------|
| `&&` | `and` | Logical AND (short-circuit) |
| `\|\|` | `or` | Logical OR (short-circuit) |
| `!` | `not` | Logical NOT |

```bmb
// Both valid and equivalent
pre b != 0 && a > 0
pre b != 0 and a > 0

// In contracts, keyword form often preferred for readability
post ret.is_none() implies forall i: 0..n. arr[i] != target
```

### 6.2 Bitwise Operators

**Keywords only** (no symbolic form):

| Operator | Meaning |
|----------|---------|
| `band` | Bitwise AND |
| `bor` | Bitwise OR |
| `bxor` | Bitwise XOR |
| `bnot` | Bitwise NOT |
| `<<` | Left shift |
| `>>` | Right shift |

```bmb
let flags = a band b;
let combined = x bor y;
let toggled = bits bxor mask;
let inverted = bnot value;
let shifted = n << 2;
```

**Rationale**: `&` and `|` are reserved for references and pattern matching.

### 6.3 Overflow Operators

| Operator | Behavior | Return Type | Use Case |
|----------|----------|-------------|----------|
| `+` `-` `*` | Requires contract | `T` | Default safe |
| `+%` `-%` `*%` | Wrapping (mod 2^n) | `T` | Hash, crypto |
| `+\|` `-\|` `*\|` | Saturating (clamp) | `T` | Graphics, audio |
| `+?` `-?` `*?` | Checked | `T?` | User input |

```bmb
// Default: requires contract or trust
fn add(a: u8, b: u8) -> u8
  pre (a as u16) + (b as u16) <= 255
= a + b;

// Explicit wrapping
let hash = a +% b;

// Explicit saturating
let pixel = r +| g;

// Explicit checked
let result: u8? = a +? b;
```

---

## 7. Control Flow

### 7.1 Conditionals

```bmb
// Statement form
if condition {
    // ...
} else if other {
    // ...
} else {
    // ...
}

// Expression form
let x = if a > b { a } else { b };
```

### 7.2 Pattern Matching

```bmb
match value {
    Pattern1 => expr1,
    Pattern2 => expr2,
    _ => default,
}

// With guards
match x {
    n if n < 0 => "negative",
    0 => "zero",
    _ => "positive",
}
```

### 7.3 Loops

```bmb
while condition {
    // ...
}

for item in iterator {
    // ...
}

for i in 0..10 {
    // ...
}

loop {
    if done { break; }
}

// With invariants
while lo < hi
  invariant lo <= hi
  invariant hi <= arr.len()
{
    // ...
}
```

---

## 8. Structures and Enums

### 8.1 Struct

```bmb
struct Point {
    x: f64,
    y: f64,
}

struct Pair<T>(T, T);  // Tuple struct

struct Marker;  // Unit struct
```

### 8.2 Enum

```bmb
enum Color {
    Red,
    Green,
    Blue,
    Rgb(u8, u8, u8),
}
```

### 8.3 Impl Blocks

```bmb
impl Point {
    fn new(x: f64, y: f64) -> Point {
        return Point { x: x, y: y };
    }

    fn distance(&self, other: &Point) -> f64 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        return (dx * dx + dy * dy).sqrt();
    }
}
```

---

## 9. Correctness Features

### 9.1 No Deref Coercion

```bmb
fn take(s: &str) {}
let string = String::new();
take(string.as_str());  // Explicit conversion required
```

### 9.2 No Auto-ref

```bmb
x.method()        // self: Self
(&x).method()     // self: &Self
(&mut x).method() // self: &mut Self
```

### 9.3 No `?` Operator for Error Propagation

```bmb
// `?` is reserved for type suffix only
let x: i32? = Some(42);

// Error propagation: explicit match
let value = match foo() {
    Some(v) => v,
    None => return None,
};
```

### 9.4 Exhaustive Pattern Matching

All match arms must be covered or use `_` wildcard.

### 9.5 No `ref` Pattern

**Decision**: Remove `ref` and `ref mut` patterns from match arms.

```bmb
// ✗ Rejected: ref pattern
match value {
    ref x => /* ... */,      // Error: ref pattern not supported
    ref mut y => /* ... */,  // Error: ref mut pattern not supported
}

// ✓ Correct: explicit reference
match &value {
    x => /* x is &T */,
}

match &mut value {
    x => /* x is &mut T */,
}
```

**Rationale**:
- P0-Correct: Binding mode should match the matched expression type
- Explicit `&`/`&mut` in match target, not implicit in pattern
- Reduces pattern complexity without losing expressiveness

### 9.6 No Struct Update Syntax

**Decision**: Remove `..expr` struct update syntax.

```bmb
// ✗ Rejected: struct update syntax
let p2 = Point { x: 10, ..p1 };  // Error: struct update not supported

// ✓ Correct: explicit field initialization
let p2 = Point { x: 10, y: p1.y };
```

**Rationale**:
- P0-Correct: All fields explicitly visible at initialization site
- No hidden field copying that could obscure large data movement
- Contract verification easier with explicit field assignment

---

## 10. Modules

```bmb
mod math {
    pub fn add(a: i32, b: i32) -> i32 = a + b;
    fn internal() {}  // private
}

use math::add;
use std::collections::HashMap;
```

---

## 11. Attributes

```bmb
#[inline]
fn small() -> i32 = 42;

#[trust("reason")]
fn unverified() {}

#[test]
fn test_add() {
    assert(add(1, 2) == 3);
}

#[cfg(target_os = "linux")]
fn linux_only() {}
```

---

## 12. Complete Example

```bmb
pure fn is_sorted(arr: &[i32]) -> bool {
    let mut i = 1;
    while i < arr.len()
      invariant i <= arr.len()
    {
        if arr[i - 1] > arr[i] {
            return false;
        }
        i = i + 1;
    }
    return true;
}

fn binary_search(arr: &[i32], target: i32) -> usize?
  pre is_sorted(arr)
  post ret.is_none() implies forall i: 0..arr.len(). arr[i] != target
  post ret.is_some() implies arr[ret.unwrap()] == target
{
    let mut lo: usize = 0;
    let mut hi: usize = arr.len();

    while lo < hi
      invariant lo <= hi and hi <= arr.len()
      invariant forall i: 0..lo. arr[i] < target
      invariant forall i: hi..arr.len(). arr[i] > target
    {
        let mid = lo + (hi - lo) / 2;

        if arr[mid] == target {
            return Some(mid);
        } else if arr[mid] < target {
            lo = mid + 1;
        } else {
            hi = mid;
        }
    }

    return None;
}

#[test]
fn test_binary_search() {
    let arr = [1, 3, 5, 7, 9];
    assert(binary_search(&arr, 5) == Some(2));
    assert(binary_search(&arr, 4).is_none());
}
```

---

## 13. Grammar Summary

### 13.1 Differences from Rust

| Item | Rust | BMB | Rationale |
|------|------|-----|-----------|
| Return in blocks | Implicit | `return` required | P0: correctness |
| Nullable | `Option<T>` | `T?` | P1: universal |
| Bitwise ops | `& \| ^ ~` | `band bor bxor bnot` | P0: no context-dependent parsing |
| Logical ops | `&& \|\| !` | Both `&&`/`and` | P1: flexibility |
| Deref coercion | Automatic | Explicit | P0: correctness |
| Auto-ref | Automatic | Explicit | P0: correctness |
| `?` operator | Error propagation | Type suffix only | P0: correctness |
| Overflow | Debug≠Release | Explicit operators | P0: correctness |
| Contracts | None | `pre`/`post`/`invariant` | P0: both |
| Closure types | `Fn`/`FnMut`/`FnOnce` | `fn(T) -> U` | P0: simplicity |
| `ref` pattern | Supported | Removed | P0: explicit binding |
| Struct update | `..expr` | Removed | P0: explicit fields |

### 13.2 Identical to Rust

| Item | Syntax |
|------|--------|
| Generics | `<T>` |
| Arrays | `[T; N]` |
| References | `&T`, `&mut T` |
| Arithmetic | `+ - * / %` |
| Comparison | `== != < > <= >=` |
| Shift | `<< >>` |
| Control flow | `if`, `match`, `while`, `for`, `loop` |
| Functions | `fn name() {}` |
| Variables | `let`, `let mut` |
| Structs/Enums | `struct`, `enum` |
| Traits | `trait`, `impl` |
| Modules | `mod`, `use`, `pub` |
| Comments | `//`, `/* */` |
| Closures | `\|x\| expr` |

---

## Appendix: Contract Verification Status

| Feature | Status |
|---------|--------|
| pre/post | Complete |
| forall/exists | Complete |
| old(expr) | Complete |
| @trust "reason" | Complete |
| todo keyword | Complete |
| Z3 integration | Complete |
| SMT-LIB2 generation | Complete |
