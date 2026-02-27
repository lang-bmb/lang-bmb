# BMB Target Users

This document defines who BMB is for, who it's not for, and why. It serves as a guide for prioritizing features, writing documentation, and evaluating design decisions.

---

## Primary Personas

### Persona A: Performance-Critical Numeric Computing

**Who**: Engineers writing compute-heavy code — scientific computing, simulation, signal processing, financial modeling.

**Why BMB**: These users need C-level performance but want stronger correctness guarantees. BMB's contracts eliminate bounds checks and null checks at runtime while proving safety at compile time. The result is code that is both fast and provably correct.

**Current fit**: Good. BMB achieves C-parity on numeric benchmarks (67/67 within 10% of Clang -O3). Contracts work for array bounds and numeric invariants. The language has all the numeric types and operators needed for this domain.

**What's missing**: No SIMD intrinsics. No GPU compute. Standard library is small (~14 packages). No large-scale matrix/tensor library.

**Example use case**:
```bmb
fn dot_product(a: &[f64], b: &[f64]) -> f64
  pre a.len() == b.len()
= {
    let mut sum = 0.0;
    let mut i = 0;
    while i < a.len() {
        sum = sum + a[i] * b[i];  // No bounds check — proven by pre
        i = i + 1
    };
    sum
};
```

### Persona B: Safety-Critical Systems

**Who**: Engineers in domains where software defects have serious consequences — avionics, medical devices, automotive, industrial control.

**Why BMB**: These domains require both formal verification and bare-metal performance. BMB's compile-time contract proofs provide formal guarantees without runtime overhead. Unlike runtime verification (assertions, exceptions), BMB's contracts are mathematically proven before the program runs.

**Current fit**: Promising but early. Contract verification with Z3 works. The compiler generates zero-overhead code for verified contracts. However, the ecosystem is too young for production safety-critical deployments.

**What's missing**: No certification (DO-178C, ISO 26262). No formal semantics document suitable for certification bodies. No track record in production. Ecosystem needs to mature significantly.

**Timeline**: This persona becomes realistic after v0.97+ (ecosystem maturity + formal verification tooling).

**Example use case**:
```bmb
fn altitude_check(current: f64, minimum: f64) -> bool
  pre minimum > 0.0
  pre current >= 0.0
  post ret == true implies current >= minimum
= current >= minimum;
```

### Persona C: AI Code Generation Pipelines

**Who**: Teams building systems where AI (LLMs) generates source code that must be compiled and executed — automated programming, code synthesis, AI-assisted development tools.

**Why BMB**: BMB's explicit, verbose syntax is precise for code generation. Every type is explicit, every contract is declared, every conversion is visible. This reduces ambiguity for AI generators. The compile-time verification catches AI mistakes before execution.

**Current fit**: Experimental. The hypothesis — that explicit syntax makes AI-generated code more correct — is under validation. No published benchmarks comparing AI accuracy on BMB vs other languages yet.

**What's missing**: Published evidence that AI generates more correct BMB code. Integration with major AI code generation frameworks. Tooling for AI-in-the-loop compilation workflows.

**Example workflow**:
```
LLM generates BMB source
    → bmb verify (Z3 checks contracts)
    → Compile error? → Feed error back to LLM
    → Verified? → bmb build → native binary
```

---

## Secondary Personas

### Persona D: Language Enthusiasts and Researchers

**Who**: People interested in programming language design, formal methods, compiler construction, or type theory.

**Why BMB**: BMB explores a specific design point — what happens when you make compile-time proofs the primary mechanism for both safety and performance? The self-hosting bootstrap compiler, the SMT-based verification, and the zero-overhead contract system are interesting from a PL research perspective.

**Current fit**: Good for exploration. The compiler source is open, the architecture is documented, and the bootstrap process is verifiable. Not suitable for academic publication without peer review of the formal foundations.

### Persona E: Systems Programmers Exploring Alternatives

**Who**: C/C++ or Rust programmers looking at new systems languages (Zig, Odin, Vale, etc.).

