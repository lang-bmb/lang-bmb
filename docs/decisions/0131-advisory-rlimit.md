# ADR-0131: Advisory queries run under their own `rlimit`

- Status: Accepted (approved by the maintainer 2026-11-13 "recommendation accepted and approved", backlog T2-18). Supersedes decision 2 of ADR-0126
- Date: 2026-11-13
- Related: ADR-0126 (memory limit of advisory queries), ADR-0129 (limits in the proof cache key), ADR-0067·0069 (probes), `scripts/toolchain.env`, `compiler/src/smt.rs`, measurement `dev-docs/measurements/t2-17.md` (e), backlog T2-18

## Context

- Of the 6,003 queries sent by the `lint` of `lib/std`, 403 were advisory queries that ran to the end of the gate's `rlimit` (5,000,000) and ended in `unknown`, and they were 89% of the solver time (T2-17 (e)).
- Every report of `lint` needs an `unsat`. The 1,601 `unsat` results of advisory queries were all under 256,942, and the 11 advisory queries that answered above 1,000,000 were all `sat`. The maximum of a gate proof is 485,775.
- Decision 2 of ADR-0126 was "the `rlimit` stays as it is": because the `rlimit` did not separate the problem of that time (runaway queries that end on memory). The problem of this ADR is other queries (they use little memory and only use the `rlimit` to the end).

## Decision

1. **The full queries of the advisory queries of `lint` run under `Z3_ADVISORY_RLIMIT` (1,000,000)** (`smt::query_rlimit`, the `(set-option :rlimit …)` in the query header). These are the precondition check, result determinism, and the re-verification of editions with a clause removed or the body changed (including the ground instance retry). The first proof of `lint` keeps the gate's `rlimit`.
2. **The probe share and the limit of reach points keep their values.** The probe share is 1/100 of the gate limit (50,000) in advisory queries too (it is not divided from the advisory limit), and the share of the range search is also 1/10 of the gate limit. Reach points have the fixed value 1,000,000 as before.
3. **An advisory query that reaches the limit is undecided** (the same as decision 3 of ADR-0126). Nothing is reported.
4. **`bmb examples` keeps using the gate's `rlimit`.** The limit is a property of the asker (`smt::Advisory`): `lint`, which reports refutations (`unsat`), is `Refutations`; `examples`, which shows models (`sat`), is `Models`, and `Models` uses only the advisory memory limit (ADR-0126).
5. **The format of the proof cache stays v5.** The key is the hash of the text the solver receives, and the `rlimit` is in that text (the header). The proof key of the incremental cache also holds the hash of the header. Gate proofs and advisory queries are separated by the text as well as by the memory limit (ADR-0129). `BMB_Z3_RLIMIT` lowers both limits together (the advisory limit does not exceed the gate limit).

## Rationale

| Option | Verdict |
| --- | --- |
| **A fixed `Z3_ADVISORY_RLIMIT` (accepted)** | It is a fixed value, so determinism holds (C2), and when the limit gets smaller a decision only turns into undecided: reports that require `unsat` can become fewer but no wrong one appears. It has the same shape as a mutation tool giving each mutant a small fixed budget, and sits in the same place as the memory of ADR-0126 |
| Per definition, k × the original `rlimit_max` | Two parameters, and queries without a first proof (the precondition check) have no base |
| As it is | Queries not used for reports take 89% of the time |

Value: 3.9 times the maximum advisory `unsat`, 2 times the maximum gate proof.

## Consequences

Before and after measurement (debug build, one `bmb lint` per file, child process CPU seconds).

| Corpus | Files | Report lines | CPU before → after | What changed in the summary |
| --- | --- | --- | --- | --- |
| `lib/std` | 11 | all 62 lines the same | 757 → 247 | `pre_rlimit` of `json` 41,054,388 → 9,054,388 |
| `tests/verify/pass` | 53 (52 with output) | all 167 lines the same | 307 → 75 | `pre_rlimit` of `proofs`·`recursive` (5,003,567 → 1,003,567, 10,003,859 → 2,003,859) |
| `port-shape` | 2 | all 40 lines the same | 108 → 84 | none |

- `pre_undecided`·`reach_undecided`·`det_undecided`, the number of mutants, the number of survivors and the number of re-verifications are the same in all 65 files with output (`pass/subslices.bmb` ended without output both before and after because of a defect in the mutation stage of `lint`. It was fixed separately). 62 files have byte-identical output, and the other 3 differ only in the `pre_rlimit` above (undecided checks that ran to the limit used less).
- **Why `bmb examples` does not use the advisory `rlimit` (decision 4).** Examples are models of `sat`, and some models only come out above 1,000,000, so the rationale "a smaller limit only reduces reports" does not reach them. When the edition that lowered it too was measured, on the reference and weak contracts of the 45 BVB tasks (96 definitions) the rows went 749 → 743: the 6 lost were all `is_non_prime` of `vb-mbpp-3-is-non-prime` (one `rejected` of the reference contract, all five `allowed` of the weak contract), and on the weak contract side no `allowed` row remained. CPU went 75 → 28 seconds. In the edition of decision 4, all 90 files are byte-identical to before and the frozen examples of `bench/intent` are unchanged.
- Tests: `compiler/tests/lint.rs` `advisory_queries_run_under_their_own_rlimit`, `a_query_at_the_advisory_rlimit_is_undecided_with_and_without_a_cache`, `examples_queries_keep_the_gates_rlimit`, `smt.rs` `advisory_queries_run_under_their_own_memory_limit`.
