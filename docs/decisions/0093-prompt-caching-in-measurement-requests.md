# ADR-0093: Prompt caching in measurement requests

- Status: Accepted (2026-11-06, instruction of the maintainer: passing on a request of the gateway operator. A measurement operation decision within the scope of ADR-0066·ADR-0070)
- Date: 2026-11-06
- Related: ADR-0066 (measurement models and environment), ADR-0070 (measurement conditions), ADR-0076 (output limit and continuation), ADR-0088 (gateway capacity failures), backlog INF-27

## Context

- What the gateway operator confirmed: from 2026-10-30 to 11-06, in about 4,900 `claude-sonnet-5-5` requests and about 50 million input tokens, cache reads are 0 and cache writes total about 10 thousand tokens. The gateway passes `cache_control` through unchanged, and both writes and reads worked on the production path. So the requests had no `cache_control`.
- That traffic is the measurement harnesses. Every harness sends through `bench/m1/anthropic_agent.py` (the standard-library HTTP client `model_api.HttpAnthropic`), and it had no cache marker. Claude Code sessions do not go through this gateway (`ANTHROPIC_BASE_URL` is a different host), and `DISABLE_PROMPT_CACHING` is not set.
- In the committed sonnet session records (2026-11-04 to 06, 456 sessions, 2,894 requests), the input per request averages about 10,900 tokens, the requests per session have a median of 4, and the output per request has a median of 460, a 99th percentile of 4,466 and a maximum of 14,063 tokens. A request of a session only appends to the prompt of the previous request (the continuation of ADR-0076 also only adds a block to the last user turn).
- The cache changes the price of a request and does not change the input the model sees. But `usage.input_tokens` of the response counts only the remainder outside the cache, so adding it up as is would make the token indicator of the measurement (the total tokens of ADR-0021) shrink with cache hits.

## Decision

1. **Anthropic requests turn the cache on.** An explicit marker on the last tool definition (tools are rendered first, so the sessions of a set that use the same tool list share one entry), and automatic caching at the top level of the request (`cache_control: {"type": "ephemeral"}`, the marker follows the end of the growing conversation). The system prompt stays a plain string (turning it into blocks changes the request body, item 3).
2. **The TTL is 5 minutes.** The next request of the same session starts after the generation of the previous response and the tool execution, and the generation is mostly a few hundred tokens, so it is within 5 minutes. The 1-hour TTL costs 2x on writes, so it gives no gain at this interval. If measurement often shows intervals over 5 minutes, this is decided again.
3. **The token indicator is the whole prompt.** `input_tokens` = outside the cache + cache writes + cache reads. Cache writes and reads are counted separately as `cache_creation_input_tokens`·`cache_read_input_tokens` (in the usage and in the per-request record). So it compares with past results under the same definition.
4. **The cache is not a measurement condition.** `condition.py same` compares request bodies and result rows with `cache_control` and the cache token counts removed (`condition.without_cache`). If what the model sees and the verdict are the same, it is the same condition.
5. **qwen (OpenAI-compatible) requests are not changed.** The operator's request is about the Anthropic path, and the OpenAI-compatible API has no request-side cache marker.

## Rationale (alternatives considered)

| Option | Verdict |
| --- | --- |
| **Marker on the last tool + automatic caching, 5 minutes (adopted)** | Reads all of the repeated prefix within a session and the tool prefix within a set. The request body only gains the markers, and the verdict and the model input are unchanged |
| Automatic caching only | The same within a session, but the first request of a session (1 of a median of 4 per session) is always full price |
| A marker on the system block too | The system prompt would have to become a list of blocks, so the request body differs. The tool marker already covers the common prefix at the front |
| 1-hour TTL | 2x on writes. The request interval is within 5 minutes, so reads do not increase |
| Not turning it on | The same input is paid again at full price on every request. That much of the daily spend cap (ADR-0088), which sets the measurement volume, is used up sooner |

## Consequences

- Future sonnet measurements run more sessions on the same spend cap. The gateway spreads requests over five vendor accounts, so at first one write occurs per account (operator's note).
- The check is `cache_read_input_tokens` in the result rows. If reads are 0 in the first real measurement, something is changing the prefix, and the request bodies are compared.
- Results already committed stay valid (they were measured without the cache, and the token definition is the same).
