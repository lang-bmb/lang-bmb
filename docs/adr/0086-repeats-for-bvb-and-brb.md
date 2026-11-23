# ADR-0086: Repeated measurement of BVB·BRB and a minimum difference per set

- Status: Accepted (approved by the maintainer 2026-11-04 "recommendation approved": start the ADR on measurement discriminating power, the T0-13 surface decision comes after this result). Widens ADR-0077 §2 (repeat scope) and supplements §4 (minimum difference)
- Date: 2026-11-04
- Related: ADR-0066 §3·§5, ADR-0077, backlog T0-13·T0-14·T0-33·2A-16

## Context

- The minimum difference D = 30%p of ADR-0077 came from the repeat range of qwen m1 writing (6/10~9/10) and is used for single-run comparisons of every set without repeats.
- Most of the comparisons up to 2026-11-04 were within D, so no direction could be claimed. That is five of the six cells of T0-13 and both of T0-14. m1 has 10 tasks, so one task is 10%p. The variation of sets with 45 tasks (BVB) and 24 tasks (BRB) has never been measured.
- sonnet reaches the ceiling on BVB·BRB (45/45, 24/24). So it is mainly qwen that separates differences between surfaces and variants.

## Decision

1. **Repeat scope.** With `openai:qwen3.8-27b`, BVB and BRB (BMB arm) are repeated **5 times** each for the original and the write surface (`bvb`·`brb-bmb`·`bvb-write`·`brb-write` in `run_models.REPEAT_SETS`). The first repeat is done at the same commit as T0-13, `e5247e8d`. The write surface (`bvb-write`·`brb-write`) goes first, because T1-41·T0-17·T0-15 compare against that edition. The run and report rules are the same as ADR-0077 §1 (nothing selected or discarded, everything committed). The BRB Rust arm is not repeated (24/24 for both sonnet and qwen).
2. **Minimum difference per set.** D(X) of a set X with repeats is the repeat range of the writing success rate of X, rounded up in percentage points (the "per-set minimum difference" line of `repeat.py`). A single-run comparison of two editions X·Y (for example the original and the write surface, the original and a variant edition) is read with max(D(X), D(Y)). If either edition has no repeats, the global D of ADR-0077 §4 is used. The number of tasks and the variation differ per set, so the range of one set is not carried over to another.
3. **Re-reading existing verdicts.** When the repeat results exist, the differences of T0-13 (write surface, surface decision) and T0-14 (H-NAME) are read again with this rule and appended to the reports. The original verdict lines are not deleted.
4. **Mission targets (2A-16).** 2A-16 is set after this result, so that the BVB writing target can be judged together with the interval of these repeats.

## Rationale

- Using the D set from the range of m1 unchanged for BVB, which has 4.5 times as many tasks, is wrong in the direction of losing discriminating power. One task's share is 2.2%p, so the range of the variation will differ too. It is not known without measuring.
- qwen has no repeat cost (ADR-0066 §2, local equipment behind the gateway). The time is about 8 minutes per qwen session. Five runs of one BVB edition take about 8 hours at parallelism 4, and BRB about 16 hours at parallelism 1 because of the wall-clock limit of the BRB verdict.
- **Rejected:** (a) the global D as is. It has no discriminating power, so running more measurement lines only piles up "cannot be judged". (b) repeating sonnet too. It is at the ceiling, so the variation cannot be measured, and it is paid (the same reason as ADR-0077 (b)). (c) computing D with a per-task binomial model. Trials of the same task are not independent (ADR-0077 rationale), so the run-level range is measured.

## Consequences

- Code: `REPEAT_SETS` in `bench/run_models.py`, `bench/repeat.py` (BRB rows are counted as writing and the arm is appended to the set name; the per-set minimum difference line).
- Tests: `test_repeats_are_declared_for_qwen_on_m1_and_m1b` in `bench/test_run_models.py`, `test_brb_rows_are_writes_of_their_arm` in `bench/test_repeat.py`.
- The first run calls the harness directly in the measurement checkout of `e5247e8d` (the `run_models.py` of that commit allows repeats only for m1·m1b). `repeat` in the result header is recorded by the harness's `condition_fields` from `BMB_REPEAT`. The BRB harness of that commit uses the same function.
- Reverting: restore `REPEAT_SETS` to the two of ADR-0077 and delete the per-set line.