**Why BMB**: Different trade-offs than Rust (contracts vs ownership) and more safety than C. If you're evaluating systems languages, BMB offers a distinct approach worth examining.

**Current fit**: Early. BMB is competitive on benchmarks but the ecosystem is small. Worth watching, not yet worth migrating production code to.

---

## Who BMB Is NOT For

| User Type | Why Not | Use Instead |
|-----------|---------|------------|
| **Beginners learning to program** | BMB requires understanding of types, memory, contracts. No gentle on-ramp. | Python, JavaScript |
| **Web application developers** | No HTTP server, no web framework, no ORM. | Go, TypeScript, Rust |
| **Mobile app developers** | No iOS/Android toolchain, no UI framework. | Swift, Kotlin |
| **Data scientists** | No dataframes, no plotting, no notebook integration. | Python, R, Julia |
| **Rapid prototypers** | Explicit types and contracts slow initial development. BMB is for when you know what you want. | Python, Ruby |
| **Large team projects (today)** | Ecosystem too young, community too small for production team adoption. | Rust, Go, C++ |

---

## Decision Matrix

When should you consider BMB?

| Factor | BMB is a good fit if... | BMB is NOT a good fit if... |
|--------|------------------------|---------------------------|
| **Performance** | You need C-level speed with formal safety | Performance is "good enough" (use Go/Java) |
| **Correctness** | Bugs have serious consequences (safety-critical) | You can tolerate occasional runtime errors |
| **Code author** | AI generates the code (explicit syntax helps) | Humans write all code (verbosity hurts) |
| **Ecosystem needs** | Minimal dependencies (numeric, core algorithms) | You need a large package ecosystem |
| **Team size** | Solo or small team, willing to contribute | Large team needing stable, proven tools |
| **Timeline** | Research, prototyping, or long-term investment | Ship to production next quarter |

---

## Comparison with Alternatives

| Language | Safety Mechanism | Runtime Cost | Ecosystem | Best For |
|----------|-----------------|-------------|-----------|----------|
| **C** | None | 0% | Massive | Legacy, embedded, OS kernels |
| **C++** | Optional (sanitizers) | Variable | Massive | Games, finance, large systems |
| **Rust** | Ownership + borrow checker | 0% (most) | Large | Safe systems programming |
| **Zig** | Comptime + safety checks | Low | Growing | C replacement, embedded |
| **Go** | GC + runtime checks | >0% | Large | Servers, CLI tools |
| **BMB** | Compile-time contract proofs | **0%** | Small (~14 packages) | Numeric, safety-critical, AI pipelines |

**BMB's unique position**: Compile-time proofs as the primary safety mechanism, targeting zero runtime overhead. This is a different approach from Rust (ownership), Zig (comptime), and Go (GC).

**BMB's honest weakness**: Ecosystem size. Rust has crates.io with 150K+ packages. BMB has ~14. This gap matters more than any language feature for most real-world projects.

---

## How This Document Guides Development

When making design decisions, ask:

1. **Does this help Persona A, B, or C?** If yes, it's in scope.
2. **Does it help none of our personas?** Reconsider priority.
3. **Does it help a persona we explicitly excluded?** It's out of scope (e.g., adding a web framework).

When writing documentation, ask:

1. **Which persona is the reader?** Adjust depth and examples accordingly.
2. **What does this persona already know?** (A knows numeric computing, B knows formal methods, C knows AI pipelines)
3. **What do they NOT know?** (Probably BMB-specific syntax and contracts)

---

## Current Status (v0.93)

| Persona | Readiness | Blockers |
|---------|-----------|----------|
| A: Numeric Computing | **Usable for evaluation** | Small stdlib, no SIMD |
| B: Safety-Critical | **Too early** | No certification, young ecosystem |
| C: AI Pipelines | **Experimental** | No published evidence, no framework integration |
| D: Researchers | **Usable** | No formal semantics paper |
| E: Systems Explorers | **Worth watching** | Small ecosystem, single platform |

BMB is honest about where it stands. The compiler works, benchmarks are competitive, and the contract system is real. But the ecosystem is young and the community is small. Contributions are welcome.
