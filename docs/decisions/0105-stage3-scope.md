# ADR-0105: Stage 3 scope: the verdict is on the port twin, tool phases open on measured need

- Status: Accepted (approved by the maintainer 2026-11-11 "recommendations A1~A7 accepted", item A2. Backlog 2H-02). The grounds at approval: all 2H-01 results of the reference model (sonnet) are in, and the two conditions of decision 4 do not hold (BNB at 10,000 definitions: token ratio 0.087 and equal accuracy; in H-EDIT the write edition does not trail by more than D). The remaining qwen runs (T0-08 repeats, T1-41) are "other results" of decision 4
- Date: 2026-11-09
- Related: ADR-0104 (port twin), ADR-0103 (evidence stage), ADR-0102 (G5-L), ADR-0037 (bootstrap), ADR-0059 (precondition of preserving refactorings), ADR-0100 (stage0 freeze), ROADMAP stage 3, backlog 3A·3P

## Context

- The stage 3 phases (3A~3E) were set before ADR-0104. 3A (large corpus) was filled by the synthetic generator BNB, and 3A-01~05 are done. What remains is 3A-06 (size axis, the model arm is 2H-01), 3A-07 (H-REUSE judge) and 3A-08 (modification tasks).
- With ADR-0104 the main measurement ground became the port twin. The twin exists only once the port passes the differential tests up to P3 (checker). The port comes after the freeze (4D-01). So the time of the stage 3 verdict is tied to the progress of 4D.
- 3B (extended map), 3C (preserving refactoring operations) and 3D (payback) are tool features. The current plan builds them before measurement. The operating rule is "a differentiating hypothesis is measured with a minimal experiment before building on it".
- The tools (map, queries, workspace) are not subject to the freeze (ADR-0100 decision 1). But changing them after the ports of P5 (map), P9 (tools) and P10 (workspace) means fixing both sides, so tool changes are also cheaper before those ports.

## Decision

1. **The stage 3 pass criteria (3E) are judged on the port twin.** The verdict is H-COMPREHEND·H-REFACTOR·H-TASK, the H-SCALE slope and G5-L (ADR-0102) of 3P-05 (size P0~P3) and 3P-06 (P0~P7, the whole). The BNB criterion (at 10,000 definitions, at most 1/3 of the file-based tokens, accuracy equal or better) stays as an auxiliary verdict. G7 is judged by BCB (2A-14e).
2. **3A keeps only three and closes.**
   - 3A-06 ends with the BNB model runs of 2H-01.
   - 3A-07 (H-REUSE judge) is used as it is for the scope judge of 3P-03·3P-04.
   - 3A-08 (re-proof scope of modification tasks) moves to 3P-03·3P-04. The comparison with the Verus twin stays in BVB.
   - The synthetic codebase is not extended.
3. **3B·3C·3D open only on measured need.**
   - **3C (preserving refactoring operations):** 3P-03 runs first with definition-unit `write` only. The operations (rename, move, split, merge) are built and measured again on the same tasks in one of these cases: H-REFACTOR is refuted, or the majority of failures are attributed to out-of-scope changes and the call limit.
   - **3B (extended map, multi-perspective views):** when, in the failure attribution of 3P-02, the majority of failures are caused by a missing view (effect path, data flow, proof dependency), only that view opens.
   - **3D (payback, isomorphic merge):** decided after the measurement of duplication in real code in 3P-04 (the 3A-07 judge).
4. **Priorities by the 2H-01 results (registered before measurement).**
   - If, in the BNB model runs (3A-06), sonnet at 10,000 definitions has accuracy lower than the Rust arm by more than D, or the token ratio is above 1/3, the query surface fixes (the 2D-17 line) come before 3P-02. Those fixes finish before the P5 port.
   - If, in H-EDIT (T1-41), the write edition trails the original edition by more than D, ADR-0101 is reopened, and the write tool of the 3P-01 BMB arm is set by that result.
   - Other results (H-ENCAP, H-GAME, H-ORACLE holding or refuted) do not change the stage 3 order. They are reflected only in the hypothesis registry and the utility report.
5. **The order of stage 3 and stage 4.**
   - 3P-01~04 (harness, task generators, judges) are built before the freeze on fixtures (a small twin sample) and put on the gate with the oracle arm.
   - 3P-05 runs after the P3 port passes the differential tests.
   - Measurement does not set the order of the port (ADR-0104 consequences).

## Rationale

| Option | Verdict |
| --- | --- |
| **Verdict on the port twin, tool phases on measured need (accepted)** | The stage 3 claims are judged on real code. Tools are built only when the failure attribution requires them (operating rule "measure first") |
| The existing 3A~3E as they are | Synthetic code is the judging ground, and 3B·3C·3D are built first without evidence |
| Implement 3C before 3P | The precondition of ADR-0059 is met, but if H-REFACTOR holds with `write` alone, the operation surface only adds cost |
| Stage 3 verdict after the port is complete | The size axis (P3·P7·the whole) is lost, and tool fixes come after the port and touch both sides |

## Consequences

- If approved: the ROADMAP stage 3 table (3A~3E) is rewritten by this decision, 3A-08 of the backlog moves to 3P, and the opening conditions are written in the rows of 3B·3C·3D.
- The rules of decision 4 are registered by a commit made before the 2H-01 results came out (the first commit of this ADR).
- The precondition 2H-02 of 4D-01 (freeze declaration) is the approval of this ADR.
