# ADR-0042: Design and pre-registration of BCB (accumulated verification cost benchmark)

- Status: Accepted (autonomous). "BCB measures the current behavior as is" in the consequences is superseded by ADR-0043: the B arms are measured with the compiler of ADR-0043
- Date: 2026-10-25
- Related: ADR-0040 (H-AMORT, G7), ADR-0041 §1 (auxiliary metric), ADR-0032·0033 (incremental key), ADR-0035 (BSB), backlog 2A-14, `dev-docs/research/테스트 실행 비용 누적 근거/report.md` recommendation 2

## Context

ADR-0040 decided to measure H-AMORT with BCB, and recommendation 2 of the research report proposed five arms and four refutation conditions. To implement it, what the report left open must be decided: the kinds and representation of edits, how to tie the two languages to the same edit, the selection rules of R1·R2, BMB's remaining tests, how to measure toolchain events, the handling of arms that are too expensive to measure, and the numeric judgement of the refutation conditions. They must be decided and committed before measurement so that there is no after-the-fact adjustment (operating rule).

## Decision

### 1. Corpus and history

- The codebase is made with the default parameters of `bench/m3/gen.py`. The sizes are N = 1,000 and 10,000, and the depth axis is `--depth 2`·`6` at N = 1,000. The Rust twin is `bench/bnb/twin.py`.
- The history is made by `bench/bcb/history.py` with a fixed seed. A commit edits the plan (`gen.Unit`·`gen.choose`), and the two renderers use the same plan, so the two languages receive the same edit. One edit changes one unit file.
- Edit kinds: `body` (interface kept: constant, template, callee, result shape), `contract` (adding or removing an `_ok` predicate in a leaf's `ensures`), `spec` (changing the upper bound of an `_ok` predicate), `add` (adding a leaf), `remove` (deleting a leaf with no callers). At every commit the BMB workspace is fully proved and the Rust compiles without warnings (gate `bench/bcb/test_bcb.py`).
- Three profiles: `body-heavy` (70/10/5/10/5%), `contract-heavy` (40/30/10/15/5%), `body-only` (for refutation condition 2). The number of edits per commit is 1·2·3 at 60·25·15%, and units are chosen uniformly. The ratios were not estimated from real repository histories. This is because there is no real history written in BMB. This point is reported as a bias.
- The history file holds the result values of the edits and a digest of the whole tree per commit. A replay must rebuild every digest, and fails if one differs. K = 1,000.

### 2. Arms

The five arms are run in order (no concurrent runs) on the same machine. The cost is the CPU time of child processes per commit (user+sys, `wait4`), and wall clock is recorded separately.

| Arm | Per commit | Selection rule |
| --- | --- | --- |
| R0 | `cargo build` + `cargo test` (all) | None. Cargo default dev profile (incremental compilation on) |
| R1 | The same build + selected tests | The tests of the changed units and of the units that transitively depend on them in the unit dependency graph. This is an exact cache that treats one unit as one Bazel target |
| R2 | The same build + selected tests | The tests of the functions whose static call closure reaches a changed function (including predicates). Test inputs do not use empty arrays, so in generated code the static closure equals dynamic coverage |
| B0 | `bmb verify` + `bmb build` + remaining tests with a new cache | All remaining tests |
| B1 | The same with a persistent `--cache` | The R2 rule on the remaining tests |

- The Rust twin is one crate. With a crate per unit, R1 would have to recompile reverse-dependent crates, which puts the Rust side at a disadvantage. One crate and rustc incremental compilation is a choice that favors Rust.
- **BMB's remaining tests** are the output of the io definitions (`show`). No contract speaks about the output (the area that needs testing of ADR-0030). The test builds and runs a `main` unit that calls `show` with fixed inputs, and compares with the expected output from evaluating the plan. When 2D-21 (remaining test area query) exists, it is replaced by that output.
- **Sampling of the expensive reference arms.** R0·B0 are reference arms not used in the claim (the report's "caution in interpretation"). At N = 1,000 every commit is measured. At N = 10,000, R0 is measured every 10th commit, and B0 at only 5 commits (0, K/4, K/2, 3K/4, K). The accumulated value is estimated as sample mean × K and marked "estimated". The cold proof of B0 is about 1,700 seconds per commit at N = 10,000 (3A-01).

### 3. Toolchain events

The event commits are K·j/5 (j = 1..4), and the 1-event variant uses only j = 2. At an event commit, first measure as usual, then clear the cache (BMB: the cache directory, the same effect as the compiler identity or the Z3 version changing; Rust: `cargo clean`, the same effect as a rustc upgrade) and measure the same commit again. The state after a cold rebuild equals the warm state (the proof cache is content-addressed and incremental compilation produces the same artifacts). So a single run yields the accumulated values for 0, 1 and 4 events. C_e(K) = C_0(K) + Σ(cold cost at the event commit − usual cost).

### 4. Test equivalence and defects

- There is one Rust test per executable definition. It calls the function with inputs that satisfy `requires`, and judges by the `debug_assert!` (contract) in the twin body and by overflow and bounds panics. A mutant is a W0904 operator (2D-09·2D-16) applied at the same body position in both languages. Inputs are extended from a deterministic candidate sequence (boundary values, fixed-seed random) until they kill every mutant that the BMB contract of that definition kills. Both kill rates are reported per definition.
- Defects: for each commit, a side tree is made with one mutant planted in one edited executable definition. The mutant is chosen deterministically and surviving mutants (ones the contract does not distinguish) are included. Each arm runs on the side tree by the selection rule of that commit. If the arm fails (proof failure, remaining test failure, selected Rust test failure), it caught the defect. The side tree does not enter the history.

### 5. Refutation conditions (pre-registered)

If even one of the following holds, H-AMORT is recorded as refuted in that scope. The result is reported from one run, without retry.

1. At N = 10,000, K = 1,000, 0 events, C_B1(K) ≥ C_R1(K). `body-heavy` and `contract-heavy` are judged separately.
2. In `body-only`, the mean number of definitions B1 proves again per commit is at or above the mean number of distinct definitions executed by the tests R2 runs again (no contract firewall advantage). N = 1,000 and 10,000 are judged separately.
3. At 4 events (Rust also does `cargo clean` at the same commits), C_B1(K) > C_R1(K). Judged per profile.
4. In the defect detection rate over all profiles combined, B1 < R2.

The break-even number of commits k* needs the write cost W (converted to CPU). W comes from the BRB measurement (the user's run), so BCB reports k*(W) = min{k : C_B1(k) + W ≤ C_R1(k)} as a function of W, and the figure is filled in after the BRB results are committed.

## Rationale

- Recording edits at the plan level makes the edits of the two languages the same by construction. It is easier to check than moving text diffs between languages (the same approach as BNB).
- Measuring an event twice within the same run removes the need to run three variants with the same event positions three times, and the noise difference between events is not mixed in either.
- Sampling of the reference arms is used only for the arms not claimed. B1·R1·R2, which are used in the judgement, measure every commit.
- Alternative: Rust with a crate per unit. Closer to Bazel, but the compile cost of the Rust side grows. Following the fair baseline principle, the side that favors Rust was chosen.

## Consequences

- Measurement cost: at N = 10,000, three judged arms × 1,000 commits × 3 profiles. One run is several hours. The gate checks only the harness logic with N = 100 and a small K.
- Generated code is easier to prove and lighter to test than real code (the report's "caution in interpretation"). The result is claimed only within the scope of this corpus.
- What the first implementation revealed: the `iface` hash contains `#[uses]` (ADR-0019), so the incremental key (ADR-0032) proves callers again on a change of the callee's `#[uses]` too. It is sound, but the firewall is that much wider. In the N = 100 history, 35~43% of body edits changed `#[uses]`. BCB measures the current behavior as is, and the improvement is left to 2D-22 (a change to an accepted ADR, so approval).
