# ADR-0068: Compact agent projection (`bmb mcp --compact`)

- Status: Accepted (autonomous, AUTONOMY.md §4: an optional flag of the MCP server. The default output, the map schema and the frozen sets are unchanged). Switching the default is an item for the maintainer's approval after the measurement of §5
- Date: 2026-10-30
- Related: DESIGN.md principles 6·7, §7.2, ADR-0019 §2 (12-character hashes in the map), ADR-0020 (map schema), ADR-0021 (freeze), ADR-0066 (qwen 64K), backlog 2D-23·2D-25·T1-05·T0-15

## Context

- Context per turn (2D-25, `dev-docs/experiments/context-size/`): in a large workspace (m1b), 63% of the context of the last request is read results. Dumps such as `query units` (all of L0) and `map L1` (3.1 times the normal-form code) carry on every row three 64-digit hashes, file and line, proof cost and empty fields. In every writing session `write` and `verify` return the status lines of the whole workspace, including the definitions that succeeded.
- The context of the formal qwen measurement (ADR-0066) is 64K. In m1b a session averaging 47,733 tokens per request ended at the output limit and failed. Context size is not only a cost but a factor in the success rate.
- L1 for agents must not be larger than the code that answers the same question (T1-05, principle 7). Now it is 3.1~3.6 times.
- MCP output and tool descriptions are frozen conditions of measurements in progress and planned (T0-10·T0-11·T0-13·T0-14·2A-07), so the default cannot be changed now (ADR-0021).

## Decision

1. **Projection.** `bmb mcp --compact` passes every tool result through `view::compact`. It is a pure function computed from the result JSONL alone.
   - The map header line is removed.
   - L1 surface: after one line `// path: status`, the normal form of the definition without its body (attribute lines, signature, contract clauses, the layout of ADR-0019 §1). The status is `verified`, `failed F of N obligations`, or omitted if there are no obligations.
   - L0: a unit is `#[summary("..")] mod path;`, a definition is `kind path`.
   - L2: `FROM KIND TO` (with ` arg N`, ` <type>` when needed).
   - `slice`: for the definition and each dependency, the source code after `// path` (no JSON escaping).
   - Proof results (`verify`, `write`, `build`): diagnostics as they are, the status lines of successful definitions as one line `{"verified":N}`, and the remaining status lines without proof cost.
   - Other JSON rows: `level` and empty values (empty array, `null`, empty string) are removed and hashes are `sha256:` + 12 characters (ADR-0019 §2).
2. **The default and the CLI are unchanged.** Without the flag, output, tool descriptions and `instructions` are the same bytes as before. `bmb map`·`bmb query`·`bmb verify` and the map schema (ADR-0020) stay JSONL. Judging and tools use that side.
3. **Tool descriptions.** With the flag, the descriptions of `query`·`map`·`verify` (and `write`) state the format that will be received. Names and input schemas are the same.
4. **The measurement edition is only prepared.** `bench/view/gen.py OUT` makes `m1b-write-compact`·`bvb-write-compact`, which add only the one server flag `--compact` to the frozen `m1b-write`·`bvb-write` (prompt, tasks, seeds and mutants are the same bytes). The freeze (generate and commit with the MANIFEST) is done by T0-15 just before the first run. The gate only checks that the reference and weak solutions pass and are rejected on the generated edition (`bench/view/test_gen.py`).
5. **Condition for switching the default.** (a) After the measurements of the current surface T0-10·T0-11·T0-13·T0-14·2A-07 are done, (b) T0-15 runs the compact edition with qwen at the same commit as the original edition (`*-write`) (sonnet too at the same commit, per ADR-0066 §3), and (c) if the first-attempt success rate of writing and navigation and the BSB discrimination rate are not lower than the original edition and prompt tokens per request go down, an ADR that changes the default to `--compact` is submitted for the maintainer's approval. Measurements after the change are reported under the new edition name and not mixed with earlier results (ADR-0021 §1).

## Rationale

