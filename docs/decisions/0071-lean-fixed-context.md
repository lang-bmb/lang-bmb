# ADR-0071: Fixed context: tool documentation in one place, the tool descriptions; prompt language rules only for what cannot be learned from diagnostics

- Status: Accepted (Accepted (autonomous), AUTONOMY.md §4: an optional flag of the MCP server and a new measurement edition. The default output and the frozen sets are unchanged). Switching the default is an item for the maintainer's approval after the measurement of §4
- Date: 2026-10-31
- Related: DESIGN.md principle 6, ADR-0021 (freeze), ADR-0068 (compact projection, the same procedure), ADR-0070 (same condition), backlog 2D-25·2D-28·T0-17

## Context

- 2D-25: in the last request of the BVB `write` edition, 52% of the difference from Verus is the fixed part that goes into every request (2,467 vs 803 estimated tokens). The tool section of the system prompt (395 tokens) restates the tool descriptions, and the language rules section (726 tokens) is a list of prohibitions frozen at milestone 1.
- Probe (`dev-docs/experiments/context-size/rules.md`): of the 40 violations of the rules section, 7 are now accepted (stale prohibitions), and for 24 of the 33 rejections the diagnostic gives the form to fix to as a `fix`.
- The prompt and the tool descriptions are frozen conditions of measurements in progress and planned (ADR-0021, ADR-0070 (a)·(c)).

## Decision

1. **Tool documentation is in one place, the tool descriptions.** `bmb mcp --tool-docs` puts the tool documentation that was only in the prompt (what each query returns, narrowing down from `units`, `write` being the only way to write) into the `query` and `write` descriptions, removes design references (ADR and backlog numbers), and describes the `write` arguments only in the schema. Names and arguments are the same. Used together with `--compact`, the result format sentences are attached as before. Without the flag, `initialize` and `tools/list` are the same bytes as before.
2. **Only two kinds of prompt language rules.** (i) Rules for which the diagnostic gives no way to fix, (ii) rules that apply to almost every definition the agent writes and differ from Rust·Verus, so that learning them from diagnostics almost certainly costs a rejected write in every session. Other rules are left to diagnostics, and one line says so. What remains now: full paths, effects·return types·`let` types, named predicate contracts, opaque predicates and `#[uses]`, `#[trigger]`.
3. **Measurement edition.** `bench/lean/gen.py` makes `m1b-write-lean`·`bvb-write-lean` from the frozen `m1b-write`·`bvb-write`: it deletes the tool section, replaces the language rules section with the five lines of 2 and the one line, adds the server flag `--tool-docs`, and hides `build`, which no task uses. Seeds, tasks, mutants, examples and the working method are the same bytes. It was generated and frozen with the MANIFEST (ADR-0021 §1), and the gate checks equality with the generated result and that the reference and weak solutions pass and are rejected (`bench/lean/test_gen.py`).
4. **Condition for switching the default.** If `T0-17` runs this edition under the same condition (ADR-0070) as the original edition (`*-write`, T0-13) with qwen and then sonnet, and the first-attempt success rate of writing and navigation and the BSB discrimination rate are not lower than the original edition and prompt tokens per request go down, an ADR that makes `--tool-docs` the default and rewrites the prompts of all editions by the same criterion is submitted for the maintainer's approval. Measurements after the change are reported under the new edition name.

## Rationale

- **Cause layer (JUDGMENT §3):** the cause of the duplication in the tool section is that the documentation is in two places. The place that describes tools is the MCP tool description (tool layer), and the prompt has no reason to restate the tool list (S3: one place for one thing). The language rules section is a cost of the language layer, but teaching in advance the rules for which the diagnostic gives a way to fix goes against principle 6 (do not leave to the agent what a tool can do).
- **Canonical way (S1):** MCP fixes the tool description and input schema as the place for tool documentation. A compiler teaching language rules through diagnostics is the `help:` and suggestion convention of rustc.
- **Alternatives:** (i) The harness overrides the tool descriptions with `set.json`: tool documentation moves into measurement assets, another layer (violates S2). (ii) Delete only the tool section from the prompt and leave the descriptions: the meaning of the queries is lost (information loss). (iii) Delete the rules section entirely: `crate::` paths, for which the diagnostic gives no way to fix, and the rules that apply to every definition produce a rejection in every session. (iv) This decision.
- **Effect (estimated, `fixed.md`):** fixed part BVB 2,467 → 1,528 (−38%), m1b 2,433 → 1,494 (−39%). The fixed difference from Verus 1,664 → 725. The cost of learning rules from diagnostics and the success rate are not estimated but measured (§4).

## Consequences

- The stale prohibitions (syntax now accepted) stay in the frozen original editions. Only the new edition matches the current language.
- A gap on the diagnostics side: `crate::` and `super::` paths are syntax errors with no way to fix (backlog `2D-29`).
- Revert: remove the flag and the edition. The default and the frozen sets do not rely on this decision.
