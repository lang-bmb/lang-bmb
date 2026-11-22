# ADR-0103: The evidence stage before the freeze (2H-01) and the power of the measurements

- Status: Accepted (approved by the maintainer 2026-11-09 "Ok, proposals and recommendations accepted", decisions D2·D5 and proposals P1·P2·P7·P8 of the overall evaluation) · the repeat count and the verdict of decision 2 are supplemented by ADR-0112 (4 runs per arm, the interval of the difference)
- Date: 2026-11-09
- Related: ADR-0100 §5 (preconditions of the freeze declaration), ADR-0066 (measurement models), ADR-0077 (repeated measurement, minimum difference D), ADR-0078 (auxiliary hypotheses), ADR-0102 (G5-L, E3 verdict), ADR-0104 (large-scale task axis), `dev-docs/reports/utility.md` §5

## Context

- Of 26 hypotheses, 0 are confirmed, 2 are refuted (H-EFFECT, H-REPAIR), and 7 have deterministic evidence. The deterministic evidence is all about properties of the tool chain (stability, locality, identity, merge). The hypotheses on the gain side of E3 (H-ORACLE, H-AMORT, H-SCALE, H-ENCAP, H-GAME) cannot be judged or have not run yet. Most of their measurement tools are frozen: `bench/brb-encap`, `bench/impossible`, `bench/{m1b,bvb}-{diff,ast}`, the BCB `suite`, the 27 BNB tasks.
- After the freeze (ADR-0100), a language change that a measurement requires must also go into the ported side, so the cost doubles.
- The minimum difference D of a single run is 30%p (ADR-0077). On a set of 25 tasks, a difference within 30%p cannot claim a direction. So there is almost no path from "direction agrees" to "confirmed".
- H-ORACLE did not discriminate, because the hidden defects were 0 in both arms. The tasks are too easy (T0-07).
- The same author made the tasks, the harness and the compiler (utility §4.3).
- H-REPAIR (counterexamples and repair suggestions reduce repair iterations) was refuted by one qwen run, yet the P3 ratchet of the gate (the share of errors that carry a way to fix them) kept pulling work toward adding more prose repair suggestions.

## Decision

1. **2H-01 is the evidence stage.** It runs on the measurement machine, in order, before the freeze declaration (4D-01). The order is by the size of the pending verdict.
   1. 2R-05: the grammar A/B before and after ADR-0095 (`bench/m1b`, as registered). Then the A/B of the write edition (`bench/m1b-write`).
   2. T0-21 H-ENCAP, T0-18 H-GAME: frozen sets, with the refutation conditions registered.
   3. T0-08 H-ORACLE discriminating tasks: newly made, frozen, and then run (decision 4).
   4. T1-41 H-EDIT: the remaining cells.
   5. The BNB model runs (the model arm of 3A-06, ADR-0104 decision 5): the first model evidence on the scale axis.
   6. 2A-14e H-AMORT: a CPU measurement that needs no model. If INF-19 shows that there is no contamination from overlap, it runs in parallel with the model measurements.
2. **Power comes from a sequential design.** Each comparison above runs once with qwen, and if the difference is within D, the repeats are raised to 3 at the same commit and conditions and the verdict uses the interval of ADR-0077 §3. The scope of repetition (ADR-0077 §2) is widened to these comparisons. Whether to raise the count is decided by the difference of the first run only, not by the direction of the result (this sentence is the pre-registration). sonnet runs once at the same commit, as in ADR-0066 §3.
3. **The reference model for claims.** Utility claims about the language and the tools are judged by the results of the frontier cross-check model (sonnet). The qwen results are reported side by side in the same table, and the gap is tracked as H-SMALL. The default model for running measurements stays qwen as in ADR-0066 (repeats, debugging, first runs). Whether small models become a first-class target is decided again after the H-SMALL measurement.
4. **External tasks are mixed in.** The tasks of T0-08 are taken from third-party sources where possible (verified public benchmarks, public bug data), and the source and licence are written per task. If a source cannot be fetched (network policy), the task is marked as an author task.
5. **New investment in prose repair suggestions stops.** The P3 ratchet stays as regression prevention. Tasks whose only purpose is to raise P3 are not created until H-REPAIR is judged again by repeated measurement (decision 2). The machine-applied `bmb fix` is outside this decision (utility of normal form and determinism).

## Rationale

| Option | Verdict |
| --- | --- |
| **Evidence stage before the freeze, sequential repeats (accepted)** | Before the freeze is the last point at which a language change is cheapest. The measurement tools are already frozen, so little more has to be built. Repeats happen only when needed, which saves measurement machine time |
| Measure after the freeze | If a result requires a language change, two compilers must be fixed |
| 5 runs for every comparison from the start | The power is highest, but the serial time on the measurement machine is five times (about 5 minutes per qwen session) |
| qwen as the reference for claims | The maintainer's instruction is "qwen where possible", and that is an instruction about running. Judging utility claims on the target user (frontier agents) matches the target of E1 to E3 |

## Consequences

- The completion criterion of backlog 2H-01 becomes the list above. 4D-01 comes after 2H-01 and 2H-02 (as in ADR-0100 §5).
- `REPEAT_SETS` in `bench/run_models.py` is widened by the sets that come to need repeats (this ADR is then cited as the rationale).
- The utility report (utility) is rewritten, including the prose of §0 to §5, each time a result comes in.
