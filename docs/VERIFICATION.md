# Verification Model

This document describes how BMB proves contracts at compile time, what happens when the SMT solver cannot decide, and how BMB compares to other contract-verified languages.

## Goals

BMB's contract system has two non-negotiable properties:

1. **Verified contracts produce zero runtime overhead.** A `pre`/`post`/`invariant` that the compiler proves is erased entirely from the generated code.
2. **Unverified contracts cannot silently pass.** If the prover cannot establish a contract, the build fails — there is no runtime fallback that turns a contract into a hidden assertion.

These two together rule out the design used by `@check`-style annotations in earlier BMB versions, where SMT timeout would fall back to a runtime check. That design was removed by [RFC-0003](rfcs/RFC-0003-Remove-Check.md).

## The Decision Procedure

```
┌─────────────────────────────────────────────────────────────┐
│  Source contract  →  SMT-LIB2 encoding  →  Z3                │
└─────────────────────────────────────────────────────────────┘
                              │
              ┌───────────────┼───────────────┐
              ▼               ▼               ▼
         proved         disproved        unknown / timeout
              │               │               │
              ▼               ▼               ▼
     compile succeeds   compile error    compile error
     (runtime erased)   (counterexample) (with hint)
```

| Outcome | Build | Runtime cost |
|---------|-------|--------------|
| `proved` | Succeeds | **Zero** — contract erased |
| `disproved` | Fails with counterexample | N/A |
| `unknown` | Fails with hint | N/A |
| `timeout` | Fails (default) or warns (configurable) | N/A |

## Escape Hatches

Real programs contain conditions outside Z3's decidable fragment (deep arithmetic, transcendentals, unbounded recursion). BMB provides two tools, both explicit.

### `@trust "reason"`

```bmb
@trust "termination follows from well-founded order on (m, n); cf. ackermann_termination.lean"
fn ackermann(m: i64, n: i64) -> i64
  pre m >= 0 and n >= 0
  post ret >= 0
= if m == 0 { n + 1 }
  else if n == 0 { ackermann(m - 1, 1) }
  else { ackermann(m - 1, ackermann(m, n - 1)) };
```

`@trust` skips SMT verification for the annotated function. The reason string is **mandatory** and surfaces in tooling output (`bmb verify --report json` includes a `trust_reasons` array). The intent: a human reviewer takes responsibility, and that responsibility is auditable.

A `@trust` is not a runtime check. The contract is still erased at codegen — `@trust` only suppresses the proof obligation, it does not insert an assertion.

### Configurable timeout policy

```toml
# bmb.toml
[smt]
timeout_ms = 5000           # per-query timeout (default: 5000)
timeout_action = "error"    # "error" | "trust_with_warning"
quantifier_depth = 3        # MBQI quantifier instantiation depth
```

| `timeout_action` | Behavior on timeout | Use case |
|------------------|---------------------|----------|
| `error` (default) | Build fails | Production, strict CI |
| `trust_with_warning` | Compile succeeds with a warning | Incremental migration of legacy code |

`trust_with_warning` is **not** silent — every timeout becomes a warning that includes the function name, contract, and elapsed time. Treat its presence in CI as a defect.

## Decidable Fragment

BMB's contract language is a subset of SMT-LIB QF_LIA + QF_BV + select extensions. What Z3 can decide reliably:

| Fragment | Decidability | BMB usage |
|----------|--------------|-----------|
| Linear integer arithmetic (LIA) | Decidable | bounds, indices, sizes |
| Bit-vector arithmetic (BV) | Decidable | overflow, shifts, masks |
| Arrays (with select/store) | Decidable (QF_AX) | array contracts |
| Uninterpreted functions (UF) | Decidable (QF_UF) | abstract predicates |
| Nonlinear integer arithmetic (NIA) | Semi-decidable | use sparingly |
| Transcendentals (sin, exp, log) | Undecidable | requires `@trust` |
| Quantifiers (∀, ∃) | Semi-decidable (MBQI) | depth-limited |
| Recursive functions in contracts | Semi-decidable | unfolding-bounded |

