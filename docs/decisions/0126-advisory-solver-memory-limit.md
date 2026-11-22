# ADR-0126: The advisory queries of `lint` run under a separate memory limit

- Status: Accepted (autonomous, AUTONOMY §4 "decision records": `T2-16` of the fourteenth execution plan, approved by the maintainer 2026-11-11 "proceed with the recommendation"). It supplements ADR-0123. Decision 2 (`rlimit` stays as it is) is superseded by ADR-0131 (the overall limit of advisory queries is `Z3_ADVISORY_RLIMIT`)
- Date: 2026-11-12
- Related: ADR-0123 (the memory limit of the solver), ADR-0067·0069 (probes), `scripts/toolchain.env`, `compiler/src/smt.rs`, backlog T2-16

## Context

- After ADR-0123, `bmb lint` still took 248 seconds on `port-shape/tree.bmb` (340 lines). About 240 seconds of that are 38 queries that ended at the memory limit (1,024MB) (3 to 30 seconds each). All other queries are 19ms or less.
- Those queries are satisfiable queries: when a recursive predicate (`spans_in`, with a quantifier inside) is in the context, the false obligations of mutants or removed clauses, reach points, precondition checks. Z3 ends by memory while building a model, before it hits `rlimit`.
- `lint` is an advisory tool that emits warnings, and it is not the gate. All of its reports need `unsat` (surviving mutants, unneeded clauses, W0901, W0905).

## Decision

1. **The advisory queries of `lint` run under `Z3_ADVISORY_MEMORY_MB` (128).** These are the precondition check, result determinacy, reach points, and the three proofs of editions with a clause removed or the body changed (W0902·W0903·W0904). The first proof of `lint` keeps the limit of the gate: the meaning of "proved" is the same as in the gate.
2. **`rlimit` stays as it is.** A runaway query ends by memory below `rlimit`, and advisory queries that are decided use up to 98% of `rlimit` (sha256). Lowering `rlimit` also changes the key of the proof cache.
3. **An advisory query that hits the limit is undecided.** It is reported neither as survived nor as killed.

## Rationale

| Option | Verdict |
| --- | --- |
| **A fixed small memory limit for advisory queries (accepted)** | A smaller limit only turns decided into undecided. Every report requires `unsat`, so reports can become fewer but no wrong report arises. It is a fixed value and not time, so determinism is kept |
| Lower the `rlimit` of advisory queries | It cannot separate runaway from normal (2 above) |
| Skip the mutation of definitions that run away | It hides the coverage |

Grounds for the value (13 files, 5,575 advisory queries): the maximum memory raised by a proof (`unsat`) is 20.8MB, and the maximum over all decided queries is 71.6MB. At 256, 128 and 64MB the `lint` output is byte for byte the same as before on the 12 files other than `tree.bmb`. 128 is twice the lowest level with the same output, and six times the largest proof.

## Consequences

- `lint` on `tree.bmb`: 248 seconds → 21 seconds (quiet machine, debug build). The 19 report lines and the number of mutants stay the same.
- A defect fixed along with it: the reach-point loop kept using a dead process after a solver error, so the later points were counted as undecided without being asked (a wall-clock dependency tied to the 60-second process reuse deadline). Now a new process is asked after an error. `reach_undecided` of `tree.bmb` went from 12 → 7 (of the 8 that were not asked, 5 are reached and 3 are undecided).
- Tests: `compiler/tests/lint.rs` `advisory_queries_end_at_their_own_memory_limit`, `smt.rs` `advisory_queries_run_under_their_own_memory_limit`.
- What remains (backlog `T2-17`): the memory accounting is per process, and a process is reused for several queries. The verdict of a query that uses close to the limit can depend on the earlier queries (this also applies to the gate's 1,024MB; the largest proof in the current corpus is 20.8MB). After a probe ends by memory, the main query goes to the dead process. If a proof from `--cache` is reused in advisory queries, the output can differ by whether there is a cache.
