# ADR-0144: Conditions of the first measurement of the porting twins

- Status: Accepted (approved by the maintainer 2026-11-17 "recommendation accepted, model runs approved, autonomous execution/verification approved": decisions 1·2·3 are that recommendation, decision 4 is a task breakdown)
- Date: 2026-11-17
- Related: ADR-0021 (freezing), ADR-0068 (compact view), ADR-0101 (official write surface), ADR-0104 (porting twins), ADR-0106·0118 (documentation conditions), backlog 3P-02·3P-03·3P-05·3P-10

## Context

When the task sets (24 comprehension, 21 refactoring) were run to the end for the first time, four things to settle before measuring showed up.

- **Write responses.** The `write` of `bmb mcp --write-tool` returns a status line for every definition of the workspace. In the port (1,515 definitions) the response of one write is about 40,000 tokens. The reference solution of a rename is 82,441 tokens in the BMB arm and 9 tokens in the Rust arm (3P-10). Measured as is, the tokens of the BMB arm are set by these lines, not by the amount changed.
- **Documentation.** The definition summaries of the port are in Korean and the `///` of stage0 are in English. The "same text" that the main condition of ADR-0118 (`summary`) presupposes does not hold.
- **Text search.** The Rust arm has `grep`, and the BMB arm has no tool that searches the text of bodies. The branches that ask by diagnostic code and field name (`diag`, `state`) measure this difference directly.
- **Replay tasks.** Commit replay (3P-04) needs the reverted state of both sides to be made by hand for each task, so it is later than the other two axes.

## Decision

1. **The server of the BMB arm is `bmb mcp --write-tool --compact`.** Proof results (`write`·`verify`·`build`) are the diagnostics, the status lines of failed definitions, and one line with the number of proven definitions (decision 1 of ADR-0068). Read results also come out in the compact view of the same flag. The compiler is not changed. `PAIR_MCP` of the harness sets the flags and `mcp` of the result row records them.
2. **The main documentation condition of the porting twins is `none`.** Neither arm has definition documentation (for BMB the `#[summary]` lines of definitions, for Rust all comments except `//!`). The commented Rust control (`all`, Rust arm only, sonnet 1 run, decision 2 of ADR-0106) runs as it is. The main condition of the `stk` fixture stays `summary`.
3. **The tasks stay as they are and are reported per branch.** The `diag`·`state` branches are not removed and text search is not added to the BMB arm. The verdict of an axis is made on the whole, and the per-branch values are put side by side in the table.
4. **The first measurement is split in two.** `3P-05a` is comprehension and refactoring, `3P-05b` is commit replay. Each set is frozen before its first run.

## Rationale

| Option | Verdict |
| --- | --- |
| **`--compact` (accepted)** | It is a flag that already exists and the response shape is the same as what 3P-10 wanted (the failed definitions and one count). No new surface |
| A new flag that reduces only the write responses | Two flags would do the same job (ADR-0120). There is no rationale for leaving out the read results of the compact view |
| Measure in the current shape | The token comparison measures the response format. That is not what the hypothesis (the total tokens of H-REFACTOR) asks |

- Whether the compact view lowers the success rate has not been measured yet (T0-15). This measurement is not a rationale for switching the default, and the results are reported together with the flags.
- `none` is a condition on the side of removing information from the Rust arm. The size of that bias is measured by the `all` control.
- The difference in text search is a real difference of the tool surface. Selecting tasks would make the measurement hide that difference. With per-branch values, where the difference comes from can be read.

## Consequences

- `mcp_flags` of `bench/twin/pair.py`, the condition table of `bench/twin/README.md`.
- The Rust reference solutions of the refactoring tasks (`old` of `edit_file`) are made to match the text of the `none` condition.
- Backlog: `3P-10` closed, `3P-05` into `3P-05a`·`3P-05b`.
- Whether the BMB arm needs text search of bodies is decided by the per-branch results of `3P-05a`.
