# BMB Language Specification

**Version**: v0.32.1
**Date**: 2026-01-17
**Status**: Final Draft

---

## 0. Core Philosophy

> **"Performance > Everything"**

### 0.1 Why BMB Exists

#### The Fundamental Question

> **"What should a language look like to achieve Runtime Overhead Zero?"**

Every programming language faces this trade-off:

```
Runtime Overhead ←―――――――――――――――――→ Developer Effort
      (decrease)                        (increase)
```

To eliminate runtime overhead, you need:
- Complete type annotations on every function
- Formal proofs for every edge case
- Explicit memory management
- Pre/post conditions on every function

**For human developers, this is unsustainable.**

#### Every Language is a Compromise

| Language | Compromise | Cost |
|----------|------------|------|
| **C** | Shifts responsibility to developer | Safety abandoned (undefined behavior) |
| **Rust** | Automates via Borrow Checker | Learning curve, compile times |
| **Go** | Sidesteps memory issues with GC | Runtime overhead accepted |
| **Java/C#** | Full embrace of GC + runtime checks | Performance compromised |
| **Python** | Everything deferred to runtime | Severe performance penalty |

No language has achieved "Runtime Overhead Zero + Human-writable" simultaneously.

This isn't a failure of language designers. It's a **physical constraint of human cognition**.

#### AI Changes the Equation

LLMs fundamentally alter this constraint:

```
Before:  Runtime Overhead ←→ Developer Effort (human limits)
After:   Runtime Overhead ←→ AI Effort (AI handles it)
```

| Painful for Humans | AI's Response |
|-------------------|---------------|
| Type annotations everywhere | Does it without complaint |
| Contracts on every function | Generates consistently |
| Mathematical proofs | High accuracy |
| Repetitive, verbose code | No fatigue |
| Exhaustive edge case enumeration | Systematic coverage |

**Developer effort is no longer the bottleneck.**

#### Why Not Generate Machine Code Directly?

If AI is so capable, why not skip programming languages entirely?

Because **LLMs are not infinite**:

| LLM Limitation | Description |
|----------------|-------------|
| **Context Window** | 128K, 1M tokens are still finite |
| **Token Efficiency** | Longer output = higher cost, slower generation |
| **Verifiability** | Generated output must be checkable |
| **Hallucination** | Error rate explodes without abstraction |
| **Modifiability** | Bug fixes need clear intervention points |

Direct machine code generation fails:
- 1,000 lines of high-level code = tens of thousands of assembly lines (context explosion)
- No way to verify correctness of generated machine code
- Hallucination rate skyrockets without abstraction
- Debugging becomes impossible

**Conclusion: AI also needs appropriate abstraction.**

#### BMB's Position: The Optimal Abstraction Level

```
High-level (Python, JavaScript)
    ↑
    │  ✓ Token efficient
    │  ✓ Easy to verify
    │  ✗ Runtime overhead unavoidable
    │
BMB ◀── Optimal point for AI
    │
    │  ✓ Runtime Overhead Zero
    │  ✓ Verifiable (Contracts)
    │  ✗ Verbose for humans
    │
Low-level (Assembly)
    │
    │  ✗ Context explosion
    │  ✗ Unverifiable
    │  ✗ Hallucination surge
    ↓
Machine Code
```

> **BMB is the lowest abstraction level that AI can efficiently produce.**

- **Lower than BMB**: Context explosion, verification impossible, hallucination
- **Higher than BMB**: Runtime overhead unavoidable

BMB exists at the intersection of two constraints:

```
Constraint 1: Runtime Overhead Zero → Must lower abstraction
Constraint 2: AI limitations → Cannot go too low

Solution: The intersection satisfying both = BMB
```

### 0.2 The Single Goal

```
Performance > Everything
```

**Safety is not a goal—it's a consequence of pursuing maximum performance.**

When you eliminate runtime checks through compile-time proofs, you get both:
- **Maximum performance**: No runtime overhead
- **Maximum safety**: Errors caught at compile time

These aren't trade-offs. They're the same thing.

### 0.3 Design Principles

| Principle | Description | Why AI-Friendly |
|-----------|-------------|-----------------|
| **Performance First** | Performance is #1 priority in all design decisions | AI doesn't need syntactic sugar |
| **Zero-Overhead Safety** | Safety verification at compile-time only | AI can provide complete proofs |
| **Explicit Everything** | No implicit conversions, no hidden control flow | AI handles verbosity easily |
| **Contract-Driven** | Formal specifications required for optimization | AI excels at formal specification |

### 0.4 What BMB Eliminates

Runtime checks are replaced with **compile-time proofs**:

| Runtime Check (Other Languages) | BMB Approach | Overhead |
|---------------------------------|--------------|----------|
| Bounds checking | `pre idx < arr.len()` proof | **0%** |
| Null checking | `T?` type + contract proof | **0%** |
| Overflow checking | Contract or explicit operators | **0%** |
| Type casting | Static types + refinement types | **0%** |
| Division by zero | `pre divisor != 0` proof | **0%** |

### 0.5 The Trade-off

