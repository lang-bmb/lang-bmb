# ADR-0045: Judgement criteria — fix at the layer of the cause and choose the standard solution; cost is not a criterion

- Status: Accepted (2026-10-26, the maintainer's instruction)
- Date: 2026-10-26
- Related: `dev-docs/JUDGMENT.md`, DESIGN.md §2·§3·§4.1, ADR-0041, `dev-docs/AUTONOMY.md` §1·§4

## Context
The grounds of recommendations were not consistent. Some recommended avoiding a compiler limit by editing the corpus (`k*k` → `3*k` in 2G-05, changing the starting point of `primes`), or covering a limit of the language's expressiveness with a workaround (a monomorphic map without user generics). The amount of work and the scope of impact acted as implicit criteria. The maintainer instructed that the judgement criteria be aligned explicitly.

## Decision
1. The judgement criteria are kept in one place, `dev-docs/JUDGMENT.md`. Every recommendation, design decision, and task derivation and review is judged by those criteria, and the number of the criterion applied is written down.
2. A problem is fixed at the layer where the cause is (language, compiler, tools, process, assets, external, user). Symptoms are not absorbed at another layer.
3. The solution is chosen among the candidates that pass the constraints (soundness, determinism, honesty, definition), in the order standardness → fundamentality and generality → consistency → objective function → evidence.
4. The amount of work, the scope of impact, preserving existing assets, agreement with current rules, whether approval is needed, and metric values are not criteria for choosing a solution.
5. During 0.x (before the stability policy 5B-01 is accepted), refactoring is always allowed, with no backward compatibility duty.
6. The accepted item "limited expressiveness" of ADR-0041 means "only what cannot in principle be a proof target is left to an escape hatch with a rationale". An intended tightening must be in the ledger of DESIGN §4.1, and any other limit is a defect.

## Rationale
- If a symptom is absorbed at another layer, the cause remains, problems of the same class recur, and the measurement hides the real capability (when benchmarks and the corpus are fitted to the compiler, C3 honesty breaks too).
- BMB's user is an agent and the surface succeeds Rust. A language that cannot express the standard solution passes the loss of analogy and the cost of workaround code on to the agent.
- In 0.x there are no external users, so there is no compatibility cost. A structure not put right now becomes a cost after the stability policy.
- Alternatives considered: (1) Mix the criteria into AUTONOMY.md: "who decides" and "what is best" get mixed, and whether approval is needed seeps into the choice of solution. (2) Keep cost only as a tie-breaking criterion: ties are rare, and the moment it is used as a criterion it becomes an implicit criterion. A tie is broken by generality and simplicity.

## Consequences
- Among existing decisions, those that absorbed a compiler limit into the corpus and those that left a limit of expressiveness to a workaround are reviewed again by these criteria. If an accepted ADR has to change, it is done with a new ADR.
- A note pointing to this ADR is attached to "limited expressiveness" in the ledger of DESIGN.md §2.
- Recommendations get an "impact" field, but it is not used as grounds for judgement.
