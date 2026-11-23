# ADR-0025: The second task set `m1b`

- Status: Accepted (autonomous, AUTONOMY.md §4)
- Date: 2026-10-24
- Related: ADR-0016 (M1 scope), ADR-0021 (measurement procedure), ROADMAP 2A, backlog 2A-01~2A-05

## Context
`m1` (ADR-0021) is a seed of 23 definitions and 15 tasks. The stage 2 pass criterion is "the 5 measurements do not get worse on a task set wider than 1E" (ROADMAP stage 2), and that set must be frozen before the features (genesis §3.5). `m1` measures only single-unit writing and one-step navigation on a small seed, so the situations where the benefit of principle 6 (through tools) grows are missing: a large seed, modifications across several units, and impact analysis.

## Decision

### 1. Scope
- The language does not go beyond the M1 subset (ADR-0016). Features from 2B on, such as structs, enums and conversions (`as`), are not used. Tasks that use features from 2B on are made as a new set (`m2` and so on).
- The prompt is `m1`'s `prompt.md`, byte for byte. Only if the difference between the sets is the tasks and the seed can the `m1` and `m1b` results be read side by side. The MCP `query` tool already accepts the `impact` query, but the prompt does not mention it. Whether the prompt tells the agent about a tool is a variable measured separately in the 2D (tools) phase.

### 2. Seed
- `bench/m1b/seed/src/`: 9 units, 116 definitions (53 executable functions, 63 `spec fn`), all proved. The normal form of `bmb import`.
- Integer and array algorithms: bounded arithmetic (`num`), number theory without bit operations (`arith`), array basics (`arr`), linear and binary search (`search`), sortedness checks and selection sort helpers (`order`), statistics (`stats`), an array of balances with an upper bound (`ledger`), a job state machine (`fsm`), the `io` entry point (`app`).
- It includes call paths of depth 3 or more (`app::main → ledger::pay_all → ledger::transfer → ledger::deposit`, `app::report → stats::spread → stats::max_of → num::max2`) and `&mut` write paths (the same `ledger` path, `order::selection_sort → order::select_step → arr::swap`).

### 3. Tasks
37 tasks: 25 write, 12 navigate. The format is the same as `m1` (ADR-0021 §1·§2). There are three types.

| Type | Tasks | Judging |
| --- | --- | --- |
| Write a new definition | w01~w11, w22~w24 (the `tasks` unit), w13·w25 (added to an existing unit) | As in ADR-0021 §2 |
| Strengthen an existing definition (several definitions, impact) | w12 `stats::spread`, w14 `ledger::transfer`, w15, w16, w17 `fsm::run`, w18, w19 `order::select_step` | Specification strength of the target + proof of the whole workspace. w12·w14·w17·w19 are proved only if the callee's contract is strengthened too, so whether its callers (the impact range) stay proved is part of the judging |
| Write across several units | w20 (`num::abs_diff` and `tasks::max_gap`) | Signature and specification strength are judged separately for each definition in the task's `also_check`, and all must pass for success |
| Navigate | n01~n12 | The harness computes the correct answer with map queries. Direct callers, transitive callers and callees (`closure`), `impact` rows (n04~n07), contract view, `&mut` write edges, uses of `#[uses]`, effects |

- Every write task has a reference solution and a weak control (`weak_reference`). The weak control of the strengthen-an-existing-definition tasks (w15~w18) is the empty transaction, that is, "the answer that leaves the seed as it is".
- The correct answer of the impact analysis navigation tasks (n04~n07) is the rows of `bmb query impact DEF iface|body`. A proof sees a callee only through its contract and a `spec fn` only through `#[uses]`, so the impact stops after one step (ADR-0017 §4). Answering with transitive callers lowers precision (n07).

### 4. Freezing and comparison rules
- `bench/m1b/MANIFEST` holds the SHA-256 of `prompt.md`·`seed/src/*`·`tasks/*`. The harness selects this set with `--set bench/m1b` and refuses to run if MANIFEST differs (the same as ADR-0021 §1). `bench/m1` does not change.
- The summary row of a result file records `task_set` (the set name) and `manifest` (the SHA-256 of MANIFEST).
- **Sets are not mixed.** `harness.py summarize` does not put files with different `task_set` into one table and refuses (an earlier result without `task_set` is `m1`). Reports also do not add up or average the figures of `m1` and `m1b`.
- **Model comparison** is done only through the columns of `summarize`, between result files run on the same set, the same MANIFEST and the same compiler commit. The result file is `bench/<set>/results/<date>-<model>.jsonl`. If the compiler commits differ, the diagnostics and hints differ, so this is shown in the "compiler commit" row of the table and the difference is stated in the comparison.
- One session per task, no retries, and the limit (40 tool calls) are as in ADR-0021 §3. To change a task, a new name (`m1c` and so on) is made.

### 5. CI
The verification gate (`scripts/check.sh`) confirms on `m1b` that every reference solution succeeds and every weak control fails (ROADMAP 2A pass criterion 2).

## Rationale
- With the same prompt, the difference between the sets is limited to the tasks and the seed. Changing the prompt breaks the comparison with the `m1` measurement.
- The "strengthen an existing definition" tasks make the agent handle the ripple of a contract change in a large workspace. Proof of the whole workspace is the success condition, so an answer that misses an affected caller fails. These are the write tasks where knowledge of `impact`·`#[uses]` is an advantage.
- Setting the correct impact answer to the output of the `impact` query lets the harness compute it deterministically, and it matches the prover's actual dependency rule (ADR-0017 §4).
- Alternatives: add tasks to `m1` → dropped because it violates the freezing rule (ADR-0021 §1). Several seeds → a different seed per task makes navigation cost hard to compare, so there is one.

## Consequences
- The harness takes the task set directory as an argument (`--set`), and supports `reasons` (`impact` rows), `closure` (transitive) and several `also` queries in the navigation answers, and `also_check` in the write judging. The `m1` task files do not use these keys, so their judging does not change.
- The seed is large, so the proof time per task is longer than in `m1`. The increase in gate time is noted in the record of backlog 2A-04.
- Bias: integers and arrays only (ADR-0016), and every write task was made by a single author together with its reference solution. This is stated in the measurement report.
