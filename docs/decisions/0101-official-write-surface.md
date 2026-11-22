# ADR-0101: The write surface of formal measurement is the single write per definition (`write`)

- Status: Accepted (approved by the maintainer 2026-11-09 "Ok, proposal and recommendation adopted", decision D1 of the overall evaluation)
- Date: 2026-11-09
- Related: ADR-0066 (measurement model environment), ADR-0077 §4 (minimum difference D), ADR-0073 (freeze of the three edit surfaces), ADR-0009 (writes are transactions), `dev-docs/measurements/t0-13.md` (recommendation), backlog T0-13

## Context

- Formal measurement (ADR-0066) ran on the transaction surface (`tx_begin`·`tx_unit`·`tx_put`·`tx_commit`). A large share of the failures of the originals was the tool call limit: qwen BVB 20/45·BRB 9/24, sonnet BRB 4/24. Writing one definition takes at least five calls.
- In T0-13 (`e5247e8d`, two models at the same commit), the single write per definition (`bench/*-write`) has no cell that trails the original by more than D (30%p), and on qwen BRB it leads by +46%p, more than D (7/24 → 18/24). Hitting the limit is qwen 1, sonnet 0. Tool calls are −80% on BVB (sonnet).
- sonnet `m1b-write` was run again at the same commit on 2026-11-05 and was 25/25 (replacing the first run voided by ADR-0088). The write edition is equal to or higher than the original on all three sets (25/25·45/45·24/24).
- On 2026-11-04 this decision was postponed until after T0-33 (D per set from BVB·BRB repeats). Of T0-33 only `bvb-write` of repeat 1 is valid, and the rest remains.
- The rule proposed in advance ("`write` if both BVB and BRB exceed D") is not met. sonnet is near the ceiling on both sets (45/45, 20~24/24), so that rule could hardly be met, and this was not considered at registration.

## Decision

1. **The default surface of formal measurement is the single write per definition.** The default sets of `bench/run_models.py` are `m1b-write`·`bvb-write`·`bvb-verus`·`brb-write`·`brb-rust`. The comparison groups (the Verus and Rust arms) do not change.
2. **The originals (`m1`·`m1b`·`bvb`·`brb-bmb`) remain as frozen comparison baselines.** They run when named. Comparisons with the T0-10·T0-11·T0-13 original results, and the variant editions made from the originals (T0-14 H-NAME, 2A-07, T0-15, T0-17, T0-18), compare with their own originals.
3. **New variant editions are made from the write edition.** Measurements already in progress finish on the set they registered (2R-05 is registered with `bench/m1b` and runs as is. The write edition A/B is added in the evidence stage of ADR-0103).
4. **It is stated that this is a post hoc change.** This decision is a rule change after seeing the T0-13 results. The grounds are "no loss beyond D" and "the cause of failure is the call cost of the tool layer", and the language (verification, verdict) is the same on both surfaces.
5. The write semantics of the language (transactions, proof is the build gate, ADR-0009) do not change. `write` is a tool surface that opens and closes the transaction of one definition in one call.

## Rationale

| Option | Verdict |
| --- | --- |
| **`write` as the formal one (adopted)** | No loss beyond D, a gain beyond D in one cell, mostly fewer calls and tokens. The standard shape of an agent edit tool (a write that finishes in one go, several steps as an option) |
| Keep the originals | The measurement measures the call limit of the tool procedure rather than the language |
| Apply the advance rule as is | Because of the ceiling effect it in effect pins the originals. A registration defect of the rule |

## Consequences

- `DEFAULT` in `bench/run_models.py` and `bench/test_run_models.py`.
- The postponement of 11-04 is lifted by this approval. T0-33 keeps running and re-reads the differences of T0-13 with D per set. If the re-read result shows the write edition trailing by more than D, this decision is reopened.
- The cost description of the utility report (`dev-docs/reports/utility.md`) takes the write edition results as the default.
