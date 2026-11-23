# ADR-0149: Gate B (economics) cannot be judged yet and is carried over to gate D

- Status: Accepted (approved by the maintainer on 2026-11-23)
- Date: 2026-11-23
- Related: DESIGN.md §2 (premise E3), ADR-0102 (the E3 criterion), ADR-0103 (reference model), ADR-0111 (decision gates; this record applies its decisions 2 and 3), ADR-0112 (judgement rule), ADR-0146 (text search and bounded responses), [`../CLAIMS.md`](../CLAIMS.md)

## Context

- Gate B asks whether premise E3 holds on a real codebase: the Rust compiler and its BMB port as twins. The criterion was registered before the measurement (ADR-0102 decision 2): E3 holds when the life-cycle token ratio G5-L is at most 1, or when it is within 2 and the escaped defects are at most half of those of Rust with tests. The verdict uses the reference model, sonnet (ADR-0103 decision 3). The action for each result was registered too (ADR-0111 decision 2).
- The twins were measured twice. The first condition gave the BMB side no text search. The second added text search and bounded responses (ADR-0146) and changed nothing else: same tasks, prompts, Rust side, model and judgement rule.
- Results of the reference model, one run each, BMB against Rust:

  | Measure | Without text search | With text search |
  | --- | --- | --- |
  | Histories (G5-L: five changes in sequence, 4 tasks): success | 4/4 against 3/4 | 4/4 against 3/4 |
  | Histories: API tokens, median of per-task ratios | 1.93 | 1.07 (no interval with 4 tasks; 1.01 by sums) |
  | Replayed commits (22 tasks): success | 21/22 against 21/22 | 21/22 against 22/22 |
  | Replayed commits: API tokens, median of per-task ratios (95% interval) | 3.66 (2.14 to 5.46) | 1.30 (0.71 to 2.14) |
  | Understanding (24 questions): correct | 22/24 against 23/24 | 24/24 against 22/24 |
  | Understanding: tokens read, median ratio (interval) | 0.83 (0.05 to 9.43) | 0.07 (0.05 to 0.09) |
  | Refactoring (20 tasks): success; API tokens | 20/20 against 20/20; 3.52 (1.90 to 5.14) | not measured again |

- Escaped defects do not discriminate: with text search each side failed one session of 26.
- The small model (qwen, three runs) finished only its Rust side under the second condition; its BMB sessions ran 41 to 102 minutes each and the runs were stopped at 6 of 78 rows. It is not the reference model and is not part of the verdict.
- The accumulated verification cost (the other half of the life-cycle unit) is still being measured.

## Decision

1. **The verdict at gate B is "cannot be judged".** G5-L under the current tools is 1.07 by the registered statistic, from four tasks and one run, with no interval. That is neither "at most 1" nor a refutation, and the defect clause has nothing to compare. This is the third registered outcome of ADR-0111 decision 2.
2. **The verdict is carried over to gate D,** where it is made once more at full size, as ADR-0111 decision 2 registers.
3. **This is the last carry-over for E3.** The move of the task-unit verdict to the life-cycle unit already counted as a first carry-over (ADR-0111 decision 3); the carry-over from gate B to gate D is the one that decision 2 registered for this gate. If E3 does not hold at gate D, it is recorded as "could not be shown" and DESIGN, README and the claims are revised. No further move.
4. **The port continues past gate B.** Stopping was the registered action for a refutation only.
5. **What the gate D measurement must bring,** so that the next verdict is not again "cannot be judged":
   - enough history tasks for an interval on G5-L, and at least three runs of the reference model;
   - tasks drawn anew at full size, not the ones the tools were tuned after;
   - refactoring measured under the current tools;
   - the small model's runs completed, after the length of its BMB sessions is explained;
   - the accumulated verification cost.
6. **No public statement claims the economics.** `CLAIMS.md` lists task replay as "measuring" with both conditions side by side, and the roadmap lists gate B as carried over.

## Rationale

| Option | Verdict |
| --- | --- |
| **Cannot be judged, carried over once more (chosen)** | It is what the pre-registered rule gives for these numbers. The cost of being wrong is bounded: the port to the full fixed point is an engineering goal that stands without E3 |
| Declare that E3 holds | 1.07 is above 1 by the registered statistic, there is one run and no interval, and the defect clause is empty. Reading 1.01 (sums) instead would be choosing the statistic after seeing the result |
| Declare E3 refuted on the first measurement | The first condition lacked a tool whose absence, not the language, carried the cost: adding it moved the BMB side's input tokens to 0.36 to 0.45 times with the Rust side unchanged. A refutation of that condition stays on record as such |
| Wait for the small model before a verdict | The verdict uses the reference model. The small model's result is reported beside it and tracked separately |

What weakens this reading, and is stated so it is not lost: the tools were fixed after seeing the first results and measured again on the same tasks. The second numbers are an upper bound on what new tasks will show; item 5 exists for that reason.

## Consequences

- `docs/ROADMAP.md`: gate B is "carried over to gate D".
- The measurement plan for gate D grows by the items of decision 5.
- If this record is rejected in favour of "refuted", the registered action applies instead: the port stops after the layer in progress and a direction record is raised.
