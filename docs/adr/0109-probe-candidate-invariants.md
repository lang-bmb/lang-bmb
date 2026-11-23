# ADR-0109: A batch probe that does not commit: a read tool that filters candidate invariants

- Status: Accepted (autonomous, AUTONOMY §4 "decision records": a tool-layer decision that adds one read tool. The write surface ADR-0101 and the language stay as they are. Ninth execution plan, approved by the maintainer "recommendation accepted")
- Date: 2026-11-10
- Related: ADR-0101 (the canonical write surface `write`), ADR-0108 (information in undecided responses), ADR-0068·0071 (behind a flag before measurement), DESIGN §3 principle 6, backlog 2D-35·T0-37

## Context

- In `t14-dedup-sorted` of sonnet `brb-write`, 22 of 34 `write` calls were undecided. Most were "change one candidate and rewrite the whole definition" (AI user opinion 2026-11-10 P2).
- `write` is an operation that commits. So every exploratory attempt stays in the workspace and in the conversation history.
- Producing several candidate invariants is something an agent does well. Telling whether they are right is a deterministic computation. This is the division of labour of principle 6.

## Decision

1. **The tool.**
   - The CLI is `bmb probe --def PATH [--loop K] --candidate EXPR..`.
   - The MCP is `probe {def, loop, candidates}`, and it is listed only with `--probe-tool`.
   - It takes up to 16 candidate expressions for the K-th loop of one function (statement `while`·`for` and `loop` expressions, in source order, from 0).
   - It does not change files or the workspace.
2. **The verdict (Houdini).**
   - Each candidate is parsed as a wrapper (`fn __candidate() -> bool { C }`) with its own file id. So the file tells which candidate a diagnostic belongs to.
   - In a copy that adds all remaining candidates to the invariants of that loop, only that one function is checked and proved.
   - Candidates that have a check error, or whose entry (E0806) or preservation (E0807) fails or is undecided (the `obligation` of E0811), are removed and it runs again. When no candidate is removed, the remaining set is the largest subset that is inductive together.
3. **The report.**
   - A remaining candidate is `holds` on both.
   - For a removed candidate, it gives the round in which it was removed and only the side that failed or was undecided. The other side is `-`. This is because the prover assumes a failed obligation afterwards, so a "holds" in a round in which another candidate failed can be vacuous. A failure has a model.
   - If there is a counterexample, it is included.
   - At the end it gives the remaining set and whether the function is proved with it (`verified`).
4. **Behind a flag before measurement.** The MCP tool exists only with `--probe-tool`. The tool list of the frozen sets stays as it is. T0-37 measures the effect with an edition that adds only this one flag.
   - The metrics are tokens per success and the number of `write` calls per session.
   - If only the tool calls increase and the token difference is within D (ADR-0077), it is refuted.

## Rationale

| Option | Verdict |
| --- | --- |
| **Houdini that proves one function in a copy (accepted)** | It uses the prover's verdict as it is. No new encoding is built. The result is deterministic |
| Prove each candidate separately | It cannot filter invariants that lean on each other (the preservation of one candidate needs another candidate) |
| A "do not commit" option on `write` | The write surface (ADR-0101) changes, and it cannot filter several candidates at once |
| Report the answer of the first round as it is | A "holds" after the failure of another candidate can be vacuous (confirmed during implementation: in the round in which `i > 5` failed at entry, the preservation of `s == 0` "holds") |

## Consequences

- `compiler/src/probe.rs`, `bmb probe`, MCP `probe` (`--probe-tool`).
- Tests: `probe_keeps_the_largest_set_that_holds_together` in `tests/verify.rs`, `probe_tool_on_request_and_read_only` in `tests/mcp.rs`.
- The measurement is T0-37 (after the evidence stage).
