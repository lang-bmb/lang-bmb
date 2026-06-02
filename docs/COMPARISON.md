# Why BMB? — Comparison with Adjacent Languages

This document answers the question *"why would I pick BMB over X?"* for the languages and toolchains BMB is most often compared with. The honest framing: **BMB is not the best choice for every problem these tools solve.** The goal here is to show where BMB has a defensible niche and where you should pick something else.

## TL;DR

| If you want… | Pick |
|--------------|------|
| Memory safety + ecosystem maturity | **Rust** |
| Memory safety + bounded model checking on Rust | **Rust + Kani** |
| Algorithm verification with rich logic, .NET/Go output | **Dafny** |
| Crypto / kernel verification with dependent types | **F*** |
| Decades of safety-critical deployment, Ada heritage | **SPARK** |
| Verified asm for crypto primitives | **Vale** |
| C-class native performance, contract-erased SMT verification, AI-generated code | **BMB** |

BMB's niche: a small intersection of *(systems-level performance) ∩ (compile-time contract verification with full erasure) ∩ (syntax tractable for LLM code generation)*. If you don't need all three, one of the alternatives is probably a better fit.

---

## vs Rust (and Rust + Kani)

### Where Rust wins

- **Ecosystem**: 150,000+ crates, mature async runtimes, production HTTP/gRPC/SQL stacks. BMB has ~14 packages.
- **Borrow checker maturity**: a decade of refinement. BMB's borrow checker is Rust-shaped but younger.
- **Tooling**: `rust-analyzer`, `cargo`, `clippy`, `miri` — best-in-class. BMB's LSP is basic.
- **Community and hiring**: not even close.

If memory safety is your primary requirement and you don't need contract verification beyond what types provide, **use Rust**. BMB is not a Rust replacement.

### Where BMB differs

- **Functional correctness contracts**: Rust's type system catches memory bugs and a handful of API misuse patterns. It does not prove "this binary search returns the index of `target` if present, `-1` otherwise." BMB's `pre`/`post`/`invariant` plus Z3 do.
- **Unchecked-by-default, with an explicit proof obligation**: Rust's optimizer cannot use `// SAFETY: idx < arr.len()` comments — they are documentation. BMB emits no bounds check at all (like unsafe C), and `pre idx < arr.len()` makes the safety obligation explicit and machine-checkable via `bmb verify` (Z3). Feeding those proofs back into codegen is a design goal; today, comparison/range facts are emitted as `llvm.assume`, array-bounds facts are not yet wired in, and verification is not build-enforced.
- **Verbose-by-design syntax for AI generation**: explicit overflow operators (`+%`, `+|`, `+?`), explicit nullable (`T?`), explicit purity (`pure fn`). The hypothesis: LLMs make fewer mistakes when the syntax forces decisions to be visible. *This is a hypothesis under validation, not a proven advantage.*

### Rust + Kani specifically

