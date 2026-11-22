# ADR-0066: Measurement model environment: the default is `qwen3.8-27b`, the frontier cross-comparison is `claude-sonnet-5-5`

- Status: Accepted, "same commit" of §3 is superseded by ADR-0070 (approved by the maintainer 2026-10-30: "use of the two models in this environment approved; use the qwen model where possible, and use sonnet when cross-comparing with a frontier model". Guidance on providing the benchmark environment)
- Date: 2026-10-30
- Related: ADR-0021 (measurement procedure), ADR-0034 (mission goals), ADR-0058 §2 (measurement path: pilot first, the formal path awaits approval), AUTONOMY.md §4 (paid APIs need approval), backlog T0-03

## Context

- The metrics that need a model to measure (G2·G3·G5·G8, H-SPEC·H-ORACLE·H-FAMILIAR etc.) had no formal measurement path and were measured only by the pilot (Claude Code CLI, ADR-0058 §2). Pilot results are not used for claims. The formal path (T0-03: API runs in which the harness holds the agent loop) had been blocked since 10-24 because the paid API and the secret setup are the maintainer's part.
- This execution environment provides the model gateway `https://token.iyulab.com`. The environment injects the credentials, so there is no key in the repository or in commands. Checked on 2026-10-30:
  - `GET /openai/v1/models`: six models. The generation models are `qwen3.8-27b` (the gateway's local machine) and `claude-sonnet-5-5`·`gpt-5.4-mini`·`gemini-3.8-flash` (external providers).
  - `GET /anthropic/v1/models`: `claude-sonnet-5-5` only.
  - Text generation and tool calls: `qwen3.8-27b` on `/openai/v1/chat/completions` (OpenAI-shaped `tool_calls`), `claude-sonnet-5-5` on `/anthropic/v1/messages` (Messages API-shaped `tool_use`, accepts `output_config.effort`). Both succeeded.
  - End to end through the harness: both models succeeded on `w01-clamp` (writing) and `n01-callers` (navigation) of m1 (verification runs, results not committed).

## Decision

1. **Measurement models use the gateway of this environment.** The harness has two agent kinds: `openai:MODEL` (Chat Completions), `anthropic:MODEL` (Messages API). The default address is the gateway and it can be changed with environment variables (`BMB_OPENAI_BASE_URL`·`BMB_ANTHROPIC_BASE_URL`). Keys are not kept in the repository (`bench/m1/model_api.py`).
2. **The default model is `openai:qwen3.8-27b`.** Measurements that need a model are done with this model first: formal measurement of the frozen task sets, the first run of a new task set, re-measurement after language or tool changes, failure cause analysis.
3. **`anthropic:claude-sonnet-5-5` is used only for the frontier cross-comparison.** The same frozen set is run once after the qwen run, on the same harness and the same commit, and the results of the two models are reported side by side. It is not used for iteration during development or for debugging.
4. **This path is the formal measurement of ADR-0021 (T0-03 unblocked).** The harness holds the agent loop, the tool limit and the verdict, and every row records the model and settings (`agent_settings`) and tokens (`model_usage`). Results can be used as the basis of claims with the model name attached. They are not generalized to other models. The CLI pilot (ADR-0058 §2) remains a pilot.
5. **Run rules (ADR-0021 as is):** one session per task, no retries, the frozen set and prompt as they are. One run per (set, model, commit), and every result that was run is committed. Measuring again at a later commit is a new measurement, and it is reported together with the earlier results without choosing. Only transport errors (connection, HTTP 429·5xx) are re-sent with the same request, and the count is recorded as `http_retries`. This is not a task retry.
6. **Scope of the usage approval:** runs that use these two models for the purposes above do not need approval per run (an exception to the paid API row of AUTONOMY.md §4). Other models (`gpt-5.4-mini`, `gemini-3.8-flash` etc.), other endpoints and other purposes still need approval.

## Rationale

- **Cause and layer:** the cause of the empty metrics was the lack of a measurement path (process). The maintainer fixed the path and the models, so the process block is lifted (the release condition of ADR-0058 §2).
- **qwen as the default:** it is the maintainer's guidance. It is also better as measurement evidence. It is an open-weight model on a local machine, so the same model can be measured again, and whether tools and contracts contribute to the success rate shows better on a model weaker than the frontier model (in the pilot the large model was not discriminated, at BRB 24/24).
- **sonnet for the cross-comparison:** to tell whether a result is a trait of one model or a trait of the language and tools, one more model of a different character is needed. It is of the same family as the pilot so far (Claude Code CLI), so it can also be read in continuation with the pilot results.

## Consequences

- The harnesses (`bench/m1`·`bench/brb`·`bench/bvb/verus`·`bench/bnb`) make agents with `model_api.make_model_agent`. The Anthropic agent uses a standard library client instead of the SDK (the same `messages.create` shape, the setting stays `effort: high`).
- The MANIFEST of BRB contains `harness.py`, so it was frozen again. What changed is only the agent kind branch and the `+dirty` verdict; tasks, prompt and verdict are unchanged, and BRB has no formal measurement results yet (pilot only). Re-freezing before formal results exist has a precedent (699ebdc0). The MANIFESTs of m1·m1b·BVB do not contain harness code and are unchanged.
- The `+dirty` verdict of the harness does not count `bench/*/results/`. This was a defect in which, when one run goes through the sets in turn, the result files of the earlier set caused the later set to be stamped `+dirty` (excluded from reports).
- Properties of the gateway (2026-10-30, measured after the first qwen run stopped in m1):
  - A non-streaming request is cut off at about 30 seconds with HTTP 502 (`upstream request failed`). Re-sending the same request gives the same. So the adapter sends every request as streaming (SSE) and folds the events into the non-streaming response shape. The same 16,000-token generation finished in 147 seconds with streaming.
  - The context of `qwen3.8-27b` is 64K. If the prompt exceeds about 65,000 tokens it is HTTP 400. An error that a retry does not fix ends only that session with `stop: "api_error"` (the message is in `model_usage.api_error`) and the verdict is given on the state at that time. The report counts these sessions separately.
  - The stream is sometimes cut in the middle (proxy tunnel closed). It is treated as a transport failure and the same request is re-sent (at most 6 times, about 2 minutes).
- The output limit of qwen is 32,768 tokens (the length Qwen recommends for thinking mode). At 16,000 there were cases where the first turn used up the limit on reasoning alone and ended (m1 `w05-fill-from`). sonnet keeps the existing settings (16,000, `effort: high`). The comparison arms (BMB vs Rust·Verus) use the same model and the same settings, so fairness (ADR-0034 §2) is unchanged.
- `bench/run_models.py` accepts both `openai:` and `anthropic:` and does not require `ANTHROPIC_API_KEY`. The gateway does not report prices, so the price arguments stay (0 for local qwen, the provider's price list for sonnet).
- Data: qwen runs on the gateway's local machine. sonnet goes out to an external provider, and by the gateway's labelling it has 30-day retention and no training use. What is sent is only the frozen tasks and BMB code.
- Backlog: T0-03 proceeds on this path. The formal qwen measurement (m1·m1b·BVB·BRB) and the sonnet cross-comparison are the next tasks.
- The mission goal verdict of 2H (M2-e) is made with this measurement. Which model's value is compared with the target value is decided in 2H. Until then the two models are reported together.
- Revert: an ADR that supersedes this one (approved by the maintainer).
