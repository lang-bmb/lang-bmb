# ADR-0110: Stateless repair turns: a measurement condition in which every request is a new conversation

- Status: Accepted (autonomous, AUTONOMY §4 "decision records": one new measurement condition of ADR-0070. The runs are T0-37 after the evidence stage. Ninth execution plan, approved by the maintainer "recommendation accepted")
- Date: 2026-11-10
- Related: ADR-0070 (measurement conditions), ADR-0068·0071 (editions that reduce what the model sees), ADR-0103 (power, the reference model for claims), ADR-0093 (cache), backlog T0-36·T0-37

## Context

- The current agents (`model_api.OpenAIAgent`, `anthropic_agent.AnthropicAgent`) resend the whole history on every request. So the input tokens of a session grow close to the square of the number of round trips.
  - Example: in `t14-dedup-sorted` of sonnet `brb-write`, the input per request grew from 2,749 → 56,096 tokens. The text newly produced in the session is 83KB, but the sum of input tokens is 883,748 (AI user opinion 2026-11-10 §3.3).
- In BMB the state is in the workspace (behind the tools). The history repeats that state.
- Upper-bound estimate (the formula of appendix A, assuming requests and calls 1:1, off for 4 of 24 sessions): with the same number of round trips and the same output, the total tokens of sonnet `brb-write` go from 2,188,065 → 749,725 (0.34x). The value does not include the risk that a model that lost its history repeats the same attempt.

## Decision

1. **The content of a request.** Every request is the system prompt and one user message. The user message is in this order.
   1. The task statement
   2. `## State now`: the current state
   3. `## Attempts so far`: one line per tool call of the session (`N. tool: ok|error: first line of the result`, up to 160 bytes)
   4. `## Results of your last calls`: all tool results of the previous turn
   - The responses and tool results of earlier turns are not sent again.
2. **The same rule for both arms.** `State now` is the workspace files (normal form) in the BMB arm, and the source files of the crate in the Rust arm. The tools, the call limit and the judgement stay as in the original edition.
3. **The prompt does not change.** No notice that there is no history is added. One message is complete in itself.
4. **Implementation and when it applies.**
   - The implementation is `Stateless` in `bench/m1/stateless.py`. It wraps the provider agent and keeps the shape of the result rows and the session records. `settings` gets `"history": "stateless"`.
   - The harness wiring (`"history": "stateless"` in the `set.json` of the set) and the frozen edition are made when T0-37 is prepared. Changing the harness now would make it diverge from the harness of the measurement session that is in the evidence stage.
   - This condition cannot be checked with the oracle (script) arm (a script has no history). The gate evidence is the fake-model tests of the two providers (`bench/m1/test_stateless.py`).
5. **Hypothesis H-HISTORY (registered before measurement).** Stateless turns reduce the tokens per success without dropping the success rate by more than D (ADR-0077).
   - Comparison: this edition of `brb-write`·`bvb-write` versus the original edition. The same commit, the Rust arm under the same rule. sonnet once (ADR-0103 decision 3), qwen with sequential repeats.
   - Refutation: if the success rate drops by more than D, or the difference in tokens per success is within D, it is refuted.

## Rationale

| Option | Verdict |
| --- | --- |
| **A new conversation per request + state + one line per attempt (accepted)** | It uses the design in which the source of truth is behind the tools as it is. The same rule can be applied to both arms |
| Cut the history to the last N turns | N becomes a new free variable, and the state is still scattered in the history |
| Only reduce the cost with a cache (ADR-0093) | The price drops, but the amount the model reads and the token metric (the whole prompt) stay the same |
| Wire it into the harness now | The measurement session is using the same harness in the evidence stage. Deferred to the preparation of T0-37 |

## Consequences

- `bench/m1/stateless.py`, `bench/m1/test_stateless.py` (gate job "bench m1 model agents").
- H-HISTORY in the hypothesis registry. T0-36 stays 🔄 because the harness wiring and the frozen edition remain, and both are done in the preparation of T0-37.
