# ADR-0129: An undecided result caused by memory is judged only on the first query of a fresh process

- Status: Accepted (autonomous, AUTONOMY §4 "decision records": backlog `T2-17`). Supplements ADR-0123 and ADR-0126
- Date: 2026-11-12
- Related: ADR-0018 (fresh state per obligation), ADR-0123 (memory limit of the solver), ADR-0126 (memory limit of advisory queries), `compiler/src/smt.rs`, `compiler/src/cache.rs`, measurement `dev-docs/measurements/t2-17.md`

## Context

- Z3's `-memory` counts the total allocation of the process. The process keeps a little for every query it answers: sending the 2,213 queries of `lib/std` to one process, the allocation right after `(reset)` grows from 0.10MB to 0.45MB.
- So the verdict of a query that uses memory close to the limit depends on the queries before it. At `-memory:18`, of 20 queries that answer when asked alone (peak 17.96~17.99MB), 16 ended at the limit after 790 queries. The opposite direction exists too: of 5 queries that end when asked alone (peak 18.00MB), 3 answered when one query came before them. The results of asking alone were the same over 12 repetitions.
- A process is reused for up to 60 seconds (wall clock) after it starts, so which queries gather in one process also depends on the wall clock.
- The window is about 0.05MB, so it does not show in the current corpus (the full run of `lib/std` with the limit lowered to 19~24MB and the `--def` single runs gave the same verdicts on all 22 failing definitions).
- The key of the proof cache did not contain the memory limit. A proof found under the gate limit answered an advisory query of the same text (a smaller limit), so the `lint` output depended on the contents of `--cache` (reproduction: the previous edition of `tests/lint.rs` `a_cached_gate_proof_does_not_answer_an_advisory_query`).
- When a probe ended at the limit, the full query went to the ended process and became undecided without being asked, and a query that ended at the limit did not get the ground instance retry.

## Decision

1. **An undecided result caused by memory is judged only when the query reached the limit as the first query of a fresh process.** The state is the same for every query, so the verdict does not depend on earlier queries.
2. **A process that handles several queries runs at half the limit** (`smt::working_memory_mb`: gate 512MB, advisory 64MB). A query that ends there is asked again from `(reset)`, alone in a fresh process, at the full limit, and that answer is the verdict. A query answered under half is taken to have answered within the limit when asked alone too (the assumption below).
3. **The query after a query that ended at the limit is asked alone right away.** Where it is asked does not change the answer, so it is a matter of cost (the queries of a false obligation reach the limit one after another).
4. **A query ended by a limit (`rlimit` or memory) is undecided, the same as `unknown`.** The full query after a probe, the ground instance retry and the range search follow as before. If the retry does not decide, the reason stays the limit.
5. **The memory limit of the query goes into the key of the proof cache** (format v5, also into the proof key of the incremental cache). A gate proof does not answer an advisory query. The re-verification of `lint` solves unchanged obligations once more at the advisory limit.

## Rationale

| Option | Verdict |
| --- | --- |
| **A working process at half the limit, asked again alone when it ends (accepted)** | The definition of the verdict is "the result of asking alone", and ordinary queries cost the same. It leans on one assumption (below) |
| A fresh process per query (Why3's way) | No assumption. The first query of a fresh process takes 13.2ms, a following query 2.3ms (200 runs): verifying `lib/std` (2,213 queries, 23 seconds) costs about 24 seconds more |
| Ask only the ended queries again at the full limit | Does not prevent the opposite direction (ends alone, answers later; 3 measured cases) |
| As it is now | The verdict depends on query order and the wall clock (C2) |
| Cache: advisory queries use only the gate proofs of this run | When the incremental cache reuses a definition whole, the keys of its obligations are unknown, so the result differs by whether there is a cache |

Assumption: a query that the working process answered under half does not exceed the limit when asked alone. To break it, the peak of the same query would have to be larger when alone by half the limit (512MB, advisory 64MB). The measured difference is under 0.05MB in both directions, and the whole peak of the smallest query is 16.8MB. It is a response to a limitation of an external tool, so it sits in one place, `smt::Solver::ask` (JUDGMENT §3).

## Consequences

- At a fixed limit, the `verify` output of `lib/std` is byte-identical to before (218 definitions, verdicts and `rlimit`). The figures are in `dev-docs/measurements/t2-17.md`.
- Lowering the limit gives fewer undecided results (`lib/std`, 18MB: memory undecided 132 → 73). Obligations proven by the ground instance retry of queries that ended at the limit: 25 of 95 at 18MB, 4 of 12 at 19MB, 4 of 5 at 20MB.
- Inputs with runaway queries cost more: the `lint` of `port-shape/tree.bmb` keeps its 19 report lines and its CPU time goes 21.5 → 49.1 seconds (queries ending at the limit 44 → 82: all 35 newly given retries end at the limit again. 43 queries end at the working limit and are asked again).
- The process reuse deadline (60 seconds) no longer reaches the verdict.
- Tests: `compiler/tests/gate.rs` `a_memory_end_in_a_working_process_is_asked_again_alone`, `after_a_probe_at_the_memory_limit_the_query_is_asked`, `a_query_at_the_memory_limit_gets_the_ground_instance_retry`, `compiler/tests/lint.rs` `a_cached_gate_proof_does_not_answer_an_advisory_query`.
- Remaining: the time of `lint` (backlog `T2-17` (e)). The `rlimit` of advisory queries stays as in ADR-0126, and lowering it is a decision that can change the report of `lint`, so it is raised only as a recommendation.