[Kani](https://model-checking.github.io/kani/) is a bounded model checker that proves memory safety and user-provided assertions on Rust code. It is excellent and complementary to Rust's type system.

Kani vs BMB:

| Property | Kani | BMB |
|----------|------|-----|
| Verification scope | Memory safety + user assertions | Functional correctness via contracts |
| Loop handling | Requires loop bounds (`#[kani::unwind(N)]`) | Recursive contract unfolding |
| Output | Standard Rust binary | LLVM IR with contracts erased |
| Solver | CBMC (SAT-based) | Z3 (SMT) |
| Annotation density | Low (assertions at call sites) | Medium (pre/post on functions) |

If you have a Rust codebase and want stronger guarantees, add Kani. BMB is not the answer to "can I verify my existing Rust code."

---

## vs Dafny

[Dafny](https://dafny.org/) is the closest cousin in terms of verification approach: SMT-driven (Z3), pre/post contracts, automatic verification with manual lemma assistance.

### Where Dafny wins

- **Verification depth**: Dafny's logic is richer (set theory, sequences, multisets, function types as first-class values, lemma functions). BMB stays in a smaller fragment.
- **Maturity**: 15+ years of development at Microsoft Research. Used to verify EthIR, IronFleet, AWS authorization libraries.
- **Tutorials and books**: extensive learning material. BMB is documented but young.

If your problem is "prove this algorithm correct" and you don't care about native binary output, **use Dafny**.

### Where BMB differs

- **Output**: Dafny extracts to .NET / Java / Go / Python — runtime-managed languages with their own GC and overhead. BMB compiles directly to native via LLVM, with contracts erased.
- **Performance integration**: Dafny verifies, then extracts. BMB's contracts feed the optimizer — precondition facts are emitted as `llvm.assume`, enabling LLVM transformations. The two are connected, not sequential.
- **Systems-level types**: Dafny doesn't have raw pointers, SIMD intrinsics, or explicit memory layout. BMB does, because the target is systems code.

The mental model: **Dafny is a verification tool that happens to produce code; BMB is a systems language that happens to verify.** Different priorities, different shapes.

---

## vs F*

[F*](https://fstar-lang.org/) is the heaviest-duty option: dependent types plus SMT, used for HACL* (verified crypto in Mozilla / Linux kernel) and miTLS.

### Where F* wins

- **Logical strength**: dependent types + effects + SMT. Can express and prove things BMB cannot (e.g., precise stateful protocols, separation logic via Steel).
- **Track record in production crypto**: HACL* code runs in Firefox, Linux kernel, Microsoft Azure. This is real verified code in real systems.
- **Effect system**: separates pure / stateful / divergent / concurrent computation at the type level.

If you are writing cryptographic primitives or formally verified protocols where the cost of any bug is catastrophic, **use F***.

### Where BMB differs

- **Annotation cost**: F* programs typically have 3–10× more proof annotations than implementation. BMB targets a 1.2–2× ratio for routine systems code by staying in a narrower logic.
- **Learning curve**: F* requires comfort with dependent types and proof tactics. BMB's contracts read like enhanced assertions.
- **Native compilation**: F* extracts to OCaml or C; BMB targets native LLVM directly.

F* is the right tool when verification correctness matters more than developer ergonomics. BMB makes the opposite trade.

---

## vs SPARK (Ada subset)

[SPARK](https://www.adacore.com/about-spark) is the most-deployed contract-verified language in safety-critical industry — used in avionics (Airbus, Lockheed), rail (Alstom), defense, and security tools (NVIDIA's secure boot, Tweag's tezos work).

### Where SPARK wins

- **Industrial track record**: certified to DO-178C Level A (avionics highest). BMB has none of this.
- **Toolchain**: GNATprove, formally verified extraction, decades of process maturity.
- **Memory model**: provably absence of runtime errors (PARE), tractable subset designed for proof.

If you are building software that must clear safety certification, **use SPARK**. BMB has no certification story.

### Where BMB differs

- **Modern syntax**: SPARK is Ada — a syntactically heavy language designed in the 1980s. BMB syntax is closer to Rust / Swift.
- **LLVM backend**: SPARK uses GNAT (GCC-based). BMB uses LLVM, gaining access to its optimization pipeline and target portability.
- **Open ecosystem**: SPARK's commercial-grade tooling is AdaCore-licensed; community version is more limited. BMB is MIT-licensed throughout.

SPARK and BMB share a verification philosophy. SPARK is the mature, certified, Ada-shaped option. BMB is the modern, LLVM-native, MIT-licensed option — and currently the unproven one.

---

## vs Vale

[Vale](https://github.com/microsoft/vale) is a verification language for crypto primitives that produces verified assembly. It is narrow and excellent at its niche.

### Where Vale wins

- **Verified asm output**: Vale generates x86-64 assembly with side-channel guarantees (constant-time). BMB does not target this.
- **Crypto-specific reasoning**: built-in support for arithmetic over modular fields used in crypto.

If you are writing verified constant-time crypto, **use Vale**. BMB is not the right tool.

### Where BMB differs

- **General-purpose**: BMB is a full systems language; Vale is domain-specific.
- **Verification scope**: Vale verifies low-level asm-style code; BMB verifies high-level functional contracts.

Different problems. Not really a comparison.

---

## What BMB Is Actually Trying to Do

The languages above span a spectrum:

```
General-purpose ←──────────────────────────────────→ Specialized
   Rust              Dafny     F*    SPARK             Vale
                       ↑         ↑     ↑
                    teaching  crypto avionics
```

BMB's claim is that there is room for a **general-purpose systems language with first-class compile-time contract verification, where verification is integrated with optimization rather than bolted on**. Rust + Kani approximates this from one direction; Dafny / F* / SPARK approximate it from the other. None of them sit exactly where BMB is trying to sit.

Whether that niche is large enough to matter is the open question. BMB is at v0.98 with a small ecosystem. The technical work is to validate that the position is real; the social work is to find the users for whom this trade-off is worth it.

If you are evaluating BMB for a real project, the correct mental model is: **this is an experimental language exploring an unproven design point.** Use it if the design point matches your problem and you can absorb the ecosystem cost. Otherwise, one of the alternatives above is the right answer.

## See Also

- [VERIFICATION](VERIFICATION.md) — BMB's verification model in detail
- [TARGET_USERS](TARGET_USERS.md) — who BMB is and is not for
- [ROADMAP](ROADMAP.md) — what is changing and when
