# ADR-0127: A response at the agent boundary is a window inside a byte budget

- Status: Accepted (approved by the maintainer 2026-11-12 "recommendation accepted/approved", the recommendation of backlog `2D-38`)
- Date: 2026-11-12
- Related: ADR-0020 (map and queries), ADR-0021 (frozen set), ADR-0068 (compact view), ADR-0070 (measurement conditions), `bound` in `compiler/src/map.rs`, `compiler/src/mcp.rs`, backlog 2D-17·2D-38

## Context

- Large responses remain even after query scopes (`2D-17`). In an N=10,000 workspace (`bench/m3/gen.py --defs 10000`, 500 units), an unscoped `units` is 1,192,293 bytes (10,501 rows), `edges DEF both 64` is 2,596,064 bytes (36,474 rows), and `status` is 2.5MB. An unscoped `status` ended a qwen session.
- Scoped responses are small (`units m0` 2,355 bytes, `status m0` 5,253 bytes). The problem is the starting point (`units` when the agent does not know what exists), answers that are large by nature, and the unscoped forms that can still be called.
- Row sizes range from 33 bytes to 9KB, so a row count does not bound the response size.

## Decision

1. **Window.** The window of a response (JSONL) starts at row `offset`, holds the whole rows that fit in the budget (bytes), and does not exceed `limit` rows. The first row is emitted even if it exceeds the budget (the window moves forward from any `offset`). The map header row is a row too.
2. **Truncation is recorded.** If the window is not the whole response, one row is appended at the end: `{"truncated":true,"offset","shown","rows","bytes"}`, and if rows remain, `"next"` (the `offset` to continue from) and `"hint"` (guidance to narrow by scope, `overview`, depth or edge kind). If the whole response fits in the window, the response is unchanged (the same bytes).
3. **It applies only at the agent boundary.** `bmb mcp --bounded` gives the results of `query` and `map` as a window with a budget of 32,768 bytes, and the two tools take `offset` and `limit` arguments. The tool descriptions mention the truncation row, `overview`, the scope arguments and the kind argument of `edges`. Without the flag, the tool list and the responses are unchanged (frozen set, ADR-0021). On the CLI it is optional: `bmb query|map --budget BYTES --offset ROWS --limit ROWS`.
4. **`overview [PATH..]`.** One row per unit: it replaces the lists of the L0 unit row (`units`·`defs`) with counts and adds the `types` count. The row size does not depend on the size of the unit.
5. **The budget is counted in JSONL bytes before the compact view.** The view of `--compact` is smaller than the JSONL, so the upper bound holds, and the window is the same rows regardless of the view.

## Rationale

| Option | Verdict |
| --- | --- |
| **A byte-budget window with a truncation row and continuation (accepted)** | The upper bound on the response size does not depend on the workspace size. The agent knows that the response was truncated and how large the whole is. The window is a pure function of the response body, so determinism holds, and the windows concatenated equal the whole (tested) |
| Row count limit | Row sizes differ by up to 270 times, so it does not bound the bytes |
| Reject the unscoped forms | The starting point disappears. In a small workspace the unscoped form is the cheapest answer |
| Truncate silently | What is absent gets read as not existing (a wrong answer) |
| Default on the CLI too | The consumers of the CLI are scripts and gates. If the output changes, they break and nothing is gained |

Measurement (N=10,000, budget 32,768): `units` 1,192,293 → first window 32,903 bytes (37 windows), `edges m0::all_inb both 64` 2,596,064 → 32,967 (80 windows), `overview` 67,313 bytes (501 rows, 134 bytes per unit, 3 windows), `duplicates` 120,437 (4 windows), `effects` 37,409 (2 windows). Scoped responses (`units m0`, `status m0`) fit unchanged in one window.

## Consequences

- The budget 32,768 is a reference value. The default for real models (success rate and tokens) is measured on the set frozen with `--bounded` (backlog `2D-39`, measurement machine). Until then no claim is made.
- The responses of `verify` and `write` (diagnostics and the status line) are not windowed. Their size has a different cause (the status line, ADR-0068) and is looked at separately.
- If the workspace changes between continuations, the windows come from different responses. It shows as `rows` and `bytes` changing.
- One flag is added (ADR-0120: the surface follows the hypotheses). After the verdict it is promoted to the default or removed.
- 2026-11-13 (`2D-39a`): `--bounded-16k` (the same window, 16,384 bytes) was added to compare two budget values. The frozen editions are `bench/bnb-bounded` and `bench/bnb-bounded-16k`. After the verdict of `2D-39`, only one of the two is kept or both are removed.
