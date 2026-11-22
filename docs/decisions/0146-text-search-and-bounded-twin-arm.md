# ADR-0146: Text search of bodies `query text`; the BMB arm of the porting twins turns on the response window

- Status: Accepted (approved by the maintainer 2026-11-22 "recommendation accepted": the recommendation of backlog `3P-14`. Two things were fixed on acceptance: a long line shows only the surroundings of the occurrence (decision 1), and the description mentions this query only under `--bounded` (decision 3))
- Date: 2026-11-22
- Related: ADR-0020 (map and queries), ADR-0127 (response window), ADR-0142 (the model is how the agent works), ADR-0144 (conditions of the first twin measurement: "Whether the BMB arm needs text search of bodies is decided by the per-branch results of `3P-05a`"), DESIGN.md principles 6·7, `dev-docs/measurements/3p-05a.md`

## Context

- `3P-05a` (the 24 comprehension tasks of the porting twins). On questions answered by the edges of the map (callers, impact), the BMB arm reads only 5~9% of the Rust arm and the correct answers are equal or higher. Questions that can only be answered by reading the text of definitions (which function emits a diagnostic code, which function changes a field of the state) are the opposite: sonnet reads 23 times and 7 times as much and the correct answers are 10/12 versus 12/12.
- The reason lies in the tools. The BMB arm has no means of searching text, so it scans by reading definitions whole. The Rust arm takes one `grep`.
- A small model cannot handle those responses. In the 3 qwen runs, 41 sessions of the BMB arm were cut off by context overflow (HTTP 400) and 0 of the Rust arm. The tool responses read by the cut-off sessions are 50,000~200,000 tokens.
- The response window already exists (`bmb mcp --bounded`, ADR-0127: 32,768 bytes, `offset`·`limit`, truncation row). The condition of the first measurement (ADR-0144) was `--write-tool --compact`, and the window was off.

## Decision

1. **`query text PATTERN [PATH..]`.** One row for each line of the normal form source of the workspace that contains `PATTERN` (literal, case-sensitive): `{"path","line","text"}` (`path` is the definition or type that holds the line, `line` is the line number within the normal form of that definition, `text` is the line with leading and trailing whitespace removed, and for a line over 200 bytes, 200 bytes starting 60 bytes before the first occurrence: the normal form of the port has lines of thousands of bytes, so emitting lines whole makes the row size not proportional to the number of matches). At the end a summary row `{"summary":true,"pattern","matches","defs"}`. The scope arguments (`PATH..`) and the response window are the same as for other queries. The order is path, then line.
2. **Regular expressions are not accepted.** There is one form of searching (principle 8). Questions that need a regular expression (the places that write this field) are for a structural query to answer (decision 4).
3. **The BMB arm of the porting twins is `--write-tool --compact --bounded`.** The description of the MCP `query` mentions `text` only under `--bounded`: the description without the flag is that of the frozen set unchanged (ADR-0021, decision 3 of ADR-0127). The same tasks (`port/c*`) are measured again under the new condition (`3P-15`). The results of the first measurement stay as the values of that condition.
4. **What is not in this decision.** (a) Read and write edges of fields (as a kind of `edges`: answering "the definitions that change this field" by structure, not text) are decided after seeing the state branch results of `3P-15`. (b) The cost of refactoring (H-REFACTOR: 3.5 times the tokens) is a matter of the write surface: the rename operation (`rename` of `apply`) was hidden on the measured surface (`--write-tool`). It is handled in 3C (preserving refactoring operations) and `T1-41` (H-EDIT).

## Rationale

| Option | Verdict |
| --- | --- |
| **Text search + window (recommended)** | The two causes differ: having no means of searching, and the response having no upper bound. With text search the response is proportional to the number of matching lines (principle 7). The tool does deterministically what the agent actually does (`grep`) (ADR-0142, principle 6). The window is a flag that already exists |
| Turn on only the window | The cut-offs decrease but the amount scanned stays the same: it reads all the definitions, paging through the windows |
| Add only structural queries (definitions that emit a literal, definitions that change a field) | Accurate, but one query is added per question. For a question not known in advance there is no means. The right order is to have text search and then promote frequent questions to structural queries |
| Leave it and limit the claim to edge questions | The sentence of H-COMPREHEND (diagnostic paths, where an invariant is kept) would have to be cut down. That covers a gap in the tools with the scope of the hypothesis |

- The normal form has no comments, so text search never matches dead text. Summaries (`#[summary]`) do match: that is useful when what is searched for is a meaning.
- Freezing condition: the tool list and descriptions of the server without flags are unchanged (the query `text` answers only when called). The BMB arm of the twins has one more flag, so it is not the same condition as the first measurement. That is why it is measured again.
- `edges` already has a `writes` kind. Decision 4 (a) first looks at whether it answers the questions of the state branch.

## Consequences

- If accepted: the query `text` of `compiler/src/map.rs`, the query table of `compiler/README.md`, the description of the MCP `query`, tests (determinism, window, scope), `mcp_flags` of `bench/twin/pair.py` and the condition table of the README, backlog `3P-15` (the same tasks under the new condition: sonnet 1 run, qwen 3 runs).