Practical guidance:
- Linear arithmetic on `i32`/`i64` indices, sizes, and offsets — almost always proves in <100ms.
- Floating-point contracts use SMT theory of FP (`QF_FP`); precise but expensive. Prefer interval-style bounds (`x >= 0.0 and x <= 1.0`) over exact algebraic identities.
- Quantified contracts (`forall i. 0 <= i < n implies arr[i] > 0`) work, but limit to one quantifier where possible.

## Comparison with Other Verified Languages

The verified-systems space is small and well-studied. BMB's position relative to the main alternatives:

| Language | Logic | Erasure | Escape hatch | Primary domain |
|----------|-------|---------|--------------|----------------|
| **BMB** | SMT (Z3) | Always erased | `@trust "reason"` | Systems / performance-critical |
| Dafny | SMT (Z3) | Always erased | `assume`, `{:axiom}` | Algorithm verification, teaching |
| F* | Dependent types + SMT | Erased after extraction | `admit`, `assume` | Crypto, kernels (e.g., HACL*) |
| SPARK | Custom prover (GNATprove) | Always erased | `pragma Assume` | Avionics, defense (Ada subset) |
| Rust + Kani | Bounded model checking | Verified separately | Loop bounds | Memory safety verification |
| Vale | SMT + custom | Custom IR → asm | Manual proofs | Crypto primitives |

Where BMB differs:

- **vs Dafny**: Dafny targets algorithm correctness with .NET/Java/Go output. BMB targets native code and integrates contract erasure with LLVM optimization (LICM, CSE, vectorization gain information from proved preconditions).
- **vs F***: F* uses a richer dependent type system that catches more, but requires more annotations and produces extracted code. BMB stays in the decidable SMT fragment to keep proof obligations tractable for routine systems code.
- **vs SPARK**: SPARK is the closest cousin — Ada subset with theorem proving, used in real safety-critical systems. BMB's distinction is the SMT-driven optimization pipeline (contracts inform LLVM directly) and a syntax oriented toward AI code generation.
- **vs Rust + Kani**: Kani uses bounded model checking on Rust source for memory safety properties. It is complementary to Rust's type system, not a substitute. BMB's contracts cover functional correctness (post-conditions) which Kani does not target.

The honest summary: **BMB is not a research breakthrough in verification logic.** Z3 is the same solver Dafny uses; the SMT fragment is similar. BMB's bet is that pairing standard SMT verification with an LLVM-native pipeline and AI-friendly syntax makes contract verification practical for systems work in a way that the existing tools — most aimed at algorithms, kernels, or teaching — do not.

## Tooling

```bash
bmb verify file.bmb                  # verify all contracts
bmb verify file.bmb --function f     # single function
bmb verify file.bmb --timeout 30000  # raise per-query timeout (ms)
bmb verify file.bmb --report json    # machine-readable report
bmb verify file.bmb --counterexample # show counterexample on failure
```

Report shape (`--report json`):

```json
{
  "verified": [{"function": "binary_search", "elapsed_ms": 42}],
  "failed":   [{"function": "f", "reason": "post-condition", "counterexample": {"x": -1}}],
  "unknown":  [{"function": "g", "reason": "timeout", "elapsed_ms": 5000}],
  "trusted":  [{"function": "ackermann", "reason": "termination follows from..."}]
}
```

CI consumers should fail the build on any non-empty `failed` or `unknown` array. `trusted` entries should be reviewed periodically — every entry is a deferred proof obligation.

## See Also

- [SPECIFICATION §6](SPECIFICATION.md) — formal contract semantics
- [ADVANCED_CONTRACTS](tutorials/ADVANCED_CONTRACTS.md) — practical patterns
- [CONTRACT_PROGRAMMING](tutorials/CONTRACT_PROGRAMMING.md) — introductory tutorial
- [RFC-0003](rfcs/RFC-0003-Remove-Check.md) — rationale for removing runtime fallback
- [RFC-0008](rfcs/RFC-0008-contract-driven-optimization.md) — using contracts to drive LLVM optimization
