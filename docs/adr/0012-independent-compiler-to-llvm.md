# ADR-0012: An independent compiler without rustc, straight to LLVM

- Status: Accepted. "Self-hosting is decided after the semantics are stable" is superseded by ADR-0037 (self-hosting is mandatory)
- Date: 2026-10-24
- Related: DESIGN.md §1, principle 9, genesis Q10

## Context
BMB is a Rust dialect. How much of it is implemented by the project itself?

## Decision
Everything from the parser to LLVM IR generation is a **fully independent implementation**. It does not depend on rustc. The external dependencies are Z3 and LLVM.

## Rationale
Alternatives considered:

| Option | Verdict |
| --- | --- |
| Sit on top of Verus, with BMB as lints and tools | Rejected: not independent enough |
| rustc driver (reuse the parser, types and borrow checker) | Rejected: cost of tracking nightly internal APIs, limits on syntax extension |
| Independent frontend + Rust output, then a rustc build | Rejected: depends on rustc |
| **Fully independent (straight to LLVM)** | **Accepted**: connects proofs directly to optimization (principle 9) |

## Consequences
- The scope of the compiler implementation is large (v0: 144K lines). Mitigations: a smaller borrow checker through second-class references (ADR-0015), one compiler and one backend, starting from the minimal end-to-end path (ADR-0016).
- Self-hosting is decided after the semantics are stable.
