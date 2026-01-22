# BMB — Bare-Metal-Banter

> **Banter for AI. Bare-metal for humans.**

A contract-verified systems programming language.

---

## The Trade-off

Most languages optimize for humans. BMB doesn't.

| You Give Up | You Get |
|-------------|---------|
| More type annotations | More aggressive optimization |
| Contracts required | Runtime checks eliminated |
| Explicit conversions | Predictable performance |
| More compile errors | Fewer runtime errors |

**Hard to write. Hard to get wrong. And that's what AI prefers.**

---

## Zero-Overhead Safety

Every safety check compiles away to nothing:

| Runtime Check (Other Languages) | BMB Approach | Overhead |
|---------------------------------|--------------|----------|
| Bounds checking | `pre idx < arr.len()` | **0%** |
| Null checking | `T?` type + contract | **0%** |
| Overflow checking | Contract or explicit op | **0%** |
| Division by zero | `pre divisor != 0` | **0%** |

```bmb
fn get(arr: &[i32], idx: usize) -> i32
  pre idx < arr.len()
= arr[idx];
```

This generates **identical assembly** to unchecked C. The proof happens at compile-time. The runtime cost is zero.

---

## Why AI-First?

| | Humans | AI |
|---|--------|-----|
| Verbose types | Annoying | Trivial |
| Explicit contracts | Tedious | Natural |
| Formal proofs | Difficult | Preferred |
| No shortcuts | Frustrating | Irrelevant |

Traditional languages hide complexity to help humans. BMB exposes everything—because AI handles verbosity effortlessly.

---

## A Taste of BMB

### Contracts as Code

```bmb
fn binary_search(arr: &[i32], target: i32) -> usize?
  pre is_sorted(arr)
  post ret.is_none() implies forall i: 0..arr.len(). arr[i] != target
  post ret.is_some() implies arr[ret.unwrap()] == target
{
    // implementation
}
```

Pass an unsorted array? **Compile error.**

### Overflow—Your Choice

```bmb
let a = x + y;      // requires contract proving no overflow
let b = x +% y;     // wrapping (mod 2^n)
let c = x +| y;     // saturating (clamp to bounds)
let d = x +? y;     // checked (returns T?)
```

No silent overflow. No debug/release differences. You decide the semantics.

### Pure Functions

```bmb
pure fn square(x: i64) -> i64 = x * x;
```

The compiler guarantees: no side effects, deterministic output. Safe for memoization, reordering, parallelization.

### Refinement Types

```bmb
type NonZero = i64 where self != 0;
type Percentage = f64 where self >= 0.0 and self <= 100.0;
```

Types that carry their own proofs.

---

## Quick Start

```bash
bmb run examples/hello.bmb        # run
bmb check examples/simple.bmb     # type check
bmb verify examples/contracts.bmb # prove contracts (requires Z3)
bmb build examples/hello.bmb -o hello  # compile native (requires LLVM)
bmb repl                          # interactive
```

---

## Performance

When the compiler knows your invariants, it knows what's safe to optimize.

```
                     C          Rust       BMB
─────────────────────────────────────────────────
fibonacci(45)        1.65s      1.66s      1.63s
mandelbrot           42ms       42ms       39ms
spectral_norm        44ms       44ms       39ms
```

BMB's goal: safe code that generates **identical assembly** to unsafe C.

---

## Documentation

| | |
|---|---|
| [Specification](docs/SPECIFICATION.md) | Formal language definition |
| [Language Reference](docs/LANGUAGE_REFERENCE.md) | Complete feature guide |
| [Architecture](docs/ARCHITECTURE.md) | Compiler internals |
| [Tutorials](docs/tutorials/) | Getting started |

---

## License

MIT

---

<p align="center">
  <b>Performance + Stability > Human Convenience</b>
</p>