| You Give Up | You Get |
|-------------|---------|
| More type annotations | More aggressive optimization |
| Contracts required | Runtime checks eliminated |
| Explicit conversions | Predictable performance |
| More compile errors | Fewer runtime errors |

**Hard to write. Hard to get wrong. And that's what AI prefers.**

### 0.6 Why This Works for AI

Traditional languages optimize for human readability and convenience. BMB inverts this:

- **Humans** struggle with: verbose types, explicit contracts, no shortcuts
- **AI** excels at: formal specification, complete type annotations, proof generation

The result: A language where AI-generated code is both **correct** and **maximally optimized**.

### 0.7 Value Verification

BMB's design choices must be **verified through benchmarks**.

| Verification Item | Criteria | Status |
|-------------------|----------|--------|
| **Zero-Overhead Proof** | BMB safe ≡ C unsafe (identical assembly) | 🔄 In Progress |
| **Contract Optimization** | Contracts enable real optimization | 🔄 In Progress |

```bash
# Value verification commands
bmb verify --zero-overhead bench.bmb   # Assembly comparison
benchmark-bmb gate 3.1 3.2 3.3         # Performance gate verification
```

### 0.8 BMB vs Other Languages

#### Why Not C?

| Aspect | C | BMB |
|--------|---|-----|
| **Philosophy** | Trust the programmer | Prove the program |
| **Safety** | Undefined behavior | Compile-time proofs |
| **Performance** | Manual optimization | Compiler optimization via contracts |
| **AI fit** | Dangerous (UB is hard to avoid) | Natural (explicit proofs) |

C achieves zero overhead but sacrifices safety. BMB achieves zero overhead **through** safety proofs.

#### Why Not Rust?

| Aspect | Rust | BMB |
|--------|------|-----|
| **Primary goal** | Memory safety | Performance |
| **Safety mechanism** | Borrow checker | Compile-time contracts |
| **Developer focus** | Fight the compiler | Specify the invariants |
| **AI fit** | Complex lifetime annotations | Straightforward contracts |

Rust optimizes for safety and achieves good performance. BMB optimizes for performance and achieves safety as a consequence.

#### Why Not Use Existing Language + AI?

> "Just have AI write careful C or verbose Rust"

| Problem | Description |
|---------|-------------|
| **No verification** | AI-generated C has no proof of correctness |
| **Implicit overhead** | High-level Rust patterns can hide runtime costs |
| **Hallucination risk** | No language-level contract to catch AI errors |
| **Optimization ceiling** | Existing compilers don't have contract information |

BMB provides:
- **Contracts as verification**: AI must provide proofs the compiler checks
- **Explicit everything**: No hidden costs
- **Contract-driven optimization**: Compiler uses proofs to eliminate checks

#### BMB's Unique Position

```
"A language that is difficult for humans to write,
 possible for AI to write,
 and achieves Runtime Overhead Zero as a result."
```

This position **could not exist before AI**.

---

## 1. Design Principles

### 1.1 Priority

| Priority | Principle | Description |
|----------|-----------|-------------|
| **P0** | **Performance** | The single goal. No syntax that prevents optimization. Contracts enable check elimination. |
| **—** | **Zero-Overhead** | Consequence of P0. Safety = compile-time. Runtime cost = 0. |
| **—** | **Correctness** | Consequence of P0. No implicit/ambiguous behavior. Same syntax = same meaning. |
| **P1** | **LLM Efficiency** | Maximize code generation accuracy through universal conventions. |

> Zero-overhead and correctness are not separate goals—they are automatic consequences of pursuing maximum performance through compile-time verification.

### 1.2 P0 Rules (Non-negotiable)

| Rule | Description | Violation Example |
|------|-------------|-------------------|
| Zero runtime cost | Safety checks at compile-time only | Runtime bounds check |
| Compile-time verification | Provable → must be enforced | Array access without contract |
| Explicit behavior | No hidden conversions or control flow | Deref coercion, `?` operator |
| Unambiguous parsing | Same token = same meaning | Context-dependent parsing |
| Single representation | One concept = one syntax | Mixing `T?` and `Option<T>` |

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

## Appendix A: Contract Verification Status

| Feature | Status |
|---------|--------|
| pre/post | Complete |
| forall/exists | Complete |
| old(expr) | Complete |
| @trust "reason" | Complete |
| todo keyword | Complete |
| Z3 integration | Complete |
| SMT-LIB2 generation | Complete |

---

## Appendix B: Value Verification

BMB's core philosophy must be verified through **quantitative evidence**.

### B.1 Zero-Overhead Safety Proof

**Goal**: Prove that BMB's safe code generates **identical assembly** to C's unsafe code.

```bmb
// BMB: Safe array access
fn get_safe(arr: &[i32], idx: usize) -> i32
  pre idx < arr.len()
= arr[idx];
```

```c
// C: Unsafe array access (no bounds check)
int get_unsafe(int* arr, size_t idx) {
    return arr[idx];
}
```

**Verification Method**:
```bash
bmb build safe.bmb --emit-asm -o bmb.s
clang -O3 unsafe.c -S -o c.s
diff bmb.s c.s  # Must be identical
```

**Verification Targets**:

