# ADR-0123: The memory limit of the solver

- Status: Accepted (autonomous, AUTONOMY §4 "decision records": `T2-16` of the thirteenth execution plan, approved by the maintainer 2026-11-11 "recommendation accepted, autonomous execution/verification approved"). It adds one to the pinned options of ADR-0017
- Date: 2026-11-11
- Related: ADR-0017 (integer semantics and SMT encoding: pinned options), ADR-0018 (a fresh state per obligation), DESIGN §6 (unknown, timeout and a missing solver are failure), `scripts/toolchain.env`, backlog T2-16

## Context

- On one false obligation, Z3 produced neither a counterexample nor a limit hit, used memory up to 12.5GB and was then killed by the system (`tree::call` of the `port-shape` experiment: the `push` length obligation when a nested quantifier inside a recursive predicate is in the context).
- The gate then emitted E0812 ("solver error") and counted the 20 later obligations of that definition as failures without solving them. It took 56 seconds, and other work on the same machine could have died with it. `bmb lint` went through the same for every mutant and did not finish.
- `rlimit` (ADR-0017) counts the work of the solver but does not count memory.

## Decision

1. **A memory limit is put on the solver process.** The value is `Z3_MEMORY_MB` (1,024) in `scripts/toolchain.env`, and it is passed as Z3's `-memory`. `BMB_Z3_MEMORY_MB` only lowers it (the same rule as `Z3_RLIMIT`).
2. **A query that hits the limit is undecided.** E0811 carries the reason `memory limit reached, 1024 MB`. It is a failure of the same class as hitting `rlimit`.
3. **Only that query is undecided.** Hitting the limit ends the process, so the next obligation is solved by a new process. The whole base is sent again for every obligation (ADR-0018), so the answer is the same as when one process solves them in sequence.

## Rationale

| Option | Verdict |
| --- | --- |
| **A fixed memory limit, undecided when hit (accepted)** | It is the standard practice for resource limits, and the failure is on the closed side (DESIGN §6). Z3's memory accounting is the same for the same version and the same query, so the verdict does not depend on the free memory of the machine |
| As now (the system kills it) | The verdict depends on the machine, and the obligations after the death are counted as failures without being solved |
| An address space limit on the process (`ulimit -v`) | How Z3 ends when it is hit is not defined (it died in the experiment as well), and the reason cannot be told apart |

Grounds for the value: the 11 units of the standard library (218 definitions) are all proved with the same `rlimit` sum (20,192,306) even at a 256MB limit. 1,024MB is four times that, and the reproducing query ends in about 4 seconds at this limit (18 seconds at 4,096MB).

## Consequences

- The reproducing query: failed obligations 20 → 1 (E0811, reason memory), 56 seconds → 16 seconds. `bmb lint` finishes on the same file (328 seconds, before it did not finish). Test `compiler/tests/gate.rs` `a_query_at_the_memory_limit_is_undecided_and_the_rest_go_on`.
- That `lint` takes 328 seconds for 340 lines remains: for every mutant there is a query that goes up to the limit. The budget of mutation queries is the remaining work of `T2-16`.
- If a normal proof comes to hit the limit, the gate shows it as E0811. Then the query is fixed and the limit is not raised (the same as `Z3_RLIMIT`).
