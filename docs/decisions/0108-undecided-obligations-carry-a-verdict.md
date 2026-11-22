# ADR-0108: Undecided obligations carry verdict information: bounded counterexample search, `uses` search

- Status: Accepted (autonomous, AUTONOMY §4 "decision records": a tool-layer decision that adds information to diagnostics. The gate verdict and the language do not change. Ninth execution plan, approved by the maintainer "recommendation accepted")
- Date: 2026-11-10
- Related: ADR-0103 decision 5 (stop investing in prose repair suggestions), ADR-0078 (H-REPAIR), ADR-0067·0069 (probes), ADR-0075 (ground instances), DESIGN §6 (undecided is failure)·§8.2 (determinism), backlog 2D-32·2D-33·2D-34·T0-37

## Context

- 88% of the tokens of sonnet `brb-write` (2026-11-04) are in the 9 sessions that received an undecided response at least once (AI user opinion 2026-11-10 §3, reproduced from the session records). The 15 sessions with no undecided response are 2.2 to 2.8 times Rust.
- Replaying the 50 undecided `write` calls of that run with the current compiler, 45 are still undecided (rlimit 28, incomplete 10, other 7). ADR-0095·0096 did not reduce this share.
- An undecided response has only the position and "could not decide". The agent cannot tell whether its candidate is wrong or the prover could not keep up, so it changes the whole structure and writes again (`t14-dedup-sorted`: 22 of 34 `write` calls undecided, 25 definitions piled up and deleted).
- H-REPAIR (counterexamples and prose repair suggestions reduce repair iterations) was refuted by one qwen run, and ADR-0103 decision 5 stopped new investment in prose repair suggestions. What this ADR adds is not prose but the result of a verdict.

## Decision

1. **Bounded counterexample search (2D-33).** An obligation that ended undecided is asked once more. Only bounds on the inputs are added to the same encoding.
   - Integer inputs are `-B..=B`, the length of arrays, `Vec` and text of undetermined length is `0..=L`, and their elements are `-B..=B` if they are integers. Fixed at `B = 8`, `L = 4`.
   - The limit is 1/10 of the `rlimit` of the obligation. It is the same fixed share as for probes (ADR-0067).
   - If `sat`, that model is also a model of the unbounded query. So it is a confirmed counterexample. The diagnostic is emitted with the obligation's own code (E0802, E0807 and so on) and the counterexample values. The message says "counterexample found within the bounds".
   - If `unsat`, E0811 is emitted as it is and the `bounded` field carries `{"b":8,"l":4}`. It is not a proof. It is "no counterexample within these bounds".
   - If undecided, E0811 as it is.
   - The gate verdict does not change. Undecided is failure, and a confirmed counterexample is also failure.
2. **`uses` search inside the response (2D-34).** For a definition for which the `fix` of E0811 recommends `#[uses(..)]`, the MCP `write`·`verify` responses run the search of `bmb uses` limited to that one definition. Re-verification goes up to 4 times. The result (`proved` and the list, or `not_found`) goes in as a row of the same response. The output of the CLI `bmb verify` does not change (because the cost is proportional to the number of definitions).
3. **On only when requested.** `bmb verify --bounded-search` and `bmb mcp --undecided-info` (turns on 1 and 2 together). The default is off, so the output of the frozen measurement sets and of the gate stays as it is. A run with it on gets a marker in the query header and uses a separate proof cache. T0-37 sets the default. This follows the precedent of the compact view (ADR-0068 §5) and the fixed context (ADR-0071 §4).
4. **Determinism.** The bounds, the limit and the enumeration order are fixed, and the solver seed is fixed (§8.2). The same program gets the same response.
5. **Hypothesis H-UNDECIDE (registered, before measurement).** Adding verdict information (1·2) to undecided responses reduces the tokens per success and the number of `write` calls of authoring sessions that received an undecided response at least once. The measurement is in T0-37.
   - Comparison: the edition of `brb-write`·`bvb-write` with only the server flag `--undecided-info` added, versus the original edition. The same commit, sonnet once (ADR-0103 decision 3), qwen with the sequential repeats of ADR-0103 decision 2.
   - Refutation: if the difference in tokens per success is within D (ADR-0077), it is refuted. If the success rate drops by more than D, it is also refuted.

## Rationale

| Option | Verdict |
| --- | --- |
| **The same encoding + input bounds (accepted)** | `sat` is a confirmed counterexample as it is (the bounds only narrow the query). No new encoding or verdict path is built |
| Concrete execution on small inputs (interpreter) | A separate interpreter is needed to execute the specification (quantifiers, `spec fn`), and its meaning can diverge from the prover encoding |
| Only raise the limit, without bounds | The causes of undecided (incomplete theories, nonlinearity) are not solved by the limit. Only the cost grows |
| Write more prose guidance | H-REPAIR refuted, ADR-0103 decision 5 |

## Consequences

- `verify`: the bounded query after undecided (`set_bounded_search`), the `bounded` field of E0811, the obligation-code diagnostic of a bounded counterexample. `uses_fill::fill_uses_limited`, `mcp`: the `{"uses_search":..}` row in the response (4 re-verifications per definition).
- Tests: `bounded_search_turns_undecided_into_a_verdict` in `tests/verify.rs`, `undecided_info_runs_the_uses_search_in_the_response` in `tests/mcp.rs`.
- The measurement edition and the runs are T0-37 (after the evidence stage 2H-01, measurement session).
- H-UNDECIDE in the hypothesis registry (ROADMAP).
