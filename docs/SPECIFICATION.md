# BMB Language Specification

**Version**: v0.18.1 (2026-01-04)
**Status**: Implementation in Progress

## 1. Overview

BMB is an AI-native programming language designed for:

- Contract-based correctness guarantees
- Native maximum performance (exceeding C/Rust)
- AI code generation optimization

### 1.1 Design Philosophy

#### Non-Negotiable Priorities (타협불가)

| Priority | Principle | Description |
|----------|-----------|-------------|
| **P0** | **Performance** | Syntax must enable maximum performance. No syntax limitations constraining performance. Must exceed C/Rust. |
| **P0** | **Correctness** | If compile-time checking is possible, it MUST be in the language spec. All verifiable invariants checked statically. |

#### Secondary Priorities

| Priority | Principle | Description |
|----------|-----------|-------------|
| 1 | AI Generation | Optimize for first-time correct code generation |
| 2 | Minimal Debugging | SMT counterexamples enable immediate fixes |
| 3 | Token Efficiency | Reasonable minimization |
| 4 | Code Reuse | Contract synthesis, duplication elimination |

### 1.2 Design Principles

| Principle | Description |
|-----------|-------------|
| Minimal Rules | Fewer rules to learn |
| Zero Exceptions | No exceptions to rules |
| Context Independence | Same syntax = same meaning |
| Composability | Small components compose into large ones |
| Semantic Clarity | Names convey meaning |

### 1.3 Performance Principles

| Principle | Description |
|-----------|-------------|
| Contract = Optimization Fuel | Proven conditions enable aggressive optimization |
| Exceed C/Rust Limits | Bounds check, aliasing, purity proofs |
| Safety + Speed | No compromise on either |

## 2. Lexical Structure

### 2.1 Keywords

| Category | Keywords |
|----------|----------|
| Definitions | fn, type, enum, struct, mod, impl |
| Contracts | pre, post, where, invariant, decreases, modifies |
| References | ret, self, old |
| Control | if, then, else, match, for, in |
| Bindings | let, var, mut, rec |
| Quantifiers | forall, exists |
| Memory | own, ref, drop, move, copy |
| Verification | pure, trust, contract, satisfies |
| Development | todo |

### 2.2 Operators

| Category | Operators |
|----------|-----------|
| Arithmetic | + - * / % |
| Comparison | == != < > <= >= |
| Logical | and or not |
| Bitwise | band bor bxor bnot shl shr |

## 3. Memory Model

### 3.1 Ownership Types

| Type | Meaning |
|------|---------|
| T | Owned value |
| own T | Explicit ownership (heap) |
| \&T | Immutable reference |
| \&mut T | Mutable reference (exclusive) |

### 3.2 Borrowing Rules

1. Multiple \&T OR single \&mut T (not both)
2. References cannot outlive their source
3. Cannot create \&T while \&mut T exists

## 4. Type System

### 4.1 Primitive Types

| Type | Description |
|------|-------------|
| i8..i128 | Signed integers |
| u8..u128 | Unsigned integers |
| f32, f64 | Floating point |
| bool | Boolean |
| () | Unit |

### 4.2 Composite Types

| Type | Description |
|------|-------------|
| [T] | Slice |
| [T; N] | Fixed array |
| ?T | Option |
| T ! E | Result |
| (T, U) | Tuple |

### 4.3 Refinement Types

Type definitions with constraints verified statically.

### 4.4 Contract Types

Types with quantified constraints like Sorted, NonEmpty, Unique.

### 4.5 Closure Types (v0.20.0)

Closures are anonymous functions that can capture variables from their enclosing scope.

**Syntax:**
```bmb
fn || { body }                    -- no parameters
fn |x: T| { body }                -- single parameter
fn |x: T, y: U| { body }          -- multiple parameters
```

**Design Decisions:**
- `fn` prefix: Disambiguates from other uses of `|` in future grammar extensions
- Block body required: Avoids grammar conflicts with refinement types
- Move semantics: Variables are captured by value (copy for primitives, move for owned)

**Example:**
```bmb
let x = 10;
let add_x = fn |y: i64| { x + y };
let result = add_x(5);  -- 15
```

**Status:** Parser complete, type inference and codegen planned.

## 5. Contract System

### 5.1 Basic Structure

Functions have pre (preconditions) and post (postconditions).

### 5.2 Verification Modes

| Annotation | Behavior |
|------------|----------|
| (none) | Full SMT verification required |
| @trust "reason" | Skip verification (programmer guarantee with documented reason) |

**Design Principle**: BMB compiled code contains NO runtime contract checks. All contracts are either proven by SMT at compile-time or trusted by the programmer via `@trust`.

**Rationale** (RFC-0003):
- Runtime checks violate P0 Performance (zero overhead)
- Runtime checks are equivalent to dynamic type checking
- `@trust` with mandatory reason enables code review

### 5.3 Incremental Development

| Keyword | Behavior |
|---------|----------|
| todo "message" | Placeholder for unimplemented code |

```bmb
fn complex_algorithm(data: Data) -> Result
  pre valid(data)
  post correct(ret)
= todo "implementation pending"
```

**Behavior** (RFC-0004):
- Parse/Type check: Valid, contracts verified
- Compile: Generates panic stub
- Runtime: Panic with message if reached
- Build flags: `--no-todo` (error), `--list-todo` (list all)

## 6. Contract-Based Optimization

| Contract | Optimization |
|----------|--------------|
| pre i < len(arr) | Bounds check elimination |
| pre b != 0 | Division check elimination |
| pure | CSE, memoization |
| Sorted | Binary search selection |
| aligned(N) | SIMD aligned load |

**Target: BMB >= C -O3 (all cases)**

## 7. Error Handling

Result type with propagation operator (?) for recoverable errors.

## 8. Modules

Module system with use statements and pub visibility.

## 9. Standard Library

Option, Result, array operations, higher-order functions.

## 10. Grammar (EBNF)

See full grammar in source code grammar.lalrpop.

## Appendix: Contract Verification Status

| Feature | Status |
|---------|--------|
| pre/post | Complete |
| forall/exists | Complete |
| old(expr) | Complete |
| @trust "reason" | Complete |
| todo keyword | Complete (v0.31) |
| module header | Planned (v0.31) |
| Z3 integration | Complete |
| SMT-LIB2 generation | Complete |
