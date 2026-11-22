# ADR-0076: The session output limit counts only visible output, and a response cut off at the output limit is continued once in the same turn

- Status: Accepted (approved by the maintainer 2026-11-01 "recommendation adopted, approval items approved": recommendation (나) of backlog T0-16). Supplements ADR-0066 §3 (measurement conditions) and ADR-0070 (same conditions)
- Date: 2026-11-01
- Related: ADR-0021 §3 (session rules), ADR-0034 §2 (fairness), ADR-0066, ADR-0070, backlog T0-16, T0-27, T1-40

## Context

- In T0-10 (qwen3.8-27b) many sessions ended on output length (`dev-docs/measurements/t0-10.md` §3.2·§3.4, `bench/failures.py`). `length` is m1 1, m1b 1, BVB 6, Verus 1, and in some of them the first response spent all 32,768 output tokens on reasoning and ended without a tool call (0 commits).
- In the BMB arm of BRB (interim tally), 11 of the 18 non-submissions ended at the session output limit of 64,000. A few responses each spent 10,000 to 21,000 tokens on reasoning. First-response `length` is 7.
- The limit was set before reasoning models, and the output tokens the API counts include reasoning. So with a reasoning model the measurement measured the token budget, not the language or the tools. qwen's BRB 2/24 (sonnet 21/24) is mixed with this condition (utility §2.7, H-SMALL in the review `dev-docs/reviews/hypotheses-2026-11-01.md`).

## Decision

1. **Visible output.** The model adapters (`OpenAIAgent` in `bench/m1/model_api.py`, `AnthropicAgent` in `bench/m1/anthropic_agent.py`) count the visible output of every response. Visible output is the text and the name and arguments of each tool call; the byte count is divided by 4 and rounded up (`model_api.visible_tokens`). `model_usage.visible_output_tokens` in the result row is the sum. If the API reports reasoning tokens separately (`completion_tokens_details.reasoning_tokens`), they are recorded in `model_usage.reasoning_tokens` but not used for the verdict or the limit. `output_tokens` (the whole output as counted by the API) is recorded as before.
2. **Session output limit.** The session limit of 64,000 (`SESSION_OUTPUT_TOKENS`) of BRB (`bench/brb/harness.py`) and BRB-encap is checked against the sum of visible output (`brb.session_output`). The value is unchanged.
3. **A response cut off at the output limit.** If a response ends at the output limit (OpenAI `finish_reason: "length"`, Anthropic `stop_reason: "max_tokens"`), the adapter makes one more request in the same turn. The text of the cut-off response stays in the conversation, and the tool calls in it are discarded (the arguments may be truncated, and Anthropic does not accept a `tool_use` without a result). After that, a fixed harness note (`model_api.CONTINUE_NOTE`) is appended as a user turn. If there is no visible text, the note is appended to the last user turn (roles must alternate). If the continued response also ends at the output limit, the session ends for that reason (`length`·`max_tokens`) as it does now. It is once per turn, and when a response with a tool call arrives, the next turn can again be continued once.
4. **Same rule.** The same rule holds for both providers and all sets (the m1 family, BVB and the Verus arm, BRB and its variants, BRB-encap). The per-response output limit (`MAX_TOKENS`: qwen 32,768, Anthropic 16,000) and the 40 tool calls are unchanged.
5. **Measurement condition.** The adapters and the BRB harness change, so this is a new condition (ADR-0070 (c)). The BRB harness is in the BRB MANIFEST, so it was frozen again, and the MANIFESTs of `brb-write`·`brb-diff`·`brb-ast`·`brb-encap`, which contain it, were rewritten too (tasks, prompts and verdicts are unchanged; ADR-0070 "when instrument code is changed, the MANIFEST is rewritten"). Recorded results (T0-10, T0-11) are left as they are and are not reported mixed with this condition. In the first formal measurement under this condition (T0-13), the originals to compare against (m1, m1b, BVB·Verus, both arms of BRB) are run again once per model at the same commit. Later comparisons (T0-14·T0-15·T0-17 and so on) take these rows as the baseline.

## Rationale

- **Cause and the layer it belongs to:** the measurement condition (process). The limit counts reasoning tokens, so the same limit means different things for a reasoning model and a non-reasoning model (S1 fairness, ADR-0034 §2).
- **Standard practice:** evaluations of reasoning models keep the reasoning budget apart from the output budget, or count only visible output. Fixing the reasoning length through the API (T0-16 candidate (다)) differs per provider and was not confirmed on the qwen gateway, so it was dropped.
- **Why bytes/4:** it does not depend on whether the provider reports reasoning tokens separately, and it gives both models the same rule from only what the model actually emitted visibly. The harness already counts the amount read the same way (tokens per query in `harness.py`, ceil(bytes/4)).
- **Why continue only once:** it stops a cut-off first response from ending the whole session, but does not let endless reasoning extend the session without bound. The limit of 40 tool calls does not count the continuation request (it is not a tool call). The number of requests is kept in `model_usage.requests`.
- **Rejected:** (가) the current condition as is (T0-16 candidate (가)). The qwen results keep measuring the token budget. Removing the session limit is not done, because ADR-0034 §2 requires the same budget for both arms.

## Consequences

- Tests: `test_length_is_continued_once_per_turn`·`test_reasoning_tokens_are_recorded_apart` in `bench/m1/test_model_api.py`, `test_max_tokens_is_continued_once_per_turn`·`test_max_tokens_with_nothing_visible_joins_the_last_user_turn` in `bench/m1/test_anthropic_agent.py`, `test_output_token_cap`·`test_reasoning_does_not_count_toward_the_cap` in `bench/brb/test_brb.py`.
- Result rows: `model_usage` gains `visible_output_tokens` (always) and `reasoning_tokens` (when the API gives it), and the agent settings (`agent_settings`) gain `"length": "continue once per turn"`. `limits` in the BRB result header has `"session_output_counts": "visible (ADR-0076)"`.
- The commit counts of the BRB BMB arm (`commits_ok`·`commits_rejected`, T1-40) went into the same commit. This is to refreeze BRB only once.
- Reverting: delete the continuation branch of the adapters and make `session_output` return `output_tokens`, and it is the previous condition.
