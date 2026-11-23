# ADR-0067: A proof obligation first solves a probe that writes every leaf definition as a macro, with 1% of the limit

- Status: Accepted (autonomous, AUTONOMY.md §4). Decides the work ADR-0064 left as backlog T2-05. The decision of ADR-0064 (the form of the full query) stays, and one attempt is added in front of it. The meaning is unchanged and only the order of asking the solver changes · decision 4 (vacuity check and precondition satisfiability check) is supplemented by ADR-0069: the two checks also solve the probe first · the order of decisions 2·3 is supplemented by ADR-0089: if the probe does not decide, the ground instance retry comes first, on the probe share
- Date: 2026-10-30
- Related: ADR-0017 §4 (definition axiom), ADR-0023 (ground instance retry), ADR-0048 §4 (bit operation abstraction attempt), ADR-0064 (macros for leaves that bind an array), backlog T2-05

## Context

- ADR-0064 made only leaf `spec fn`s that bind an array into macros. Making scalar leaves macros too gave failing corpus rlimit −84%, but one bench proof hit the limit (`gcd_exec` of `vc-va0053-aspect-pairs`: 226,723 → limit, retry 53,006).
- Of the failing corpus rlimit 126,061,811, about 120 million is the share of 25 definitions that have an attempt that hit the limit of 5,000,000. Most are counterexample queries with a quantified definition of a scalar predicate. The smallest example is `contracts::wrong_post` (one definition, two assertions). The verdict was given by the ground instance retry. One counterexample throws away one full limit.

## Decision

1. **Probe.** For every proof obligation, a query in which the definitions of all non-recursive leaf `spec fn`s are written as macros (the probe) is solved first with rlimit `Z3_RLIMIT / 100` (now 50,000). The macro form and the pattern rule are the same as ADR-0064 decisions 1~4 (quantified axiom if a pattern contains the symbol).
2. **Use of the answer.** If the probe is `unsat` it is a proof, and if `sat` its model is the counterexample (same meaning, ADR-0064 decision 3). Otherwise the current full query (the form of ADR-0064, full limit) is solved, and after that it is the same as now (ground instance retry). If the probe is the same text as the full query (no scalar leaves) there is no probe.
3. **Order.** For an obligation with bit operations the order is abstraction attempt (ADR-0048 §4), probe, exact query. In the `--emit-smt` script the probe comes directly before its full query as `; probe of obligation k`. The `rlimit` of the status line is the sum of all attempts, and `rlimit_max` is the maximum over the attempts excluding retries (abstraction attempt, probe, full query).
4. The vacuity check and the precondition satisfiability check (`bmb lint`) keep the current form (backlog T2-06).

## Rationale

- **Cause and layer:** the cost of counterexample queries is that Z3 cannot build a model over quantified definitions (the same as ADR-0064. For scalars it runs until `canceled`). The macro form removes this cost. But a macro expands every application in preprocessing, and a quantified definition adds only the instances needed during search. For hard proofs, which is better depends on the query. The layer is the encoder (how to ask the solver).
- **Canonical way:** trying the same goal with several encodings and settings, shortest share first, is an established method of proof tools (the slices of Isabelle Sledgehammer, short limits first in Why3 strategies, the stepwise proof levels of SPARK). `define-fun` for non-recursive definitions is the canonical form of SMT-LIB (ADR-0064).
- **Candidates (seed 0, Z3 4.15.4, rlimit 5,000,000, the same measurement as T2-04: the failing corpus is the sum of `rlimit` in the status lines of `tests/verify/fail`, V4·V8·V9 are the functions of `scripts/health.py`, the libraries are the sum of the standalone proofs of each of `lib/*.bmb`):**

  | Candidate | Failing corpus | V4 | V8 | V9 | `gcd_exec` max | Libraries |
  | --- | --- | --- | --- | --- | --- | --- |
  | Current (ADR-0064) | 126,061,811 | 7,879,083 | 26,900,517 | 0 | 226,723 | 8,871,954 |
  | All leaves as macros | 20,609,594 | 7,139,615 | 30,498,652 | 1 | Limit (retry 53,006) | `base64::byte0` proof fails |
  | Quantified first, macro again on `unknown` | 136,063,526 | 7,879,083 | 26,900,517 | 0 | 226,723 | Same as current |
  | Macro first (full limit), quantified on `unknown` | 25,860,213 | 7,139,615 | 30,672,369 | 1 | 226,723 after the limit | — |
  | Choose the form by the arithmetic of the query (quantified if non-linear or bit conversion) | 20,901,679 | 7,338,757 | 25,136,198 | 0 | 226,723 | 18,869,898 |
  | Probe 1% (this decision) | 15,960,213 | 7,387,374 | 24,992,756 | 0 | 226,723 (plus probe 50,000) | 10,673,150 |

  - The "quantified first" option uses the full limit on the first attempt, so the counterexample cost does not go down. Searching for a counterexample separately after the proof is a gain only when the first attempt ends cheaply.
  - "All leaves as macros" and "macro first" make a non-linear proof hit the limit (V9). `base64::byte0` of the libraries (bit conversion) hits the limit in the macro form for 7 of 8 seeds.
  - "Choose by arithmetic" first chose only non-linear and lost `base64::byte0`, then regained it by adding bit conversion. Even so `lib/base64.bmb` is 3.8 times (3,651,389 → 13,796,112). A rule that chooses the form by syntax had to be fixed for every new case.
- **Evidence that the form depends on the query (seeds 0~7, first-attempt queries where the two forms differ, change in the rlimit sum of the macro form and the count of undecidable):** on the corpus, the seeds and 8 BVB reference solutions: 969 linear −15% (0 → 0), 178 non-linear +150% (1 → 8), 25 bit conversion −40% (56 → 32). On the libraries and game: 343 linear +158% (0 → 0), 14 bit conversion +27% (64 → 76), 6 non-linear −37%. Neither form is always better, for any syntactic class. The probe answers in the macro form only the queries that form finishes cheaply, and leaves the rest to the current form.
- **What the probe does not lose:** if the probe does not decide, the full query is the same text and the same limit as now. Everything proved now is proved, and per obligation the cost grows by at most the probe share (50,000).
- **Sensitivity to the share (failing corpus / V4 / V8 / libraries):** 20,000 gives 15,900,213 / 7,526,897 / 24,749,194 / 10,498,991, 200,000 gives 16,260,213 / 7,319,763 / 24,970,784 / 13,980,576, 1,000,000 gives 17,860,213 / 7,139,615 / 26,341,170 / 22,225,372. 1% is in the middle of the flat region. A large share throws away that much on every query the probe does not win.
- **The 226,723 of `gcd_exec` is luck.** The quantified form of the same query is 101,992~2,039,131 on six of seeds 1~7 and the limit on seed 6. The probe does not change this query.
- **Failing corpus:** diagnostic codes, positions and definition status are unchanged. Counterexample values in 21 places changed to another real model. Definitions with an attempt that hit the limit go 25 → 3 (`nl::f` is non-linear and undecidable in both forms, `pff::tri_le`·`geo::off_diag` hit the limit in both forms).

## Consequences

- The query text is unchanged and the probe is added. The key (full query text) and format of the proof cache are unchanged.
- Solver stability (V7, `scripts/solver_stability.py`) measures the full query (0 → 0). The stability of the probe is not measured. Measuring the verdict of obligations made of several attempts is backlog T2-07, together with the abstraction attempt of bit operations.
- Revert: if the `PROBE_QUANTIFIED` marker is not put on scalar leaves in `Prog::axiom`, no probe is made. The storage format and the language do not change.
