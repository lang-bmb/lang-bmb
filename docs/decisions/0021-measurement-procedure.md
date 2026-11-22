# ADR-0021: M1 measurement procedure

- Status: Accepted (autonomous, AUTONOMY.md §4)
- Date: 2026-10-24
- Related: DESIGN.md §9 R4, genesis §3.5·§5, ADR-0016, backlog 1E-01

## Context
The hypothesis of M1 is "the agent writes proved code without files and without tests" (genesis §5). v0 raised its AI bench score while editing the problem statements, and reported with retries mixed in (genesis §3.5). The measurement procedure is fixed before the tasks.

## Decision

### 1. Tasks and freezing
- Tasks live in `bench/m1/tasks/<id>.json`. Each task holds the instruction, the starting workspace and the judging method.
- There are two task types.
  - **Write (write):** implement and prove a definition with a given signature.
  - **Navigate (navigate):** answer a question about the starting workspace with a set of definition paths. Examples: the affected definitions, the definitions that ensure a given predicate.
- The starting workspace is the BMB program in `bench/m1/seed/`.
- There is one instruction frame (system prompt), `bench/m1/prompt.md`.
- `bench/m1/MANIFEST` is the list of SHA-256 hashes of the task, seed and prompt files. The harness refuses to run if MANIFEST and the actual files differ.
- To change a task or the prompt after freezing, a new task set name (`m1b` and so on) is made, and its results are not reported mixed with earlier ones.

### 2. Judging
- **Write tasks:** success requires all three below.
  1. At the end of the session the whole workspace passes `verify`.
  2. The target definition has the required signature.
  3. **Specification strength check:** for the reference contract `ref_pre`·`ref_post` hidden in the task, the harness adds a judging definition and proves it. That definition has `requires ref_pre`, `ensures ref_post` and its body only calls the target definition. If it is proved, the agent's `requires` is weaker than the reference (callable) and its `ensures` is stronger than the reference (implies the reference).
  - So "a specification that is proved but says nothing" is a failure.
- **Navigate tasks:** precision and recall of the answered set of definitions against the correct set (computed deterministically from the map). Success if both are 1.

### 3. Session rules
- There is one session per task. A failed session is not run again (first-attempt success rate).
- Inside the session the agent uses only the MCP tools (`bmb mcp`). No file system or shell tools are given.
- Session limits: 40 tool calls, and a fixed number of model output tokens per task (recorded in the harness configuration). When a limit is reached, the state at that point is judged.
- Settings such as the model ID and temperature are recorded in the result file. Models are compared only on the same task set and prompt.

### 4. Metrics
| Metric | Definition |
| --- | --- |
| First-attempt success rate | Number of successful tasks / number of tasks, separately per type |
| Navigation accuracy | Mean precision and recall of the navigate tasks |
| Tokens per query | Mean `ceil(bytes / 4)` of the results of the read tools (`query`·`map`). The input tokens reported by the model are recorded as well |
| Rust-habit error rate | Number of E02xx among the diagnostics returned during the session / number of `tx_commit` calls |
| Unproved area | Number of failed obligations and number of `io` definitions at the end of the session |

The token unit is fixed as the byte-based approximation (`ceil(bytes/4)`). Per-model tokenizer values are recorded for reference only.

### 5. Reporting
- Raw data is committed to `bench/m1/results/<date>-<model>.jsonl` (one line per task), and the summary to `dev-docs/measurements/`.
- The summary states the task bias (integers and arrays only, ADR-0016), the number of tasks, the limits and the failure cases.
- The harness must be able to run to the end with a scripted agent (replay) without a real model, and that path is tested in CI.

## Rationale
- Without the specification strength check, a "successful proof" can be made with a specification close to `ensures true`. Judging the implication of the reference contract by proof measures the sufficiency of a specification without tests.
- The freeze hashes and the ban on retries directly prevent v0's measurement failures.

## Consequences
- Writing tasks has a cost. CI confirms for each task that the reference solution passes the judging (guaranteeing that the task is solvable).
- Running a real model uses a paid API, so it needs the maintainer's approval (AUTONOMY.md §4).
