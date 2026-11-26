# ADR-0152: A query the wall-clock safeguard ended spends no units

- Status: Accepted
- Date: 2026-11-26
- Related: ADR-0018 (a fresh state per obligation), ADR-0032 (the incremental cache), ADR-0129 (limits decided per query), DESIGN.md §6

## Context

- Every query runs under the pinned resource limit (`rlimit`), which decides the verdict on every machine, and under a wall-clock safeguard (Z3's soft timeout `-t`, 300 seconds, lowered by `BMB_Z3_BACKSTOP_MS`) for a query whose units tick slowly. A query the safeguard ends is undecided with its own reason, and the status line of its definition counts it (`"backstop":N`), as that verdict depends on the machine.
- After each answer the prover reads the units the query spent (`(get-info :rlimit)`) and adds them to the definition's `rlimit` and `rlimit_max`, to the costs of the satisfiability checks (`pre_rlimit`, `reach_rlimit`, `det_rlimit` of `lint`, the `rlimit` of `examples`), to the rows of `stability`, and to the proof cache entry of a proved obligation. For a query the safeguard ended, those units are how far the solver got when the clock ran out. Three runs of one compiler on one program, each with one such query, gave the sums 10118573, 10118422 and 10118784. With the safeguard lowered to 50 ms, ten runs of `pff::tri_le` of `tests/verify/fail/proofs.bmb` gave nine sums between 71637 and 185024.
- The output of `bmb verify` is meant to be the same bytes for the same input. A line that differs from run to run cannot be compared between the Rust compiler and its port, and a nightly case had to leave the program out.
- The proof cache stored a proof whose obligation had a query the safeguard ended (a probe, then a full query that proved). A hit gives that entry with no `backstop`, while a run without the cache on a faster machine asks the probe to its limit and counts it: the line then depends on which machine filled the cache. The incremental cache already stores no definition with such a query.

## Decision

1. **The units of a query the safeguard ended are 0.** From the `check-sat` the safeguard ended to the next `(reset)`, the solver connection reports 0 units without asking the solver (`Solver::rlimit_used` in `compiler/src/smt.rs`, `rlimit_used` in `selfhost/solver.bmb`). So such a query adds nothing to `rlimit`, `rlimit_max`, the costs of the satisfiability checks, `stability` or a cache entry. The count `backstop` says how many queries were ended this way.
2. **A proof with such a query is not stored in the proof cache.** The obligation is asked again by the next run, as the incremental cache already does for the whole definition.
3. Both compilers do the same, so the port stays byte-identical to the Rust compiler.

## Rationale

- The rule is one place, the reading of the units, because every count above goes through it. A count of 0 is what the deterministic part of the run knows of the query: that it was asked and not decided. The units a query spends within its limit are a property of the query and the pinned solver; the units at the moment a clock ran out are not.
- The rule runs to the next `(reset)`, not to the next `check-sat`, because Z3 counts units from the `(reset)`: a later `check-sat` of the same query would report the ended one's units with its own.
- Whether the safeguard ends a query at all still depends on the machine. That is the safeguard's purpose and the line says so; what this decision removes is a number that varies even when the same queries are ended. With it, `gen/g35.txt` of the port's tests (one query ended after 300 seconds) gives the same output in two runs of the optimized compiler.
- Alternative: keep the units and leave the programs that meet the safeguard out of comparisons. Rejected: it hides a nondeterministic output instead of removing it, and each new such program needs its own exception.
- Alternative: report the units under a separate key. Rejected: that key would still vary between runs.
