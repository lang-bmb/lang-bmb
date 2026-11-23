# ADR-0136: Freezing the ablation editions of H-UNDECIDE, H-PROBE and H-HISTORY

- Status: Accepted (autonomous, AUTONOMY §4 "decision records": freezing the task sets of registered hypotheses. The verdict criteria are not changed)
- Date: 2026-11-14
- Related: ADR-0108 (H-UNDECIDE), ADR-0109 (H-PROBE), ADR-0110 (H-HISTORY), ADR-0021 (freezing), ADR-0070 (measurement conditions), backlog T0-36·T0-37

## Context

- The three ADRs each registered their comparison before any run: an edition of `brb-write`·`bvb-write` with just that one thing added versus the original edition, at the same commit.
- The features are behind flags (`bmb mcp --undecided-info`, `--probe-tool`, `history` in `set.json`), and the harness wiring was finished in T0-36. What remained was freezing the editions.

## Decision

1. **Six editions.** `bench/ablate/gen.py` generates them: `bvb-write-undecided`·`-probe`·`-stateless`, `brb-write-undecided`·`-probe`·`-stateless`.
2. **They differ from the original edition in one key of `set.json` only.** One flag is added to `mcp_flags`, or `"history": "stateless"` is added. The prompts, seeds, tasks and hidden judgement are the bytes of the original edition unchanged.
3. **The prompt does not mention the probe tool.** What the model sees is the tool list and descriptions of the server (decision 4 of ADR-0109: an edition with just one flag added). Adding guidance to the prompt makes two variables.
4. **The Rust arm of BRB.** It has no edition (there are only variants of the BMB arm). The Rust arm of H-HISTORY is `harness.py --arm rust --history stateless`, and the other two are features of the BMB server, so the Rust arm uses that of the original edition.
5. **Gate.** `bench ablate`: the committed editions equal the output of the generator, they differ from the original edition in one key only, the tool list of the server per edition (only the probe editions have one more, `probe`), the scripted reference and weak agents pass and fail in every edition (2 BVB tasks, 2 BRB tasks), the `--history stateless` of the Rust arm.

## Consequences

- The measurement machine can run the editions as soon as it receives them. The order and number of runs are those of each ADR unchanged (sonnet 1 run, qwen sequential repetitions).
- The verdict is T0-37. If H-PROBE is refuted, a task to remove the batch probe is derived (ADR-0120).
- The scripted agents read neither the prompt nor the tool descriptions, so what the gate shows is only that the editions pass through the harness. Only model runs measure the effect of the flags.
