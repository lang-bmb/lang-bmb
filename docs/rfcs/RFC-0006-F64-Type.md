# RFC-0006: f64 Floating-Point Type Support

**Status**: Implemented
**Created**: 2026-01-08
**Implemented**: 2026-01-09
**Target Version**: v0.34
**Priority**: P0 (blocks n_body, mandelbrot_fp, spectral_norm_fp benchmarks)

## Implementation Status

| Component | Status | Notes |
|-----------|--------|-------|
| Lexer | ✅ Complete | `TyF64`, `FloatLit(f64)`, scientific notation (v0.34) |
| Parser | ✅ Complete | Full f64 support in grammar.lalrpop |
| AST | ✅ Complete | `Expr::FloatLit(f64)`, `Type::F64` |
| Type Checker | ✅ Complete | Arithmetic/comparison ops |
| MIR | ✅ Complete | `MirType::F64`, float operations |
| LLVM Codegen | ✅ Complete | `context.f64_type()`, float instructions, sqrt intrinsic |
| WASM Codegen | ✅ Complete | f64 support |
| stdlib/math | ⚠️ Partial | sqrt, i64_to_f64, f64_to_i64 intrinsics (v0.34) |
| Benchmarks | ✅ Complete | mandelbrot_fp (3989), n_body (-142736398) verified |

**Scientific Notation Support** (v0.34):
- `6.022e23`, `3.14e10` (positive exponent)
- `1e-5`, `1.5e-10` (negative exponent)
- `1E10`, `6.022E23` (uppercase E)
- `1e10` (integer mantissa with exponent)

## Summary

Add IEEE 754 double-precision floating-point type (`f64`) to BMB for scientific computing, physics simulations, and graphics applications.

## Motivation

### Benchmark Gaps

Current benchmarks blocked by missing f64:

| Benchmark | Requirement | Impact |
|-----------|-------------|--------|
| n_body | Physics simulation | Blocked |
| mandelbrot_fp | Complex number math | Blocked (using fixed-point workaround) |
| spectral_norm_fp | Matrix operations | Blocked |

### Use Cases

1. **Scientific Computing**: Physics simulations, numerical analysis
2. **Graphics**: 3D transformations, color calculations
3. **Machine Learning**: Gradient computations, loss functions
4. **Financial**: Precise monetary calculations (with appropriate rounding)

### Philosophy Alignment

Per BMB's Non-Negotiable Priorities:
- **P0 Performance**: f64 operations must match C/Rust performance
- **P0 Correctness**: Contracts must support floating-point constraints
- **P0 Self-Hosting**: Not blocking (compiler uses i64 arithmetic)

## Design

### Type Specification

```bmb
-- IEEE 754 double-precision (64-bit)
-- Range: ±1.7976931348623157E+308
-- Precision: ~15-17 significant decimal digits
type f64 = primitive;
```

### Literal Syntax

```bmb
-- Decimal notation
let pi = 3.14159265358979;
let negative = -2.5;
let zero = 0.0;

-- Scientific notation
let avogadro = 6.022e23;
let planck = 6.626e-34;

-- Hex float notation (optional, for precise bit patterns)
let half = 0x1.0p-1;  -- 0.5
```

### Arithmetic Operations

| Operation | Syntax | LLVM IR | Notes |
|-----------|--------|---------|-------|
| Addition | `a + b` | `fadd double` | |
| Subtraction | `a - b` | `fsub double` | |
| Multiplication | `a * b` | `fmul double` | |
| Division | `a / b` | `fdiv double` | |
| Remainder | `a % b` | `frem double` | |
| Negation | `-a` | `fneg double` | |

### Comparison Operations

| Operation | Syntax | LLVM IR | Predicate |
|-----------|--------|---------|-----------|
| Equal | `a == b` | `fcmp oeq` | Ordered equal |
| Not equal | `a != b` | `fcmp one` | Ordered not equal |
| Less than | `a < b` | `fcmp olt` | Ordered less than |
| Less or equal | `a <= b` | `fcmp ole` | Ordered less or equal |
| Greater than | `a > b` | `fcmp ogt` | Ordered greater than |
| Greater or equal | `a >= b` | `fcmp oge` | Ordered greater or equal |

**Note**: All comparisons use "ordered" predicates (return false if either operand is NaN).

### Standard Library Functions

