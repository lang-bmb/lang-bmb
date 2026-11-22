# ADR-0137: The evidence premise of gate A is narrowed to "what can change the accepted language"

- Status: Accepted (approved by the maintainer 2026-11-14 "recommendation accepted and approved", backlog `4D-14`)
- Date: 2026-11-14
- Related: ADR-0100 (stage0 feature freeze), ADR-0103 (evidence stage before the freeze), ADR-0105 (scope of stage 3, decision 4), ADR-0111 (verdict gates), ADR-0120 (tool surface), `dev-docs/health/freeze-readiness.md`

## Context

- Of the 14 premises of the freeze declaration (`4D-01`, gate A), 13 were closed on 2026-11-12. The remaining one is three cells of the evidence stage `2H-01`: the qwen repetitions of `T0-08` (H-ORACLE), the qwen of `T1-41` (H-EDIT), and `2A-14e` (H-AMORT, N=1,000). All three run on the measurement machine.
- The results of the measurement machine have not been committed since 2026-11-10 (the last commit of `bench/*/results`). The daily spending limit of the gateway sets the amount run per day (ADR-0088). Meanwhile the critical path (freeze → porting → gate B0) did not move, and all the work in this environment was advance investment (the fourteenth and fifteenth plans).
- The queue of the measurement machine grew in the meantime: `T0-33`, `T0-41`, `2D-39`, the six editions of `T0-37`, `bench/review`, `bench/intent`.
- ADR-0103 put the evidence stage before the freeze for one reason: if a measurement demands a language change, after the freeze two compilers have to be fixed. Decision 1 of ADR-0100 defined the object of the freeze as "the accepted language", and the tool surface is not an object (the tool surface is decided before the tool port P9, ADR-0120).
- Looking at the actions registered for the remaining three cells, none changes the accepted language.
  - `T0-08`: the verdict is support, refutation or failure to discriminate for H-ORACLE (`bench/brb-hard/README.md`). sonnet already failed to discriminate, and the qwen repetitions fulfil registered rule 2. No language change is registered for any result. The verdict of H-ORACLE is received by gate B (ADR-0111, the mutation score edition `T0-40` and H-TASK).
  - `T1-41`: the action after the verdict is "removal of the edit surface not chosen" (ADR-0120). The edit surface is tool surface. The H-EDIT condition of decision 4 of ADR-0105 was already read as "does not hold" from the sonnet result (the stage 3 section of ROADMAP).
  - `2A-14e`: a CPU measurement without a model, and what it measures is the cumulative cost of incremental re-proving. It is already in the premise list of gate B (ADR-0111).
- The four finished cells (`2R-05`, `T0-21`, `T0-18`, `3A-06`) did not demand a language change.

## Decision

1. **The evidence premise of gate A is "measurements whose registered action includes a change to the accepted language".** Of the seven cells of `2H-01`, only `2R-05` (the syntax A/B) falls under this, and it is finished. H-ENCAP·H-GAME·BNB could have led to a language change by their results, so placing them before the freeze was right, and they are finished too.
2. **The remaining three cells move to the premises of gate B.** The `T0-08` qwen repetitions, the `T1-41` qwen and `2A-14e` must be committed before P5 starts (gate B). `2A-14e` is already on that list. The verdict rules, the number of repetitions and the reference models are not changed.
3. **`2H-01` is not closed.** The verdict of stage 2 (the 2H pass criteria) needs the three cells to finish. Only what `4D-01` waits for changes: from the whole of `2H-01` to "the language-affecting cells of `2H-01`".
4. **Safety valve.** If, contrary to expectation, the results of the three moved cells demand a change to the accepted language, both sides are fixed together by the exception procedure of decision 4 of ADR-0100 (in `exceptions` of `surface.json`, with the reason). The port at that point is at most P0~P3.

## Rationale

| Option | Verdict |
| --- | --- |
| **Narrow the premise to language impact (accepted)** | It fixes at the layer where the cause is (process): the freeze, a language decision, is waiting on measurements that cannot change the language. The rationale of ADR-0103 ("the cost of a language change doubles") is kept as is. Not one verdict criterion changes, and all the measurements run as they are |
| Keep waiting | It keeps the registered order, but the wait buys nothing. The largest unmeasured risk is the porting cost (the estimate of porting map §7, 210,000~340,000 lines, with 0 lines of compiler written in BMB), and its first real measurement (gate B0) is tied to the spending limit of the measurement machine |
| A trial port before the freeze (one component as an experiment) | The option the fifteenth plan was to ask about separately. It brings forward the estimate of gate B0 but bypasses the stage order: experiment code cannot be counted as the port, so it has to be rewritten or kept outside the pinning mechanism (ADR-0113). Fixing the premise makes it unnecessary |
| Run the remaining cells in this environment | The model runs are registered under the conditions of the measurement machine (the same commit, the same gateway wallet), and `2A-14e` needs a quiet machine (the first measurement was contaminated by a concurrent build and discarded). Changing the conditions makes them incomparable with the earlier runs |

A concern dismissed: "freezing without evidence blurs the verdict of stage 2." The freeze is not the verdict of stage 2. The pass criteria of 2H and the E3 verdict of gate B remain as they are, and porting P0~P3 is work that builds the measurement object of gate B (the twins) regardless of the results.

## Consequences

- Premise 9 of `freeze-readiness.md` became "closed (language-affecting cells)", so all 14 premises are closed. Rows A and B of the ROADMAP gate table and the dependencies of backlog `4D-01` were fixed.
- The freeze declaration (`4D-01`, acceptance of ADR-0100) is not part of this decision. It is approved separately.
- The order of the measurement machine is that of the ROADMAP "priorities" (the three cells first). `2H-01` closes and gate B is passed only when the three cells finish.
