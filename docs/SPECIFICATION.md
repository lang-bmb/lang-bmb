# BMB Language Specification

**Version**: v0.32.1
**Date**: 2026-01-17
**Status**: Final Draft

---

## 0. Core Philosophy

> **"인간 편의를 희생하고, 최고 성능과 안정성을 모두 확보한다"**

BMB는 C/Rust가 포기한 마지막 1~20%의 성능을 최적화하여 **이론상 C/Rust를 추월**하는 것을 목표로 한다.

### 0.1 The BMB Principle

```
성능 최우선 + 안정성 = 언어 복잡도로 해결
```

| 원칙 | 설명 | 결과 |
|------|------|------|
| **성능 최우선** | 모든 설계 결정에서 성능이 1순위 | 기계어 수준 최적화 |
| **Zero-Overhead Safety** | 안전성 검증은 컴파일 타임에만 | 런타임 비용 = 0 |
| **No Defense Code** | 방어 코드는 결국 성능 저하 | 증명으로 대체 |
| **Complexity for Humans** | 성능/안정성 충돌 시 → 언어가 복잡해짐 | 개발자가 더 많은 정보 제공 |

### 0.2 What BMB Eliminates

기존 언어들이 런타임에 수행하는 검사를 **컴파일 타임 증명**으로 대체:

| 런타임 검사 (다른 언어) | BMB 방식 | 오버헤드 |
|-------------------------|----------|----------|
| Bounds checking | `pre idx < arr.len()` 증명 | **0%** |
| Null checking | `T?` 타입 + 계약 증명 | **0%** |
| Overflow checking | 계약 또는 명시적 연산자 | **0%** |
| Type casting | 정적 타입 + 정제 타입 | **0%** |
| Division by zero | `pre divisor != 0` 증명 | **0%** |

### 0.3 The Trade-off

| 희생 (Human Convenience) | 획득 (Machine Efficiency) |
|--------------------------|---------------------------|
| 더 많은 타입 명시 | 더 공격적인 최적화 |
| 계약 작성 필수 | 런타임 체크 완전 제거 |
| 명시적 변환 필요 | 예측 가능한 성능 |
| 컴파일 에러 증가 | 런타임 에러 감소 |

### 0.4 Value Verification (가치 검증)

BMB의 철학은 **벤치마크로 검증**되어야 한다. 주장만으로는 불충분.

| 검증 항목 | 기준 | 상태 |
|-----------|------|------|
| **Zero-Overhead Proof** | BMB safe ≡ C unsafe (어셈블리 동일) | 🔄 검증중 |
| **Performance Parity** | 전체 벤치마크 ≤1.05x vs C | 🔄 일부 달성 |
| **Performance Win** | 3개 이상 벤치마크에서 C 추월 | 📋 계획 |
| **Contract Optimization** | 계약이 실제 최적화 유발 | 🔄 검증중 |

```bash
# 가치 검증 명령어
bmb verify --zero-overhead bench.bmb   # 어셈블리 비교
benchmark-bmb gate 3.1 3.2 3.3         # 성능 게이트 검증
```

---

## 1. Design Principles

### 1.1 Priority

| Priority | Principle | Description |
|----------|-----------|-------------|
| **P0** | **Performance** | 최적화를 방해하는 문법 없음. 계약으로 체크 제거. |
| **P0** | **Zero-Overhead** | 안전성 = 컴파일 타임. 런타임 비용 = 0. |
| **P0** | **Correctness** | 암시적/모호한 동작 없음. 동일 문법 = 동일 의미. |
| **P1** | **LLM Efficiency** | 범용 관례로 코드 생성 정확도 극대화. |

### 1.2 P0 Rules (Non-negotiable)

| Rule | Description | Violation Example |
|------|-------------|-------------------|
| Zero runtime cost | 안전성 검사는 컴파일 타임에만 | 런타임 bounds check |
| Compile-time verification | 증명 가능 → 반드시 적용 | 계약 없는 배열 접근 |
| Explicit behavior | 숨겨진 변환/제어흐름 없음 | Deref coercion, `?` operator |
| Unambiguous parsing | 동일 토큰 = 동일 의미 | Context-dependent parsing |
| Single representation | 하나의 개념 = 하나의 문법 | `T?`와 `Option<T>` 혼용 |

### 1.3 P1 Rules (Balanced)