```bmb
-- stdlib/math/f64.bmb

-- Basic operations
fn abs(x: f64) -> f64{it >= 0.0};
fn sqrt(x: f64{it >= 0.0}) -> f64{it >= 0.0};
fn pow(base: f64, exp: f64) -> f64;

-- Trigonometric
fn sin(x: f64) -> f64{it >= -1.0, it <= 1.0};
fn cos(x: f64) -> f64{it >= -1.0, it <= 1.0};
fn tan(x: f64) -> f64;
fn asin(x: f64{it >= -1.0, it <= 1.0}) -> f64;
fn acos(x: f64{it >= -1.0, it <= 1.0}) -> f64;
fn atan(x: f64) -> f64;
fn atan2(y: f64, x: f64) -> f64;

-- Exponential and logarithmic
fn exp(x: f64) -> f64{it > 0.0};
fn log(x: f64{it > 0.0}) -> f64;
fn log10(x: f64{it > 0.0}) -> f64;
fn log2(x: f64{it > 0.0}) -> f64;

-- Rounding
fn floor(x: f64) -> f64;
fn ceil(x: f64) -> f64;
fn round(x: f64) -> f64;
fn trunc(x: f64) -> f64;

-- Special values
fn is_nan(x: f64) -> bool;
fn is_inf(x: f64) -> bool;
fn is_finite(x: f64) -> bool;

-- Constants
fn PI() -> f64 = 3.14159265358979323846;
fn E() -> f64 = 2.71828182845904523536;
fn INF() -> f64;  -- Positive infinity
fn NEG_INF() -> f64;  -- Negative infinity
fn NAN() -> f64;  -- Not a number
```

### Contract Support

```bmb
-- Preconditions with f64
fn safe_sqrt(x: f64) -> f64
  pre x >= 0.0
  post ret >= 0.0
= sqrt(x);

-- Inline refinement types
fn normalize(x: f64, y: f64) -> f64{it >= 0.0, it <= 1.0}
  pre x*x + y*y > 0.0
= let len = sqrt(x*x + y*y);
  x / len;

-- Named contracts
fn clamp(x: f64, lo: f64, hi: f64) -> r: f64
  where {
    bounds: r >= lo and r <= hi,
    identity: (x >= lo and x <= hi) implies r == x
  }
= if x < lo then lo else if x > hi then hi else x;
```

### Type Conversion

```bmb
-- i64 to f64 (lossless for values up to 2^53)
fn i64_to_f64(x: i64) -> f64;

-- f64 to i64 (truncates toward zero)
fn f64_to_i64(x: f64) -> i64;

-- With explicit rounding
fn f64_to_i64_floor(x: f64) -> i64;
fn f64_to_i64_ceil(x: f64) -> i64;
fn f64_to_i64_round(x: f64) -> i64;
```

## Implementation

### Lexer Changes

```
-- New token types
F64_LITERAL    := [0-9]+ '.' [0-9]* ([eE] [+-]? [0-9]+)?
               |  [0-9]* '.' [0-9]+ ([eE] [+-]? [0-9]+)?
               |  [0-9]+ [eE] [+-]? [0-9]+
HEX_FLOAT      := '0x' [0-9a-fA-F]+ '.' [0-9a-fA-F]* 'p' [+-]? [0-9]+
```

### Parser Changes

- Add `f64` to type grammar
- Parse f64 literals as `Expr::F64Literal(f64)`
- Support f64 in binary/unary expressions

### Type Checker Changes

```
-- Type rules
Γ ⊢ e1 : f64    Γ ⊢ e2 : f64
────────────────────────────── (f64-arith)
Γ ⊢ e1 op e2 : f64             where op ∈ {+, -, *, /, %}

Γ ⊢ e1 : f64    Γ ⊢ e2 : f64
────────────────────────────── (f64-cmp)
Γ ⊢ e1 op e2 : bool            where op ∈ {<, <=, >, >=, ==, !=}
```

### MIR Changes

- Add `MirType::F64`
- Add f64 operations: `FAdd`, `FSub`, `FMul`, `FDiv`, `FRem`, `FCmp`
- Add conversion operations: `SIToFP`, `FPToSI`

### LLVM Code Generation

```llvm
; Arithmetic
%result = fadd double %a, %b
%result = fsub double %a, %b
%result = fmul double %a, %b
%result = fdiv double %a, %b
%result = frem double %a, %b

; Comparison (ordered predicates)
%cmp = fcmp olt double %a, %b
%result = uitofp i1 %cmp to double  ; If needed as f64

; Conversion
%f = sitofp i64 %i to double
%i = fptosi double %f to i64

; Math intrinsics
declare double @llvm.sqrt.f64(double)
declare double @llvm.sin.f64(double)
declare double @llvm.cos.f64(double)
declare double @llvm.pow.f64(double, double)
declare double @llvm.exp.f64(double)
declare double @llvm.log.f64(double)
declare double @llvm.floor.f64(double)
declare double @llvm.ceil.f64(double)
```

### SMT Verification

```smt2
; Z3 Real theory for f64 approximation
(declare-fun x () Real)
(declare-fun y () Real)

; Precondition: x >= 0.0
(assert (>= x 0.0))

; Postcondition: sqrt(x) >= 0.0
; Note: Z3 Real theory is an approximation of IEEE 754 semantics
```

