# ADR-0153: What self-proof means, and the oracle after the Rust compiler retires

- Status: Accepted
- Date: 2026-11-26
- Related: ADR-0037 decision 4 (self-proof), ADR-0029 (escape hatches), ADR-0111 (decision gates), DESIGN.md §3 principle 4, ROADMAP.md layer 9

## Context

- ADR-0037 defines self-proof as: the compiler written in BMB passes its own proof gate, and the parts that use escape hatches are counted as unproved area under a bound.
- By that definition the port is already close. Every definition of it is proved and none is marked as of unproved termination (ADR-0151). What is proved is safety: indices in range, no overflow, termination. Of about 2,600 function lines in `selfhost/`, 123 lines carry an `ensures`, most of them range conditions (counted with `grep`, 2026-11-25).
- That the port is right rests on the Rust compiler: the differential cases of the gate and of the nightly set compare the two byte for byte. When the Rust compiler retires as the seed, that oracle is gone, and safety proofs do not stop a compiler that is safe and wrong.
- Principle 4 already says that tests remain at trust boundaries, the compiler itself among them. What is open is which part proof carries and which part tests carry once there is no second compiler.

## Decision

1. **Self-proof is safety plus named soundness properties.** Beside the proof of every definition, the compiler carries contracts for, and proves:
   - **P1, the discipline of the verdict:** an obligation is reported proved only from an `unsat` answer of the solver; never from an unknown, a timeout, an error or a limit.
   - **P2, the condition of dropping a check:** a runtime check is dropped only when the key of its obligation is in the set of usable proofs.
   A third property, **P3** (a cached proof is reused only under an equal key, and the key is a function of what was read), is decided after P1 and P2 are written and proved.
2. **Preservation of meaning by the whole compiler is outside self-proof.** It is not claimed. Where proof does not reach, the oracle of decision 4 stands, and the area is reported.
3. **The values follow a minimal experiment.** P1 and P2 are written and proved first; the lines, the proof time and the friction are recorded. A later record then fixes whether P3 is in, and the bounds on unproved area, which is reported as two numbers: definitions that use an escape hatch, and definitions on the path of the named properties that are trusted without a contract.
4. **The oracle after retirement has two parts.**
   - **Frozen expected outputs.** Before retirement the outputs of the Rust compiler on the differential cases become tracked expected outputs (as they are for small ones, as digests for large ones). The gate compares the compiler with them; a commit that changes one gives its reason, as a change of the surface fingerprint does.
   - **Checks that never depended on the Rust compiler's internals.** The conformance suite and the fixed point stay. The soundness fuzzers, which today are Rust tests bound to the Rust compiler's API, are moved to run against the compiler binary before retirement.
5. **The seed is reproduced through a chain of releases.** The seed builds the first release; each release is built by the one before it; CI checks the chain. The compiler's source is not held to the language the seed accepts.
6. **Layer 9 closes when** every definition is proved with no unproved termination; the named properties are proved; unproved area is under its bounds; the frozen outputs stand and the gate compares with them without the Rust compiler; the fuzzers run against the binary; CI reproduces the chain from the seed.

## Rationale

- The three properties are the links of one claim, that the gate does not pass what is not proved (goal G1), and each is a postcondition of one component, so the cost is expected to be low. That expectation is not measured, which is why the values wait for the experiment.
- Idempotence of the normal form, a lossless lexer and invariance of the identity hash under renaming have value too, but each is a proof over the whole syntax tree or over two programs, and the differential cases and the rename experiments already cover them. They are reported, not required.
- Alternative: keep the definition of ADR-0037 as it is. Rejected: the layer would close on safety alone, and the compiler of a language whose principle is that proof replaces tests would, after retirement, be guarded by neither.
- Alternative: prove preservation of meaning end to end. Rejected as outside this project's goals; no comparison with verified compilers was made for this record.
- Alternative for the seed: keep the compiler's source inside the seed's language so one step always rebuilds it. Rejected: the compiler could never use a feature added after the freeze.
