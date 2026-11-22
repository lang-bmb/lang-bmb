# ADR-0116: The porting cost checkpoint gate B0 and the stage0 shrink survey before the freeze

- Status: Accepted (approved by the maintainer 2026-11-11 "recommendations A1~A7 accepted", item A5). It supplements the gate table of ADR-0111 and the freeze preconditions of ADR-0100 §5. Size correction (4D-11, `dev-docs/measurements/surface-cut.md` §3): `ast_json` is a tool (C10), so P2 is 5,883 lines (10%) and up to P3 is 25,912 lines (43%). The position of the gate stays as it is
- Date: 2026-11-11
- Related: ADR-0111 (decision gates), ADR-0100 (freeze), ADR-0113 (fixing of ported components), `dev-docs/measurements/selfhost-port-map.md` §7 (rebaseline, backlog 4D-07), backlog 4D-08·4D-11

## Context

- The port map was measured again at the current size (4D-07). stage0 is 60,463 lines (4.4 times 4A-02), and with the measured multipliers of the two components that exist on both sides (3.5~5.7), the whole port is estimated at 210,000 to 340,000 lines. The estimate of 4A-02 was 30,000 to 40,000 lines.
- Gate B (the E3 verdict) is after P3, at the 28,309-line (47%) point. Before that there is no point at which the cost is looked at again.
- The multipliers come from two pairs, and the multiplier of the path without proofs (E1) was not measured. 4A-02 also wrote "after C1 is moved, correct it with the real multiplier".
- The size of what is to be ported can be reduced cheaply only before the freeze.

## Decision

1. **The stage0 shrink survey (4D-11) is added to the preconditions of the freeze declaration (4D-01).** The survey gives, per candidate, the line count, the risk and the effect on the differential tests as a recommendation in the form of JUDGMENT §10. Accepted reductions are done before the freeze. The conclusion that there is nothing to reduce is also a result of the survey.
2. **Gate B0 (cost checkpoint) is set.** Its position is after P2 (lexer and parser, C1·C2: 8,280 lines, 14%) passes the differential tests and before P3 starts. What must be committed:
   - The C0~C2 rows of the port ledger (4D-08): BMB line count and definition count, the multiplier without contracts and the multiplier with proofs, the authoring cost (session tokens, rejected writes, undecided obligations), the share of proved definitions (S7), escape hatches, friction.
   - The whole estimate recomputed from those measurements (line count, obligation count, proof cost) and the update of port map §7.
3. **B0 is not a hypothesis verdict but a reporting gate.** Actions per result:
   - The whole estimate computed with the measured multipliers is within or below the range of port map §7 (210,000 to 340,000 lines): P3 as planned.
   - It exceeds the range, or the share of proved definitions of C2 is under half, or undecided obligations are the majority of the authoring cost: before P3 starts, report to the maintainer and raise a recommendation (repair of the completeness of the judge, adjustment of the porting scope, further shrinking of stage0). Progress comes after approval.
4. The position and the verdict of gate B (ADR-0111) stay as they are.

## Rationale

| Option | Verdict |
| --- | --- |
| **B0 + shrink survey before the freeze (accepted)** | Re-measuring the plan with the measurements of the first component is the standard practice of estimation (JUDGMENT S1·S5). It closes the gap of having no checkpoint up to the 47% point after the estimate changed by 6 to 8 times |
| Move gate B forward | The judging ground is a twin of P3 size (ADR-0104). Comprehension, refactoring and task tasks cannot be made from the lexer and parser alone |
| As now | It goes nearly halfway with only two pairs as the basis of the estimate |

## Consequences

- A B0 row in ROADMAP "critical path and decision gates", 4D-11 in the dependencies of backlog 4D-01, a new task 4D-12 (B0 report).
- The start of P3 waits for the B0 report. If within the range, it only reports and does not stop.
