# ADR-0102: The unit of judgement for utility (lifecycle unit G5-L) and the scope of principle 4

- Status: Accepted (approved by the maintainer 2026-11-09 "Ok, proposals and recommendations accepted", proposals P4·P5 of the overall evaluation)
- Date: 2026-11-09
- Related: DESIGN.md §2 (premise E3, ledger)·§3 principle 4, ADR-0034 (G5), ADR-0041 (ledger, consequence "hard to write versus AI efficiency"), ADR-0003 (area that needs tests), ADR-0078 (H-EFFECT), `dev-docs/measurements/t0-30.md`, `dev-docs/reports/utility.md` §2.7

## Context

- **G5 (ADR-0034) is judged by the total tokens of one BRB task.** The stage 2 criterion is "defect rate at most 1/2 of Rust+tests, total tokens within 2x". In the formal measurements, tokens per success are about 15 to 18 times Rust for sonnet (original edition and write edition), and 1.3 to 2.4 times for qwen on the write edition. The Rust baseline of a strong model is about 3 tool calls per session.
- **Premise E3 is a trade over the lifecycle.** "A cost paid once at writing time reduces the verification cost repeated at every change and the defects that escape." One task measures only the front half of that trade (writing). The gain side (later changes, re-verification, defect repair) is in no formal measurement.
- **H-EFFECT is refuted (T0-30, sonnet).** In the four tasks that split off I/O by effect, the remaining test area was 0.667 of the executable definitions (criterion ≤ 0.2), and the code that needs tests was 4.1 times the Rust test lines. The area comes from the decomposition: the `io` shell definition is itself area. Principle 4 already says "the scope of replacement is functional correctness at the unit level", but it does not say that the effect shell remains.

## Decision

1. **G5 is reported in two layers.**
   - **G5 (task unit)**: as in ADR-0034 (BRB, within 2x). For now it is recorded as **not met**. The consequence of ADR-0041 (if total tokens are large, fix the surface and the tools) stays valid, and the utility report reports the trajectory of the ratio (per measurement).
   - **G5-L (lifecycle unit)**: in the same codebase, the total tokens and the cumulative verification run cost of 1 authoring + k follow-up changes + the repair of defects found by the hidden judgement. The comparison group is a Rust+tests arm that follows the same history. The target puts the stage 3 criterion of ADR-0034, "total tokens at most Rust+tests", on this unit.
2. **E3 is judged by G5-L and the defect rate.** E3 holds when G5-L ≤ 1x, or when G5-L is within 2x and the escaped defects are at most 1/2 of Rust+tests. If neither, it is refuted. This criterion is registered before the measurement (the measurement grounds are ADR-0103·0104).
3. **The scope of principle 4 is written down.** What proof replaces is the functional correctness of the pure core (`pure` definitions). `io` effect definitions (the shell) remain as test area, and the compiler computes and reports the area (ADR-0003). The ways to shrink the shell (tests generated from the specification, record and replay, a decomposition diagnostic that makes the shell thin) stay candidates only, and are measured before adoption.
4. The verdict on H-EFFECT stays "refuted (sonnet)". The same criterion is measured again on the effect distribution of real code (the port twin, ADR-0104).

## Rationale

| Option | Verdict |
| --- | --- |
| **Two layers, the decisive verdict is the lifecycle unit (accepted)** | The unit of the trade that E3 claims matches the unit of measurement. The task-unit failure is not erased but recorded as it is, so the target is not lowered after the fact |
| Keep only the task unit | With a strong model it has a shape that cannot be met in principle (three calls on the Rust side), and it does not measure the gain side of E3 |
| Lower the task-unit target (for example 5x) | A relaxation after the fact with no grounds |
| Principle 4 as it is | The refuted expectation (aiming at "0 tests") reads as if it also applies to the shell |

## Consequences

- The sentence on the unit of judgement under the ledger in DESIGN.md §2 and the sentence on the scope of principle 4 in §3.
- The H-EFFECT row of the hypothesis registry (ROADMAP), utility report §2.1·§2.7.
- The measurement grounds of G5-L: the BCB history (2A-14e, cumulative cost) and the task history of the port twin (ADR-0104).