| Rule | Description | Application |
|------|-------------|-------------|
| Universal over Rust-specific | 범용 관례 선호 | `T?` over `Option<T>` |
| Rust when universal | Rust가 표준일 때 Rust 문법 | `<T>`, `match`, `&&` |
| Modern over historical | 현대 표준 선호 | `T?` (2011+) over `Option<T>` (2010) |
| LLM data coverage | 학습 데이터 가용성 고려 | `Result<T,E>` (Rust 데이터 풍부) |

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

## Appendix B: Value Verification (가치 검증)

BMB의 핵심 철학은 반드시 **정량적 증거**로 검증되어야 한다.

### B.1 Zero-Overhead Safety Proof

**목표**: BMB의 안전한 코드가 C의 unsafe 코드와 **동일한 어셈블리**를 생성함을 증명

```bmb
// BMB: 안전한 배열 접근
fn get_safe(arr: &[i32], idx: usize) -> i32
  pre idx < arr.len()
= arr[idx];
```

```c
// C: unsafe 배열 접근 (bounds check 없음)
int get_unsafe(int* arr, size_t idx) {
    return arr[idx];
}
```

**검증 방법**:
```bash
bmb build safe.bmb --emit-asm -o bmb.s
clang -O3 unsafe.c -S -o c.s
diff bmb.s c.s  # 동일해야 함
```

**검증 대상**:

| 검사 유형 | BMB 코드 | C 코드 | 어셈블리 |
|-----------|----------|--------|----------|
| Bounds check | `pre idx < len` | (없음) | 동일 |
| Null check | `T?` + `pre x.is_some()` | raw pointer | 동일 |
| Overflow | `pre a + b <= MAX` | (없음) | 동일 |
| Division | `pre b != 0` | (없음) | 동일 |

### B.2 Performance Gates

| Gate | 기준 | 측정 방법 | 목표 |
|------|------|-----------|------|
| **#3.1** | Compute ≤1.10x vs Clang | fibonacci, mandelbrot | ✅ 달성 |
| **#3.2** | 전체 ≤1.05x vs C | 26개 벤치마크 | 🔄 진행중 |
| **#3.3** | 3개 C 추월 | 계약 최적화 케이스 | 📋 계획 |

```bash
# 성능 게이트 검증
benchmark-bmb gate 3.1 --verbose
benchmark-bmb gate 3.2 --verbose
benchmark-bmb gate 3.3 --verbose
```

### B.3 Contract Optimization Proof

**목표**: 계약이 실제로 컴파일러 최적화를 유발함을 증명

| 최적화 | 계약 | 기대 효과 | 검증 상태 |
|--------|------|-----------|-----------|
| Bounds elim | `pre idx < len` | 배열 접근 시 체크 제거 | 🔄 |
| Branch elim | `pre x > 0` | dead branch 제거 | 🔄 |
| SIMD vectorize | `pure fn` + no aliasing | 자동 벡터화 활성화 | 🔄 |
| Loop hoist | `invariant` | 불변량 루프 밖 이동 | 🔄 |
| CSE | `pure fn` | 중복 호출 제거 | 🔄 |

**검증 방법**:
```bash
# LLVM IR 비교
bmb build with_contract.bmb --emit-llvm -o with.ll
bmb build without_contract.bmb --emit-llvm -o without.ll
diff with.ll without.ll  # 최적화 차이 확인
```

### B.4 Benchmark Categories

| Category | 목적 | 벤치마크 |
|----------|------|----------|
| **Zero-Overhead** | 안전성 = 무비용 증명 | bounds, null, overflow |
| **Compute** | CPU 성능 | fibonacci, mandelbrot, spectral_norm |
| **Memory** | 메모리 효율성 | cache_stride, allocation |
| **Contract** | 계약 최적화 효과 | purity_opt, aliasing |
| **Real-World** | 실제 워크로드 | json_parse, lexer |

### B.5 Verification Workflow

```
[코드 작성] → [계약 추가] → [SMT 검증] → [컴파일] → [벤치마크]
                                ↓
                         [증명 실패] → [코드 수정]
                                ↓
                         [증명 성공] → [런타임 체크 = 0]
```

### B.6 Success Criteria for v1.0

| 항목 | 기준 | 필수 |
|------|------|------|
| Zero-Overhead 증명 | 5개 검사 유형 어셈블리 동일 | ✅ |
| Gate #3.1 | Clang 대비 ≤1.10x | ✅ |
| Gate #3.2 | 전체 벤치마크 ≤1.05x | ✅ |
| Gate #3.3 | 3개 이상 C 추월 | ✅ |
| Contract 최적화 | 3개 이상 케이스에서 >10% 개선 | ✅ |

---

*Last updated: 2026-01-17*
*Specification version: v0.32.1*