**Limitations**:
- Z3 Real theory does not model NaN, Inf, or rounding
- Floating-point edge cases require `@trust` or runtime checks
- Consider using Z3's FPA theory for precise verification (performance cost)

## Alternatives Considered

### 1. f32 Only
- Pro: Smaller memory footprint
- Con: Insufficient precision for scientific computing
- Decision: Start with f64, add f32 later if needed

### 2. Fixed-Point Only
- Pro: Predictable, no floating-point edge cases
- Con: Insufficient range for physics simulations
- Decision: Keep fixed-point for graphics, add f64 for science

### 3. Decimal Type
- Pro: Exact decimal representation
- Con: Slower, not IEEE standard
- Decision: Consider for financial computing in future

## Migration

### Existing Code
No breaking changes. f64 is additive.

### Fixed-Point Benchmarks
Existing fixed-point benchmarks (mandelbrot) remain valid.
New `_fp` variants will use f64 for comparison.

## Test Plan

### Unit Tests
- f64 literal parsing
- Arithmetic operations
- Comparison operations
- Type checking rules
- MIR generation
- LLVM IR output

### Integration Tests
- n_body benchmark (physics)
- mandelbrot_fp benchmark (complex math)
- spectral_norm_fp benchmark (matrix ops)

### Contract Tests
- Precondition checking with f64
- Postcondition verification
- SMT solver integration

## Performance Targets

| Operation | Target vs Rust | Verification |
|-----------|----------------|--------------|
| fadd/fsub/fmul/fdiv | 1.0x (exact parity) | LLVM same backend |
| sqrt/sin/cos | 1.0x | LLVM intrinsics |
| f64 benchmark suite | >= 0.95x Rust | Benchmark Gate #2 |

## Timeline

**Actual Implementation** (2026-01-09):
- Core f64 support was already implemented in the compiler
- v0.34: Added scientific notation to lexer (1 hour)
- stdlib/math/f64: Deferred to future version

| Phase | Task | Planned | Actual |
|-------|------|---------|--------|
| 34.1.1 | Lexer + Parser | 3 days | ✅ Pre-existing |
| 34.1.2 | Type Checker | 3 days | ✅ Pre-existing |
| 34.1.3 | MIR + LLVM | 5 days | ✅ Pre-existing |
| 34.1.4 | SMT Integration | 3 days | ✅ Pre-existing |
| 34.1.5 | Scientific notation | - | ✅ 1 hour |
| 34.1.6 | stdlib/math/f64 | 3 days | 📋 Deferred |
| 34.1.7 | Math intrinsics (sqrt, i64_to_f64, f64_to_i64) | - | ✅ v0.34 |
| 34.1.8 | LLVM IR fixes (double type, float ops) | - | ✅ v0.34 |
| 34.1.9 | Benchmarks (mandelbrot_fp, n_body) | 2 days | ✅ v0.34 |

**Intrinsics Implementation** (v0.34):
- `sqrt(x: f64)` → `@llvm.sqrt.f64` intrinsic declaration + call
- `i64_to_f64(x: i64)` → `sitofp i64 %x to double`
- `f64_to_i64(x: f64)` → `fptosi double %x to i64`

**LLVM IR Fixes** (v0.34):
- Type name: `f64` → `double` (LLVM-compatible)
- Float ops: `add nsw` → `fadd`, `sub nsw` → `fsub`, etc.
- Float literals: `4e0` → `4.000000e+00` (proper format)

## References

- [LLVM Language Reference - Floating Point](https://llvm.org/docs/LangRef.html#floating-point-types)
- [LLVM fcmp codegen](https://theunixzoo.co.uk/blog/2025-04-15-llvm-fcmp-codegen.html)
- [IEEE 754-2019 Standard](https://standards.ieee.org/ieee/754/6210/)
- [Z3 Floating Point Theory](https://microsoft.github.io/z3guide/docs/theories/IEEE%20Floats)
- [Kaleidoscope Tutorial - Code Generation](https://rocm.docs.amd.com/projects/llvm-project/en/latest/LLVM/llvm/html/tutorial/MyFirstLanguageFrontend/LangImpl03.html)

## Appendix: NaN Handling Strategy

### Option A: Propagate NaN (Recommended)
```bmb
-- NaN propagates through operations
let result = 0.0 / 0.0;  -- NaN
let check = result == result;  -- false (NaN != NaN)
```

### Option B: Contract-Based NaN Prevention
```bmb
-- Contracts prevent NaN-producing operations
fn safe_div(a: f64, b: f64) -> f64
  pre b != 0.0
  pre not (a == 0.0 and b == 0.0)
= a / b;
```

### Option C: Option<f64> for Partial Functions
```bmb
fn try_sqrt(x: f64) -> Option<f64> =
  if x >= 0.0 then Option::Some(sqrt(x))
  else Option::None;
```

**Recommendation**: Use Option A as default, encourage Option B/C for safety-critical code.