| Check Type | BMB Code | C Code | Assembly |
|------------|----------|--------|----------|
| Bounds check | `pre idx < len` | (none) | Identical |
| Null check | `T?` + `pre x.is_some()` | raw pointer | Identical |
| Overflow | `pre a + b <= MAX` | (none) | Identical |
| Division | `pre b != 0` | (none) | Identical |

### B.2 Performance Gates

| Gate | Criteria | Measurement | Status |
|------|----------|-------------|--------|
| **#3.1** | Compute ≤1.10x vs Clang | fibonacci, mandelbrot | ✅ Achieved |
| **#3.2** | All ≤1.05x vs C | 26 benchmarks | 🔄 In Progress |
| **#3.3** | 3 faster than C | Contract optimization cases | 📋 Planned |

```bash
# Performance gate verification
benchmark-bmb gate 3.1 --verbose
benchmark-bmb gate 3.2 --verbose
benchmark-bmb gate 3.3 --verbose
```

### B.3 Contract Optimization Proof

**Goal**: Prove that contracts actually trigger compiler optimizations.

| Optimization | Contract | Expected Effect | Status |
|--------------|----------|-----------------|--------|
| Bounds elim | `pre idx < len` | Remove array bounds check | 🔄 |
| Branch elim | `pre x > 0` | Remove dead branch | 🔄 |
| SIMD vectorize | `pure fn` + no aliasing | Enable auto-vectorization | 🔄 |
| Loop hoist | `invariant` | Hoist invariant outside loop | 🔄 |
| CSE | `pure fn` | Eliminate redundant calls | 🔄 |

**Verification Method**:
```bash
# LLVM IR comparison
bmb build with_contract.bmb --emit-llvm -o with.ll
bmb build without_contract.bmb --emit-llvm -o without.ll
diff with.ll without.ll  # Check optimization difference
```

### B.4 Benchmark Categories

| Category | Purpose | Benchmarks |
|----------|---------|------------|
| **Zero-Overhead** | Prove safety = zero cost | bounds, null, overflow |
| **Compute** | CPU performance | fibonacci, mandelbrot, spectral_norm |
| **Memory** | Memory efficiency | cache_stride, allocation |
| **Contract** | Contract optimization effect | purity_opt, aliasing |
| **Real-World** | Real workloads | json_parse, lexer |

### B.5 Verification Workflow

```
[Write Code] → [Add Contracts] → [SMT Verify] → [Compile] → [Benchmark]
                                      ↓
                              [Proof Failed] → [Fix Code]
                                      ↓
                              [Proof Success] → [Runtime Check = 0]
```

### B.6 Success Criteria for v1.0

| Item | Criteria | Required |
|------|----------|----------|
| Zero-Overhead Proof | 5 check types with identical assembly | ✅ |
| Gate #3.1 | ≤1.10x vs Clang | ✅ |
| Gate #3.2 | All benchmarks ≤1.05x | ✅ |
| Gate #3.3 | 3+ cases faster than C | ✅ |
| Contract Optimization | 3+ cases with >10% improvement | ✅ |

---

## Appendix C: Contract-Driven Optimization (CDO)

> **RFC**: [RFC-0008-contract-driven-optimization](rfcs/RFC-0008-contract-driven-optimization.md)

### C.1 Philosophy

Contracts are not just safety guards—they are **optimization fuel**.

```
OLD: Contracts prove code is safe
NEW: Contracts describe what code MEANS, enabling everything that follows
```

### C.2 CDO Capabilities

| Capability | Contract Example | Optimization |
|------------|------------------|--------------|
| **Semantic DCE** | `pre x > 0` | Eliminate `if x <= 0` branch |
| **Minimal Extraction** | `pre s.len() < 1000` | Skip large-string handling paths |
| **Pure Precomputation** | `pure fn` + `pre n <= 50` | Generate lookup table |
| **Semantic Deduplication** | Equivalent `post` contracts | Merge implementations |
| **Contract Specialization** | `pre is_ascii(s)` | Skip unicode normalization |

### C.3 CDO Example

```bmb
// Library provides generic parser
fn parse(s: &str) -> Result<Value, Error>
  // Handles: large files, streaming, unicode, errors

// Application uses with constraints
fn my_parse(s: &str) -> Value
  pre s.len() < 1000
  pre s.is_ascii()
= parse(s).unwrap();

// CDO extracts only:
// - Small-string paths
// - ASCII-only paths
// - No error handling (unwrap)
// Result: 80%+ code reduction
```

### C.4 CDO Phases

| Phase | Version | Description |
|-------|---------|-------------|
| Foundation | v0.55-56 | Contract IR, Semantic DCE |
| Intra-Module | v0.57-58 | Specialization, Pure optimization |
| Cross-Module | v0.60 | Link-time CDO |
| Ecosystem | v0.65 | gotgan integration |

### C.5 AI Synergy

BMB is AI-first. AI writes verbose contracts without complaint.

**More contracts = More semantic information = More optimization opportunities**

This creates a virtuous cycle where AI-generated code with rich contracts outperforms hand-written code.

---

*Last updated: 2026-01-24*
*Specification version: v0.32.1*