- **Cause layer (JUDGMENT §3):** hashes, positions and costs are map fields used by tools (incremental cache, duplicate finding, diagnostic positions), and MCP has no tool that takes a hash or a position. The cause is the format shown to the agent, so it is fixed in the projection of the tool layer. Removing hashes from the map schema is a measure in another layer that loses information tools use, so it is not chosen.
- **Comparison of candidates (the surface of the same whole workspace, estimated tokens relative to normal-form code, three seeds):** 12-character hashes 1.88~2.25, hashes removed 1.40~1.68, positions and costs also removed 0.99~1.21, empty fields also omitted 0.82~1.02, Rust shape 0.44~0.59. Only the Rust shape is at or below code outside the estimation error (−7%~+4%) (C3·S4).
- **Canonical way (S1):** showing signatures in source shape is the method used by rustdoc item signatures, IDE hover, and repository maps of agent tools (Aider repo map: a code-shaped skeleton of the definition lines per file). It also fits the definition that BMB's surface is Rust (DESIGN §4). The normal form without body uses the layout rules of `bmb fmt` as they are (S3: no new notation is made).
- **Determinism (C2):** the projection is a pure function of deterministic JSONL. The same input gives the same bytes.
- **Folding of proof results:** the list of status lines of successful definitions is proportional to workspace size and is not used for the next action. Failures and diagnostics stay as they are, so the ways to fix (counterexamples, fix suggestions) are not reduced (HEALTH §5.3).
- **Effect (re-run, the same calls of pilot sessions):** tool result tokens −74%~−79%, last request m1b −57%, m1b-write −54%, BVB −9~−12%. The maximum of the lower bound of the last request of qwen m1b sessions 40,150 → 11,434.

## Consequences

- What the agent does not see in the compact projection: hashes (12 characters in the other rows), file and line, proof cost, the obligation count of successful definitions, the list in L0 unit rows (the same information from the definition lines that follow). When needed, `query status` gives obligation counts by status.
- The verdict of the harness (JSONL of the `bmb` CLI) and the metric (the byte approximation of `read_bytes`) are unchanged. The "tokens per query" of the compact edition becomes a reduced value on the same ruler as the original edition.
- In a small workspace like BVB, most of the difference from Verus (52%) is the fixed part (the language rules prompt, the duplication between tool descriptions and the tool section of the prompt), so this projection does not reduce it. Backlog `2D-28`.
- With `BMB_CONTEXT_LOG=1` the formal harness records tokens per request and result bytes per call (off by default so result rows are the same bytes as before; what the agent sees is the same when it is on). T0-15 runs with it on and gives the breakdown without estimation.
- Revert: remove the flag. The default and the frozen sets do not rely on this decision.

## Recommendation: switching the default (JUDGMENT §10, item for the maintainer's approval)

1. **Problem:** the map and proof results the agent sees are 3 times the size of the code that holds the same information, and in the qwen 64K context one session hit the end of the context and failed (m1b `w11-clamp-all`).
2. **Cause and layer:** the tool result format (map fields shown as they are, successful status lines listed). The tool layer.
3. **Canonical solution:** a source-shaped signature projection and results that show only failures in detail (rustdoc, the diagnostic conventions of compilers).
4. **Candidates:** (A) Switch the default now: violates C3 (the conditions of measurements in progress change). (B) Keep it as an optional flag and switch after measuring the new edition: passes C1~C4, best on S4. (C) Remove hashes from the map schema: another layer (violates S2). (D) Keep JSON and omit fields: at 0.82~1.02 relative to code it does not clear the check criterion with margin.
5. **Recommendation:** (B). The criteria that decided it are C3 and S4.
6. **Impact (information):** after the switch, measurements of the frozen sets use the new edition. `compiler/README.md`; the prompt does not state the format, so it is unchanged.
7. **Approvals needed:** switching the default (a change of measurement conditions; it touches the accepted measurement procedure and the composition of editions).
8. **Workarounds:** none.
