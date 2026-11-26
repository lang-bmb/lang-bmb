# ADR-0162: The values of self-proof after the minimal experiment

- Status: Accepted
- Date: 2026-11-26
- Related: ADR-0153 decisions 1, 3 and 6 (the named properties, values after an experiment, when layer 9 closes), ADR-0029 (escape hatches), ADR-0032 (the proof cache), ROADMAP.md layer 9

## Context

- ADR-0153 named two soundness properties (P1, the discipline of the verdict; P2, the condition of dropping a check), left a third (P3, the proof cache) and the bounds on unproved area to a record written after P1 and P2 were stated and proved.
- They are stated and proved in the compiler written in BMB: 87 lines net in three files; the resource units of its whole proof went from 676,585,209 to 681,574,025 (+0.74%), its obligations from 23,393 to 23,436; one proof failed on the way (a helper that changes another field of a value with a type invariant has to reveal the invariant). No output changed.
- What is proved is a chain of links, not one statement from end to end, because a contract cannot speak of what was read from a process:
  - the verdict unsat is made from the word `unsat` and from nothing else (`solver::verdict_word`);
  - an answer has kind 0 for the verdict unsat and for nothing else (`prove::ans_of`);
  - a `Verdict` that is proved carries kind 0 as the kind of the answer that decided it (its type invariant);
  - a yes to "is this check's obligation proved" has a witness among the usable proofs (`codegen::Gen::proved`).
- Between the links some steps are read, not proved (decision 3 lists them). An obligation found in the proof cache counts as proved without an answer of its own, so P1 leans on the cache.
- No definition of that compiler uses an escape hatch, and none has unproved termination.

## Decision

1. **P3 is a criterion of layer 9.** A cached proof is reused only under an equal key, and the key is a function of what was read. Without it the chain of P1 has a link nothing proves. It is stated and proved as P1 and P2 were; its lines, cost and friction are recorded when it is.
2. **The bound on definitions that use an escape hatch is 0**, the present value, and likewise on definitions with unproved termination.
3. **The steps trusted without a contract are a closed list.** Layer 9 closes with this list or a shorter one; a step is added only by a record.
   - `prove::decide` and `prove::decide_sub` store in `Verdict.by` the kind of the answer that decided the obligation.
   - `solver::check_sat` returns the verdict `solver::verdict_word` gives.
   - The text read from the solver's process is the solver's answer.
   - `code` and the position passed to `codegen::Gen::proved` are those of the check being dropped, and the code generated after a yes has no such check.
   - The solver's uses outside the gate (`lint`, `stability`, `examples`) are advice, outside the properties.
4. **The first step of that list stays trusted for now.** Proving it means making the verdict a pure function of the answers, and the order of the queries of `prove::decide` depends on the answers before. It is looked at again before layer 9 closes.

## Rationale

- The cost of P1 and P2 was as low as ADR-0153 expected, so the reason to hold P3 back (an unmeasured cost) is gone, and P1 already names the cache as a source of a proved obligation.
- A bound of 0 on escape hatches costs nothing today and keeps it so. A numeric bound on the trusted steps would count unlike things; a closed list says what is trusted and makes an addition visible.
- Alternative: close the chain now by restructuring `prove::decide`. Not now: a large change to the part whose behaviour the differential cases pin, for one link of five.
- Alternative: leave P3 out and report the cache as trusted. Rejected: a wrong cache hit passes what was never proved, the failure the three properties exist to exclude.

## Consequences

- The escape-hatch bound and the closed list are not yet checked by the gate. Until they are, this record is where they stand.
- Layer 9's conditions (ADR-0153 decision 6) read "the named properties" as P1, P2 and P3.
