# ADR-0039: Continuous management system (multi-axis metrics, ratchet, self-hosting ladder, practicality rules)

- Status: Accepted (the maintainer's instruction 2026-10-25)
- Date: 2026-10-25
- Related: ADR-0034 (mission goals G1~G6), ADR-0037 (mandatory bootstrap), AUTONOMY.md, genesis §3.5 (claims only by measurement)
- Operating document: `dev-docs/HEALTH.md`

## Context

The maintainer's instruction (2026-10-25): manage completeness with metrics from several angles so that the language is verified systematically and reaches its goals, and set guidelines and criteria for continuously managing self-verification, practicality and self-containment (removing the Rust dependency).

Checking the current system against this, the following were missing.

| # | Gap | Evidence |
| --- | --- | --- |
| 1 | The gate sees only pass or fail. There is no **trend** of the metrics, so slow degradation is not caught | `scripts/check.sh` leaves no state file |
| 2 | There is no metric or per-stage target that measures **self-hosting (Rust and external dependencies)**. ADR-0037 sets only the end point (stage2=stage3) | Nothing counts external processes, dependencies of generated binaries, or the coupling of tests to the Rust API |
| 3 | There is no **practicality** metric. All 6 executable programs are small examples. Cases that cannot be expressed are scattered only in the friction table (m1.md §6) | `tests/run/` |
| 4 | A **proof cost regression** is caught by chance. The 1B-37 regression was caught by the BSB gate, and the corpus proof cost sum changed by only +1.0% (measurement below) | Experiment: reapplying the original 1B-37, V4 2,172,366 → 2,193,502 |
| 5 | **The prover's own detection power** (if the prover gets a defect, does the gate catch it) has never been measured | The fuzzers show 0 violations, but their ability to find a violation is not measured |
| 6 | The **trust base (TCB)** is not stated. The verdict depends on Z3 alone | `smt.rs` |
| 7 | By reading the gate result from its output, a failed commit was pushed **twice** (`8dcdb33`, `6947378`). There is only a rule (AUTONOMY §2) and no mechanical device | Backlog and commit records |
| 8 | **Document and test consistency**, such as the ADR index matching the files and every diagnostic code having a test, is not checked | `check_backlog.py` looks only at the backlog |
| 9 | The **review cadence** is not set. There is no regular snapshot other than the end-of-phase audit | AUTONOMY §3.2 |

## Decision

### 1. Five axes

| Axis | Question | Metric source |
| --- | --- | --- |
| M Mission effect | Is the north star achieved | ADR-0034 G1~G6 (benchmarks). This ADR does not change them |
| V Proof trust | Can "proved" be trusted, is the cost under control | `scripts/health.py` V1~V6, fuzzers, 1B-38 |
| S Self-hosting | Is the dependence on Rust and external tools shrinking | `scripts/health.py` S1~S6, the ladder of §3 |
| P Practicality | Can real programs be written, do errors give a way to fix them | `scripts/health.py` P1~P3, the rules of §4 |
| O Operations | Are records, documents and procedures consistent | `scripts/health.py` O1~O2, `check_backlog.py`, `scripts/ship.sh` |

### 2. Metrics and ratchet

- Metrics a machine can count are computed by `scripts/health.py` and checked by the gate on every commit. The baseline is `dev-docs/health/baseline.json`.
- There are four kinds of rule.
  - **Fixed:** must always hold. Cannot be changed even through the baseline (for example: 0 external crates, every diagnostic code tested).
  - **Ratchet:** does not get worse (for example: number of executable programs, share of errors with help attached).
  - **Budget:** can grow only up to a set ratio (proof cost sum +10%).
  - **Report:** only the value is recorded (the stage0 line count before porting starts).
- A change that exceeds a ratchet or budget enters the baseline only through `health.py accept --reason`, and the reason stays in the baseline history (subject to commit review). A change that increases external dependencies (S2) needs an ADR and the maintainer's approval besides the reason (the dependency row of AUTONOMY §4).
- When something improves, the gate reports it, and the next commit reflects it in the baseline (tightening the ratchet).

### 3. Self-hosting ladder

The way to the end point of ADR-0037 is divided into measurable rungs.

| Rung | Condition | Metric | When |
| --- | --- | --- | --- |
| L0 | 0 external crates, external processes {z3, clang}, generated binaries only libc | S1·S2·S3 fixed | Now ✅ |
| L1 | Integration tests run through the CLI, not the stage0 internal API. They can be used as is after porting | S5 ratchet (decrease only) | 4A-04 |
| L2 | stage0 feature freeze | S4 switches from report → budget (+2%, bug fixes only) | Start of 4D |
| L3 | The BMB compiler has stage2=stage3 | S6 increases, differential test | 4E |
| L4 | The Rust toolchain is needed only to reproduce the seed. The gate and CI run without cargo | 0 cargo uses per gate step | 4G |
| Kept | Z3, LLVM/clang, libc and Linux stay as the trust base. The Python of the benchmark harness is kept as a measurement tool outside the language (ADR-0139, 2026-11-14: a declared trust base, standard library only, version pinned) | S2·S3 fixed | — |

### 4. Practicality rules

1. **A feature is shown by programs.** A language sub-phase (2B~2G, 2S) closes only after adding 2 or more executable programs (`tests/run/`, with expected output) that use the feature. After 2S·4C these include real tools with input and output (line count, sort, JSON check and the like).
2. **Ledger of what cannot be expressed.** Anything that could not be written when tried is recorded with its handling in the friction table of `dev-docs/measurements/m1.md` §6. A sub-phase is not closed while an entry has an empty handling cell.
3. **Errors give a way to fix them.** The share of error diagnostics with a counterexample or fix suggestion attached (P3) is a ratchet. It is 0.46 now, and 2D-18 sets the improvement target.

### 5. Self-verification devices

- **Pushing is done only with `scripts/ship.sh`.** It runs the gate on the committed tree and pushes only when the exit code is 0. This removes the place where reading the output could intervene (gap 7).
- **The prover's detection power** is measured (1B-38). Deliberate defects are put into the prover and code generation, the share the gate catches is measured, and a test is added for each defect missed.
- **The trust base is stated** (HEALTH.md §3), and the single-solver risk is kept as a candidate for cross-checking with a second solver (1B-39, an external tool so approval is needed).
- **Proof cost is also measured under benchmark load** (INF-11). With the corpus alone, a regression like 1B-37 was not visible.

### 6. Cadence

| Cadence | What |
| --- | --- |
| Every commit | The gate (`check.sh`) + the `health.py` ratchet, push with `ship.sh` |
| Daily | Nightly long fuzzing (INF-07), performance regression (INF-12, planned) |
| End of sub-phase | The closing audit + a `health.py table` snapshot committed as `dev-docs/health/<phase>.md`. The five axes are reported in one table |
| After a real model measurement | Update axis M (G1~G6), and if a metric falls short of the criterion, a cause investigation task (AUTONOMY §3.2) |

## Rationale

- A gate that sees only pass or fail reports only the moment of falling below the criterion. A ratchet turns every change in the worsening direction into a record a person reads (the reason in the history).
- Self-hosting progress cannot be measured by the end point (fixed point) alone. Each rung of the ladder needs a metric to say "how far along" (genesis §3.5).
- The practicality rules close, at every sub-phase, the gap between "the feature exists" and "programs can be written with the feature". There are only 6 executable programs now, and self-hosting (phase 4) requires a program of tens of thousands of lines.
- The measurement of gap 4 shows that the metric design can be wrong. So the metrics themselves are also tested with real regressions (1B-38, INF-11), and a metric that fails to catch one is fixed.

## Consequences

- `health` and `health rules` jobs are added to the gate (about 9 seconds, in parallel).
- A baseline file comes into being, and an intended degradation must be committed with a reason.
- Reverting needs an ADR that supersedes this one. Adding metrics and adjusting thresholds are handled autonomously as implementation decisions within the scope of this ADR and reported. But relaxing a fixed rule needs approval.
