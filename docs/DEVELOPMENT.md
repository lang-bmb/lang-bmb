# BMB Development Guidelines

> **Performance > Everything**

This document defines the development philosophy and decision framework for BMB compiler development.

---

## Core Principle

**Performance degradation is a bug, not a trade-off.**

When BMB code runs slower than equivalent C code, it's a compiler bug that must be fixed. There is no acceptable "overhead" for safety or convenience.

---

## Development Principles

### 1. No Workarounds

BMB is a programming language, not an application. Every workaround becomes technical debt that affects thousands of programs written in BMB.

| Application Project | BMB (Language Project) |
|--------------------|------------------------|
| Quick fixes OK | Root cause fixes only |
| Workarounds acceptable | Workarounds forbidden |
| Refactor later | Fix it now |
| Avoid spec changes | Change spec when needed |

### 2. Complexity is Not an Excuse

If the proper fix requires:
- Language spec change → **Change the spec**
- Compiler restructure → **Restructure the compiler**
- MIR/AST modification → **Modify MIR/AST**
- Large-scale refactoring → **Execute the refactoring**

The scope of work is never a valid reason to choose an inferior solution.

### 3. "Language Limitation" is Not an Answer

BMB is the language we're building. If there's a limitation, we remove it.

```
WRONG: "This is a language design limitation"
RIGHT: "The language needs this feature. Let's add it."
```

### 4. Evidence Before Claims

Never claim something is "fixed" or "optimized" without verification:
- Run benchmarks before claiming performance improvements
- Compare assembly output to verify optimizations
- Test edge cases before claiming correctness

---

## Decision Framework

When facing a performance or correctness issue, evaluate solutions in this order:

| Priority | Level | Question | Example |
|----------|-------|----------|---------|
| 1 | **Language Spec** | Does the language need this feature? | while loops, pattern matching |
| 2 | **Compiler Structure** | Does the compiler need restructuring? | New optimization pass |
| 3 | **Optimization Passes** | Can we add/improve an optimization? | LICM, tail call elimination |
| 4 | **Code Generation** | Can we generate better code? | LLVM IR patterns |
| 5 | **Runtime** | Does the runtime need changes? | Intrinsic functions |

**Always start from level 1.** The temptation to solve problems at lower levels (easier, less work) must be resisted when a higher-level solution is the proper fix.

---

## Example: Recursive Function Performance

**Problem**: Recursive functions run 2x slower than C loops.

### Wrong Approach (Bottom-up)

```
Level 5 (Runtime): "Add a trampoline runtime function"
Level 4 (Codegen): "Add alwaysinline attribute"
Level 3 (Optimization): "Improve tail call detection"
```

These are band-aids that don't address the root cause.

### Correct Approach (Top-down)

```
Level 1 (Language): "Does BMB need loop constructs?"
Answer: Yes. Recursion has fundamental overhead that TCO can't eliminate.
Action: Add while/loop to the language spec.
```

The fix requires: Parser → AST → Type checker → MIR → Codegen → Bootstrap compiler.

This is significant work. **Do it anyway.**

---

## Benchmark-Driven Development

Performance claims must be verified through benchmarks:

```bash
# Run full benchmark suite
cargo run --release --features llvm -- bench

# Compare against C
./ecosystem/benchmark-bmb/run.sh

# Check specific benchmark
./ecosystem/benchmark-bmb/run.sh mandelbrot
```

### Performance Gates

| Gate | Criteria | Action if Failed |
|------|----------|------------------|
| #3.1 | All benchmarks ≤ 1.10x vs C | Block release |
| #3.2 | Average ≤ 1.05x vs C | Investigate |
| #3.3 | At least 3 benchmarks faster than C | Investigate |

---

## Code Review Checklist

When reviewing PRs, verify:

1. **No Workarounds**: Is this a proper fix or a band-aid?
2. **Right Level**: Is the fix at the appropriate abstraction level?
3. **Benchmarked**: Has performance impact been measured?
4. **Complete**: Does this fix all instances of the problem?

---

## Common Anti-Patterns

### "LLVM will optimize it"

LLVM is powerful but not omniscient. If BMB generates suboptimal IR, LLVM often cannot recover. The compiler must generate optimal IR in the first place.

### "Good enough for now"

There is no "for now" in a language compiler. Today's compromise becomes tomorrow's compatibility burden.

### "The benchmark is unfair"

If a benchmark shows BMB is slower, the response is to make BMB faster—not to question the benchmark. (Unless the benchmark is genuinely measuring something irrelevant.)

### "Users can work around it"

Users shouldn't need to work around compiler limitations. If there's a pattern that compiles to slow code, fix the compiler.

---

## Related Documents

- [SPECIFICATION.md](SPECIFICATION.md) - Language specification
- [ARCHITECTURE.md](ARCHITECTURE.md) - Compiler architecture
- [ROADMAP.md](ROADMAP.md) - Development roadmap
