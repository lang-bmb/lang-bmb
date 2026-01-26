# BMB Language Reference

**Version**: v0.51.40
**Status**: Implementation Reference

This document provides a comprehensive reference for the BMB programming language syntax and semantics.

## Table of Contents

1. [Lexical Structure](#1-lexical-structure)
2. [Types](#2-types)
3. [Expressions](#3-expressions)
4. [Functions](#4-functions)
5. [Data Types](#5-data-types)
6. [Traits and Implementations](#6-traits-and-implementations)
7. [Module System](#7-module-system)
8. [Memory Model](#8-memory-model)
9. [Control Flow](#9-control-flow)
10. [Contracts](#10-contracts)
11. [Attributes](#11-attributes)

---

## 1. Lexical Structure

### 1.1 Comments

BMB uses line comments starting with `--`:

```bmb
-- This is a comment
fn add(a: i64, b: i64) -> i64 = a + b;  -- inline comment
```

### 1.2 Keywords

| Category | Keywords |
|----------|----------|
| Definitions | `fn`, `struct`, `enum`, `trait`, `impl`, `type` |
| Contracts | `pre`, `post`, `where`, `it`, `ret` |
| Control Flow | `if`, `then`, `else`, `match`, `while`, `for`, `in`, `try` |
| Bindings | `let`, `var`, `mut`, `set` |
| Memory | `new`, `&`, `mut`, `null` |
| Type Operations | `as` |
| Visibility | `pub`, `use`, `mod` |
| External | `extern` |
| Literals | `true`, `false`, `null` |
| Module Header | `module`, `version`, `summary`, `exports`, `depends` |
| Incremental | `todo` |
| Logical | `and`, `or`, `not` |

### 1.3 Operators

#### Arithmetic Operators
| Operator | Description | Precedence |
|----------|-------------|------------|
| `+` | Addition | 6 |
| `-` | Subtraction | 6 |
| `*` | Multiplication | 7 |
| `/` | Division | 7 |
| `%` | Modulo | 7 |

#### Comparison Operators
| Operator | Description | Precedence |
|----------|-------------|------------|
| `==` | Equal | 4 |
| `!=` | Not equal | 4 |
| `<` | Less than | 4 |
| `>` | Greater than | 4 |
| `<=` | Less or equal | 4 |
| `>=` | Greater or equal | 4 |

#### Logical Operators
| Operator | Description | Precedence |
|----------|-------------|------------|
| `and` | Logical AND | 3 |
| `or` | Logical OR | 2 |
| `not` | Logical NOT | 8 (unary) |

#### Range Operators
| Operator | Description | Example |
|----------|-------------|---------|
| `..<` | Exclusive range [start, end) | `0..<10` |
| `..=` | Inclusive range [start, end] | `0..=10` |
| `..` | Legacy exclusive (deprecated) | `0..10` |

#### Other Operators
| Operator | Description |
|----------|-------------|
| `?` | Error propagation |
| `&` | Reference / immutable borrow |
| `&mut` | Mutable borrow |
| `*` | Dereference / pointer type |
| `.` | Field access / method call |
| `::` | Path separator (enum variants) |
| `->` | Return type arrow |
| `=>` | Match arm arrow |
| `as` | Type casting (v0.39) |
| `:=` | Field/index assignment (v0.51.23) |

### 1.4 Literals

#### Integer Literals
```bmb
42        -- decimal
-17       -- negative
0         -- zero
```

#### Float Literals
```bmb
3.14      -- decimal float
-0.5      -- negative float
1.0       -- with decimal point
```

#### String Literals
```bmb
"hello"           // simple string
"hello, world"    // with comma
"line 1           // literal newline (BMB does not support escape sequences)
line 2"           // multi-line strings use actual newlines
```

**Note**: BMB currently does not support escape sequences like `\n`, `\t`, or `\"` in strings. To include newlines, use actual newline characters within the string literal. Double quotes cannot be included in string literals at this time.

#### Boolean Literals
```bmb
true
false
```

### 1.5 Identifiers

Identifiers start with a letter or underscore, followed by letters, digits, or underscores:

```bmb
foo
_private
myVariable
var123
```

---

## 2. Types

### 2.1 Primitive Types

| Type | Description | Range |
|------|-------------|-------|
| `i32` | 32-bit signed integer | -2³¹ to 2³¹-1 |
| `i64` | 64-bit signed integer | -2⁶³ to 2⁶³-1 |
| `f64` | 64-bit floating point | IEEE 754 double |
| `bool` | Boolean | `true` or `false` |
| `String` | UTF-8 string | - |
| `()` | Unit type | single value `()` |

### 2.2 Reference Types

```bmb
&T        -- immutable reference to T
&mut T    -- mutable reference to T
```

### 2.3 Pointer Types (v0.51.37)

Typed pointer types for low-level memory operations:

```bmb
*T        -- typed pointer to T

-- Example: Self-referential struct
struct Node {
    value: i64,
    next: *Node      -- pointer to Node
}

-- Null pointer literal (v0.51.40)
let p: *Node = null;

-- Cast between pointer and integer
let addr: i64 = ptr as i64;
let ptr: *Node = addr as *Node;

-- Field access through pointer (auto-dereference)
let n = malloc(16) as *Node;
let v = n.value;              -- auto-derefs: (*n).value
set n.value = 42;             -- field assignment through pointer
```

**Key Features:**
- `*T` creates a typed pointer to type T
- `null` is the null pointer literal (type-inferred)
- Pointers auto-dereference for field access
- Use `as` for casting between `*T` and `i64`
- Pointer comparisons work with `==` and `!=`

### 2.4 Array Types

```bmb
[T; N]    -- fixed-size array of N elements of type T

-- Example
let arr: [i64; 5] = [1, 2, 3, 4, 5];
```

### 2.4 Generic Types

```bmb
Container<T>        -- generic type with one parameter
Result<T, E>        -- generic type with two parameters
Option<T>           -- optional value
```

### 2.5 Refinement Types

Refinement types add constraints to base types using `{constraints}` syntax:

```bmb
-- Single constraint
i64{it > 0}           -- positive integer

-- Multiple constraints (comma-separated)
i64{it >= 0, it < 100}  -- integer in range [0, 100)

-- Using in function signature
fn abs(x: i64) -> i64{it >= 0} = if x >= 0 then x else 0 - x;
```

The `it` keyword refers to the value being constrained.

### 2.6 Named Types (Type Aliases)

```bmb
type NonZero = i64 where self != 0;
type Positive = i64 where self > 0;
```

---

## 3. Expressions

### 3.1 Expression Precedence (High to Low)

| Level | Operators | Associativity |
|-------|-----------|---------------|
| 9 | Primary (literals, variables, parentheses) | - |
| 8 | Unary (`-`, `not`, `&`, `&mut`, `*`) | Right |
| 7 | Multiplicative (`*`, `/`, `%`) | Left |
| 6 | Additive (`+`, `-`) | Left |
| 5 | Range (`..<`, `..=`, `..`) | - |
| 4 | Comparison (`==`, `!=`, `<`, `>`, `<=`, `>=`) | - |
| 3 | Logical AND (`and`) | Left |
| 2 | Logical OR (`or`) | Left |
| 1 | Conditional (`if-then-else`), `let`, blocks | - |

### 3.2 Primary Expressions

```bmb
-- Literals
42                    -- integer
3.14                  -- float
"hello"               -- string
true                  -- boolean
()                    -- unit

-- Variables
x                     -- variable reference
ret                   -- return value (in contracts)
it                    -- refinement self-reference

-- Parenthesized
(a + b) * c

-- Todo placeholder (v0.31)
todo                  -- panics at runtime
todo "not implemented"  -- panics with message
```

### 3.3 Arithmetic Expressions

```bmb
a + b       -- addition
a - b       -- subtraction
a * b       -- multiplication
a / b       -- division
a % b       -- modulo
-a          -- negation
```

### 3.4 Comparison Expressions

```bmb
a == b      -- equality
a != b      -- inequality
a < b       -- less than
a > b       -- greater than
a <= b      -- less or equal
a >= b      -- greater or equal
```

### 3.5 Logical Expressions

```bmb
a and b     -- logical AND
a or b      -- logical OR
not a       -- logical NOT
```

### 3.6 Conditional Expressions

```bmb
if condition then expr1 else expr2

-- Example
let max = if a > b then a else b;
```

### 3.7 Block Expressions

```bmb
{
    let x = 1;
    let y = 2;
    x + y       -- last expression is the block's value
}
```

### 3.8 Let Expressions

```bmb
let name = value; body

-- With type annotation
let x: i64 = 42; x + 1

-- Mutable binding
let mut counter = 0;
counter = counter + 1;
counter
```

### 3.9 Function Calls

```bmb
func(arg1, arg2, arg3)

-- Examples
add(1, 2)
len(array)
max(a, b)
```

### 3.10 Method Calls

```bmb
receiver.method(args)

-- Examples
array.len()
option.unwrap()
list.push(item)
```

### 3.11 Field Access

```bmb
struct_expr.field_name

-- Example
point.x
person.name
```

### 3.12 Index Access

```bmb
array[index]

-- Example
arr[0]
matrix[i][j]
```

### 3.13 Struct Initialization

```bmb
new StructName { field1: value1, field2: value2 }

-- Example
new Point { x: 10, y: 20 }
```

### 3.14 Enum Variant Construction

```bmb
-- Unit variant
EnumName::Variant

-- Tuple variant
EnumName::Variant(value1, value2)

-- Examples
Option::None
Option::Some(42)
Result::Ok(value)
Result::Err("error message")
```

### 3.15 Array Literals

```bmb
[elem1, elem2, elem3]

-- Examples
[1, 2, 3, 4, 5]
["a", "b", "c"]
[]                    -- empty array
```

### 3.16 Reference Expressions

```bmb
&expr         -- immutable reference
&mut expr     -- mutable reference
*expr         -- dereference
```

### 3.17 Range Expressions

```bmb
start..<end    -- exclusive range [start, end)
start..=end    -- inclusive range [start, end]

-- Examples (used in for loops)
for i in 0..<10 { ... }
for i in 1..=100 { ... }
```

### 3.18 Closure Expressions

Closures are anonymous functions that capture variables from their environment:

```bmb
-- No parameters
fn || { 42 }

-- Single parameter (with or without type)
fn |x| { x + 1 }
fn |x: i64| { x + 1 }

-- Multiple parameters
fn |a, b| { a + b }
fn |a: i64, b: i64| { a + b }

-- Example usage
let add_ten = fn |x: i64| { x + 10 };
let result = add_ten(5);  -- result = 15
```

### 3.19 Error Propagation

```bmb
expr?         -- propagate error if Result::Err, unwrap if Ok

-- Example
fn read_file(path: String) -> String ! IoError = {
    let handle = open(path)?;
    read_all(handle)?
};
```

### 3.20 Try Blocks

```bmb
try {
    body_that_may_fail
}
```

### 3.21 State References (Contracts)

```bmb
expr.pre      -- value before function execution
expr.post     -- value after function execution

-- Used in contracts
fn increment(x: &mut i64) -> ()
  where { increased: x.post == x.pre + 1 }
= { *x = *x + 1 };
```

---

## 4. Functions

### 4.1 Basic Function Definition

```bmb
fn function_name(param1: Type1, param2: Type2) -> ReturnType = body;

-- Example
fn add(a: i64, b: i64) -> i64 = a + b;
```

### 4.2 Named Return Value

```bmb
fn function_name(params) -> result_name: ReturnType = body;

-- Example (used in contracts)
fn abs(x: i64) -> r: i64
  where { non_negative: r >= 0 }
= if x >= 0 then x else 0 - x;
```

### 4.3 Generic Functions

```bmb
fn function_name<T>(param: T) -> T = body;
fn function_name<T, U>(a: T, b: U) -> T = body;

-- With bounds
fn function_name<T: Ord>(a: T, b: T) -> T = body;
fn function_name<T: Ord + Clone>(param: T) -> T = body;
```

### 4.4 Function with Contracts

```bmb
-- v0.2 style with where block (preferred)
fn divide(a: i64, b: i64{it != 0}) -> r: i64
  where {
    correct: r * b == a
  }
= a / b;

-- Legacy style with pre/post (deprecated)
fn divide(a: i64, b: i64) -> i64
  pre b != 0
  post ret * b == a
= a / b;
```

### 4.5 External Functions

```bmb
-- Default ABI
extern fn external_function(params) -> ReturnType;

-- C ABI
extern "C" fn c_function(params) -> ReturnType;

-- System ABI
extern "system" fn system_function(params) -> ReturnType;

-- With link attribute
@link("library_name")
extern "C" fn linked_function(params) -> ReturnType;
```

### 4.6 Visibility

```bmb
-- Private (default)
fn private_function() -> i64 = 42;

-- Public
pub fn public_function() -> i64 = 42;
```

---

## 5. Data Types

### 5.1 Struct Definition

```bmb
struct StructName {
    field1: Type1,
    field2: Type2,
}

-- Example
struct Point {
    x: i64,
    y: i64,
}

-- With visibility
pub struct PublicPoint {
    x: i64,
    y: i64,
}
```

### 5.2 Generic Struct

```bmb
struct Container<T> {
    value: T,
    count: i64,
}

struct Pair<T, U> {
    first: T,
    second: U,
}
```

### 5.3 Enum Definition

```bmb
enum EnumName {
    Variant1,
    Variant2(Type1),
    Variant3(Type1, Type2),
}

-- Example
enum Option<T> {
    None,
    Some(T),
}

enum Result<T, E> {
    Ok(T),
    Err(E),
}
```

### 5.4 Pattern Matching

```bmb
match expression {
    Pattern1 => result1,
    Pattern2 => result2,
    _ => default_result,
}

-- Patterns
_                       -- wildcard
42                      -- literal
"hello"                 -- string literal
true                    -- boolean
variable                -- binding
EnumName::Variant       -- unit variant
EnumName::Variant(x, y) -- tuple variant with bindings

-- Example
match option {
    Option::Some(x) => x,
    Option::None => 0,
}
```

---

## 6. Traits and Implementations

### 6.1 Trait Definition

```bmb
trait TraitName {
    fn method(self) -> ReturnType;
    fn method_with_params(self, param: Type) -> ReturnType;
}

-- Example
trait Eq {
    fn eq(self, other: &Self) -> bool;
}

trait Ord {
    fn cmp(self, other: &Self) -> Ordering;
}
```

### 6.2 Generic Trait

```bmb
trait Container<T> {
    fn get(self, index: i64) -> T;
    fn len(self) -> i64;
}
```

### 6.3 Trait Implementation

```bmb
impl TraitName for TypeName {
    fn method(self) -> ReturnType = body;
}

-- Example
impl Eq for Point {
    fn eq(self, other: &Point) -> bool =
        self.x == other.x and self.y == other.y;
}

-- Generic implementation
impl<T> Container<T> for List<T> {
    fn get(self, index: i64) -> T = self.data[index];
    fn len(self) -> i64 = self.size;
}
```

---

## 7. Module System

### 7.1 Module Header (v0.31)

```bmb
module module.path.name
  version "1.0.0"
  summary "Module description"
  exports symbol1, symbol2
  depends
    dependency.path (imported_symbol1, imported_symbol2)
===

-- Module body follows
fn symbol1() -> i64 = 42;
fn symbol2() -> i64 = 100;
```

#### Header Components

| Component | Required | Description |
|-----------|----------|-------------|
| `module` | Yes | Fully qualified module path (dot-separated) |
| `version` | No | SemVer version string |
| `summary` | No | One-line module description |
| `exports` | No | Comma-separated list of exported symbols |
| `depends` | No | Module dependencies with optional imports |
| `===` | Yes* | Header-body separator (*only when header present) |

### 7.2 Use Statements

```bmb
use module::path::symbol;
use module::path::*;           -- import all (if supported)

-- Example
use std::option::Option;
use std::result::Result;
```

### 7.3 Visibility Modifiers

```bmb
pub fn public_function() -> i64 = 42;
pub struct PublicStruct { ... }
pub enum PublicEnum { ... }
pub trait PublicTrait { ... }
```

---

## 8. Memory Model

### 8.1 Ownership

Every value in BMB has a single owner. When the owner goes out of scope, the value is dropped.

```bmb
{
    let x = create_resource();  -- x owns the resource
    use(x);
}  -- x goes out of scope, resource is dropped
```

### 8.2 Borrowing

#### Immutable Borrow

```bmb
let x: i64 = 42;
let r: &i64 = &x;     -- immutable borrow
use(*r);              -- dereference to use value
```

Multiple immutable borrows are allowed:

```bmb
let x: i64 = 42;
let r1: &i64 = &x;
let r2: &i64 = &x;    -- both r1 and r2 can exist
```

#### Mutable Borrow

```bmb
let mut x: i64 = 42;
let r: &mut i64 = &mut x;  -- mutable borrow
*r = 100;                   -- modify through reference
```

Only one mutable borrow at a time:

```bmb
let mut x: i64 = 42;
let r: &mut i64 = &mut x;
-- Cannot create another borrow while r exists
```

### 8.3 Borrowing Rules

1. **Either** multiple immutable borrows (`&T`) **OR** a single mutable borrow (`&mut T`)
2. References cannot outlive their source
3. Cannot create `&T` while `&mut T` exists

---

## 9. Control Flow

### 9.1 Conditional Expression

```bmb
if condition then expr1 else expr2

-- Example
let abs_x = if x >= 0 then x else 0 - x;
```

### 9.2 Match Expression

```bmb
match value {
    pattern1 => result1,
    pattern2 => result2,
    _ => default,
}
```

### 9.3 While Loop

```bmb
while condition {
    body
}

-- Example
let mut i = 0;
while i < 10 {
    i = i + 1
}
```

### 9.4 For Loop

```bmb
for variable in range {
    body
}

-- Examples
for i in 0..<10 {
    process(i)
}

for i in 1..=100 {
    sum = sum + i
}
```

---

## 10. Contracts

### 10.1 Contract System Overview

BMB uses compile-time contract verification via SMT solver (Z3). All contracts are verified statically - there are NO runtime contract checks in compiled code.

### 10.2 Where Block (v0.2 Recommended)

```bmb
fn function_name(params) -> result: ReturnType
  where {
    contract_name1: condition1,
    contract_name2: condition2,
  }
= body;
```

#### Named Contracts

```bmb
fn min(a: i64, b: i64) -> r: i64
  where {
    less_or_equal_a: r <= a,
    less_or_equal_b: r <= b,
    is_input: r == a or r == b,
  }
= if a <= b then a else b;
```

#### Anonymous Contracts

```bmb
fn abs(x: i64) -> r: i64
  where {
    r >= 0,                    -- anonymous constraint
    x >= 0 or r == 0 - x,      -- another anonymous constraint
  }
= if x >= 0 then x else 0 - x;
```

### 10.3 Inline Refinement Types

```bmb
-- In parameter type
fn divide(a: i64, b: i64{it != 0}) -> i64 = a / b;

-- In return type
fn abs(x: i64) -> i64{it >= 0} = if x >= 0 then x else 0 - x;

-- Multiple constraints
fn clamp(x: i64, lo: i64, hi: i64{it >= lo}) -> i64{it >= lo, it <= hi}
= if x < lo then lo else if x > hi then hi else x;
```

### 10.4 Legacy Pre/Post (Deprecated)

```bmb
fn function_name(params) -> ReturnType
  pre precondition
  post postcondition
= body;

-- Example
fn divide(a: i64, b: i64) -> i64
  pre b != 0
  post ret * b == a
= a / b;
```

### 10.5 Contract Keywords

| Keyword | Context | Description |
|---------|---------|-------------|
| `it` | Refinement type | Self-reference in constraint |
| `ret` | Post-condition | Return value reference |
| `.pre` | Contract | Pre-state value |
| `.post` | Contract | Post-state value |

### 10.6 Verification Modes

| Mode | Annotation | Behavior |
|------|------------|----------|
| Verified | (none) | Full SMT verification required |
| Trusted | `@trust "reason"` | Skip verification with documented reason |

### 10.7 Trust Attribute

```bmb
@trust "verified manually due to external library"
fn external_call(x: i64) -> i64{it >= 0} = lib_abs(x);

@trust "performance-critical, manually verified"
fn optimized_sort(arr: &mut [i64]) -> ()
  where { sorted: forall(i in 0..<len(arr)-1): arr[i] <= arr[i+1] }
= simd_sort(arr);
```

---

## 11. Attributes

### 11.1 Attribute Syntax

```bmb
@simple                     -- simple attribute
@name(arg1, arg2)           -- attribute with arguments
@trust "mandatory reason"   -- attribute with reason (v0.31)
```

### 11.2 Common Attributes

| Attribute | Description |
|-----------|-------------|
| `@pure` | Function has no side effects |
| `@inline` | Hint for inlining |
| `@trust "reason"` | Skip verification with reason |
| `@link("name")` | Link to external library |
| `@decreases(expr)` | Termination measure for recursion |
| `@invariant(expr)` | Loop/type invariant |

### 11.3 Examples

```bmb
@pure
fn square(x: i64) -> i64{it >= 0} = x * x;

@decreases(n)
fn factorial(n: i64{it >= 0}) -> i64{it >= 1}
= if n == 0 then 1 else n * factorial(n - 1);

@link("math")
extern "C" fn sin(x: f64) -> f64;
```

---

## Appendix A: Grammar Summary

### A.1 Program Structure

```ebnf
Program     ::= ModuleHeader? Item*
ModuleHeader ::= 'module' ModulePath
                 ('version' STRING)?
                 ('summary' STRING)?
                 ('exports' IdentList)?
                 ('depends' Dependency+)?
                 '==='
Item        ::= FnDef | StructDef | EnumDef | TraitDef | ImplBlock | UseStmt | ExternFn
```

### A.2 Declarations

```ebnf
FnDef       ::= Attr* Visibility 'fn' IDENT TypeParams? '(' Params ')' '->' ReturnType
                ContractClause? '=' Expr ';'
StructDef   ::= Attr* Visibility 'struct' IDENT TypeParams? '{' StructFields '}'
EnumDef     ::= Attr* Visibility 'enum' IDENT TypeParams? '{' EnumVariants '}'
TraitDef    ::= Attr* Visibility 'trait' IDENT TypeParams? '{' TraitMethods '}'
ImplBlock   ::= Attr* 'impl' TypeParams? IDENT 'for' Type '{' ImplMethods '}'
```

### A.3 Types

```ebnf
Type        ::= PrimitiveType
              | PrimitiveType '{' Constraints '}'  -- refinement
              | IDENT                               -- named type
              | IDENT '<' TypeArgs '>'              -- generic type
              | '&' Type                            -- immutable ref
              | '&' 'mut' Type                      -- mutable ref
              | '[' Type ';' INT ']'                -- array
              | '(' ')'                             -- unit
PrimitiveType ::= 'i32' | 'i64' | 'f64' | 'bool' | 'String'
```

### A.4 Expressions

```ebnf
Expr        ::= Primary
              | UnaryOp Expr
              | Expr BinOp Expr
              | 'if' Expr 'then' Expr 'else' Expr
              | 'let' 'mut'? IDENT (':' Type)? '=' Expr ';' Expr
              | '{' (Stmt ';')* Expr? '}'
              | 'match' Expr '{' MatchArm* '}'
              | 'while' Expr '{' Expr '}'
              | 'for' IDENT 'in' RangeExpr '{' Expr '}'
              | 'try' '{' Expr '}'
Primary     ::= INT | FLOAT | STRING | 'true' | 'false' | '(' ')' | IDENT
              | IDENT '(' Args ')'                  -- call
              | Expr '.' IDENT                      -- field access
              | Expr '.' IDENT '(' Args ')'         -- method call
              | Expr '[' Expr ']'                   -- index
              | 'new' IDENT '{' FieldInits '}'      -- struct init
              | IDENT '::' IDENT ('(' Args ')')?    -- enum variant
              | '[' Args ']'                        -- array literal
              | 'fn' '|' Params '|' '{' Expr '}'    -- closure
              | Expr '?'                            -- error prop
              | '(' Expr ')'
              | 'todo' STRING?
```

---

## Appendix B: Operator Precedence Table

| Precedence | Operators | Associativity | Description |
|------------|-----------|---------------|-------------|
| 1 | `if-then-else`, `let`, `match`, `while`, `for` | - | Control flow |
| 2 | `or` | Left | Logical OR |
| 3 | `and` | Left | Logical AND |
| 4 | `==`, `!=`, `<`, `>`, `<=`, `>=` | Non-assoc | Comparison |
| 5 | `..<`, `..=`, `..` | Non-assoc | Range |
| 6 | `+`, `-` | Left | Additive |
| 7 | `*`, `/`, `%` | Left | Multiplicative |
| 8 | `-` (unary), `not`, `&`, `&mut`, `*` | Right | Unary |
| 9 | `.`, `[]`, `()`, `?`, `.pre`, `.post` | Left | Postfix |

---

## Appendix C: Reserved for Future Use

The following are reserved for potential future features:

- `async`, `await` - Asynchronous programming
- `unsafe` - Unsafe code blocks
- `move`, `copy`, `drop` - Explicit memory operations
- `forall`, `exists` - Quantifiers in contracts
- `old` - Legacy pre-state reference
- `invariant`, `decreases` - Additional contract annotations
- `satisfies` - Type constraint verification
- `linear` - Linear types
- `own` - Explicit ownership
- `rec` - Recursive bindings

---

*Last updated: v0.31 (2026-01-07)*
