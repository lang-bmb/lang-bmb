# ADR-0130: Freezing the H-INTENT item set, and its limits

- Status: Accepted (autonomous, AUTONOMY §4 "decision records": freezing the task set of a hypothesis defined by ADR-0078. The verdict criteria are not changed)
- Date: 2026-11-12
- Updated: the example text and the figures in "Consequences" changed with the re-freeze of ADR-0134 (2026-11-14). The values below are those of the first freeze.
- Related: ADR-0078 (H-INTENT), ADR-0128 (the same harness as the H-REVIEW set), ADR-0021 (freezing), `bench/intent`, `bmb examples` (T1-44c), backlog T1-44e·T1-44f

## Context

- ADR-0078 measures H-INTENT as "on a set with planted specifications that deviate from the intent, (a) review of code+spec and (b) review of spec+generated examples and counterexamples", and froze the confirmation criterion as "the detection rate of (b) is at least that of (a) (equal if within D) and the tokens read by (b) are 1/2 or less of (a)".
- The examples and counterexamples are produced deterministically by `bmb examples`, which reads only the contract.

## Decision

1. **Set.** `bench/intent`: two items for each of the 45 frozen BVB tasks, 90 in all. The program with the reference contract (45 sound) and the program with the weak contract (45 defective), with the same intent sentence. Nothing is hand-written. It was added as data to the harness of `bench/review`, and the frozen files there are unchanged.
2. **Two editions.** (a) `code+spec` is the whole program (contract and body). The contract is not attached a second time (attaching it makes (a) larger, which favours the hypothesis). (b) `spec+examples` is the contract text and the output of `bmb examples` as is (JSONL, default 3 inputs). The examples are generated once and the committed text is the frozen artifact; the compiler commit that generated them is in `set.json`.
3. **Prompt and judgement.** The prompt asks "does the contract guarantee the intent" and states the meaning of `allowed`·`rejected`·`excluded`, and that a missing row or an `undecided` means the tool could not produce it. The shape of the answer and the judgement are the same as ADR-0128. The confirmation and refutation rules are those of ADR-0078 unchanged.
4. **The only evaluators are model reviewers** (maintainer decision 2026-11-11, `2H-05` cancelled). The verdict is limited to model reviewers.
5. **Gate.** `bench intent`: structure, proof and labels of the 90 programs, whether the committed examples equal the output of the current compiler, determinism of the editions, two scripted reviewers, the verdict rules. The example comparison takes 63 seconds and gave the same bytes three times. A compiler change that changes the output of `bmb examples` is caught by this test: before the official run, re-freeze; after it, a separate decision moves the comparison out of the gate.

## Consequences

All are values measured deterministically before any model run.

- **The token criterion (1/2) is not reached on this set.** The (b)/(a) token ratio is 1.571 (sound 1.402, defective 1.749) and no item is within 1/2. The example output alone averages 222 tokens, larger than the whole program (216 tokens). The contract text alone is 0.39 of (a). The verdict this set can give is "refuted" or "not confirmed". The criterion is not changed.
- **The label can be read from the surface look of the examples.** The rule "two `allowed` results for one input means a defect" gets 82 of the 90 right without reading the contract or the intent (detections 42/45, false alarms 5/45). This is also the signal the examples were meant to give (the specification does not determine the result), but it is wrong on tasks where the intent permits several results (5 sound items). The detection rate of (b) is read together with the value of this rule (93%).
- **Sound items get fewer examples.** Items with no `allowed` row: 10 sound, 2 defective (the solver finds no model for contracts with recursive predicates). Items were not removed to even this out.
- **There is one kind of defect.** All 45 are an insufficient specification (a weak contract). The "other meanings" of ADR-0078 (a contract that says the wrong thing, a `requires` that is too narrow) are absent.
- The token axis and other kinds of defect are handed to `T1-44f` (a set derived from programs of real size). Reducing the example output (fields that are noise to the reader such as the `rlimit` of the summary row, the number of inputs) is a decision about the tool surface, so it is looked at together then.
- The real model runs are done on the measurement machine.
