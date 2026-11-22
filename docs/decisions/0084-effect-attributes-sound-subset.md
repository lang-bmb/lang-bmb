# ADR-0084: LLVM attributes from effects are only the sound subset, and implementation waits for measured evidence

- Status: Accepted (approved by the maintainer 2026-11-02 "recommendation adopted for the decision items". Supersedes ADR-0030 §6)
- Date: 2026-11-02
- Related: ADR-0030 §6·§8, ADR-0029 (trust levels), ADR-0063 (incremental IR fragments), ADR-0041 (performance baseline), backlog 2F-04

## 1. Problem

ADR-0030 §6 decided to emit LLVM function attributes from effects (`nounwind` on every function, `willreturn`·`mustprogress` when there is no `panic` and it is `proved`, `memory(argmem: read)`/`memory(none)` when the effects are empty, with `inaccessiblemem: readwrite` added for `alloc`). The review before starting (2026-11-02) found three cases in which emitting them as written gives a wrong attribute. A wrong attribute is undefined behaviour to LLVM, and optimization can miscompile a proved program.

1. `memory(argmem: read)` covers only memory reached directly from pointer arguments. A function that takes a `Vec`·`Box`, or a struct or enum containing one, as an argument and reads an element reads the heap through a pointer taken out of the header. That is not argument memory.
2. An `alloc` function writes a newly allocated buffer and returns it. `inaccessiblemem: readwrite` does not cover that. It also `abort`s on allocation failure, so `willreturn` is false.
3. `willreturn` is true only if not just the function itself but also its callees leave no checks (traps). Relying on the callees' check counts means the callees must enter the reuse key of the incremental IR fragments (ADR-0063). Also, a function that returns its result through memory (`ret.out`) writes to argument memory.

## 2. Cause and the layer it belongs to

The design layer (the rule of ADR-0030 §6). The compiler does not emit attributes yet, so there is no wrong code now.

## 3. Standard solution and source

The definitions of `memory(..)`·`willreturn`·`nounwind` in the LLVM LangRef. Rust emits only argument attributes such as `readonly`/`noalias` and leaves function memory attributes to LLVM inference (`FunctionAttrs`).

## 4. Comparison of candidates

| Candidate | Soundness (C1) | Cost | Verdict |
| --- | --- | --- | --- |
| (A) §6 as is | wrong in the three cases above | — | rejected |
| (B) the sound subset | pass | the callee condition enters the incremental key | adopted |
| (C) emit nothing and leave it to LLVM inference | pass | 0 | the state until the gain of (B) is measured |

## 5. Decision

- These are the only attributes that may be emitted.
  - `nounwind`: every function (BMB has no unwinding).
  - `memory(none)` / `memory(argmem: read)` / `memory(argmem: readwrite)` if the result is returned through memory: a function whose effects are empty (`pure`, no `panic`·`alloc`), that has no `&mut` argument, and that has no heap pointer (`Vec`, `Box`, a type containing one) anywhere in its argument, local or result types.
  - `willreturn`·`mustprogress`: in addition to the conditions above, a function that leaves no checks and all of whose callees satisfy the same condition (transitively). This condition of the callees enters the key of the incremental IR fragments.
- For `alloc`·`io`·`panic` functions and functions with checks left (a trust level other than `proved`, or unproved obligations), nothing but `nounwind` is emitted.
- The implementation (2F-04) is put on hold until there is measured evidence of a speed gain. The evidence is the `bench/baseline` time of (B) against (C) (ADR-0041 conditions). If there is no gain, it stays at (C).
- The dynamic effect fuzzer of ADR-0030 §8 remains regardless of this decision (it can be split off from 2F-04).

## 6. Impact (information)

Code generation (function header attributes), the incremental IR key (the callee condition), IR tests. Nothing changes before the implementation.

## 7. Approval needed

It changes one section of ADR-0030 (accepted), so approval. Approved 2026-11-02.
