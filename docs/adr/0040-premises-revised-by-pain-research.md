# ADR-0040: Premises revised by the pain point research, and new hypotheses

- Status: Accepted (approved by the maintainer 2026-10-25, "agree with all")
- Date: 2026-10-25
- Related: DESIGN.md §2·§3 principle 4, ADR-0034 (mission goals), ADR-0039 (continuous management)
- Source material: `dev-docs/research/BMB 페인포인트 가설 검증/report.md`, `dev-docs/research/테스트 실행 비용 누적 근거/report.md` and each `notes/`

## Context

BMB's premises were held up against external evidence in two rounds of web research. Caution: many primary sources did not open in this environment, so many figures came from search summaries. Each report carries a "source not checked" mark. This ADR decides only the direction, and moving a figure into a document as a claim goes through a check against the source (genesis §3.5).

Conclusions of the first research (pain point hypothesis verification):

- The direction of the four hypotheses (tests, scale, AI limits, collaboration) is supported. But the evidence differs from the reasons written in the documents.
- The strongest evidence is "passing tests does not guarantee a correct answer" (oracle insufficiency). In UTBoost (ACL 2025), 15.7% of the patches that passed SWE-bench Verified were wrong.
- Humans write less code but do not read less. Review is the new bottleneck.
- The next bottleneck is spec validity (N1).
- Seven pains that were not in the hypotheses appeared (N1~N7).

Conclusions of the second research (accumulation of test run cost, a pain the maintainer experiences directly):

- **The accumulation of repeated run cost is documented at large scale.**
  - Google TAP: the number of changes and the number of tests grew together, so the computation of "every test on every change" grew quadratically.
  - About 1% of test runs found a failure.
  - Meta, Uber and Shopify cut test infrastructure cost by around half through test selection and queue improvements.
- **In the AI era, PRs, pushes and CI runs increased greatly.** GitHub Actions minutes grew several times over (2023→2026), and agent PRs also surged. Stripe's agent pipeline limits CI rounds to 2 per job.
  - Main branch throughput and deployments hardly grew (CircleCI 2026). The repetition happens "per PR, push and retry" rather than "per deployment".
- **There is counter-evidence too.**
  - The CI of an ordinary team is short (median about 3 minutes).
  - Test selection and caching already recover a large share of the waste.
  - Proofs also have a recheck on every build and a maintenance cost (solver upgrades, proof repair).
  - There is no model in the literature that quantitatively compares "verify once versus test repeatedly".
- **There is a field precedent.** In aviation (DO-333, Airbus), proof replaced unit tests. Integration tests and tests on target hardware remain.

## Decision

### 1. Premise revision (DESIGN.md §2)

- **E1.** Line-by-line reading cannot keep up with the output of AI. What humans review moves from code to intent and specs.
- **E2.** The bottleneck of AI coding is not generation but verification and navigation. There are four bottlenecks on the verification side.
  - (a) Oracle insufficiency: passing tests does not guarantee a correct answer
  - (b) Review cost that cannot keep up with the output
  - (c) Test cost that is run again on every change and accumulates as number of changes × number of tests. It grows as AI raises the frequency of PRs and pushes
  - (d) Whether the spec matches the intent (spec validity)

  The bottleneck on the navigation side is the cost of finding where to modify and what is affected in a large codebase.
- **E3.** The trade of paying a cost once at writing time to reduce the verification cost repeated on every change and the defects that escape is a gain.
  - The cost paid once includes writing specs and proofs and the cost of a surface not trained on.
  - Profit and loss are measured by total tokens, accumulated run cost and defect rate.

### 2. Revision of principle 4 (DESIGN.md §3)

**Proof replaces repeated testing.**

- Without a proof there is no binary.
- A proof is paid for once, and only the changed definition and the definitions that rely on its contract are proved again (proof cache, incremental build).
- The area that needs testing is derived by the compiler, reported as a metric, and minimized.
- The scope of replacement is the functional correctness of a single program (unit level). Tests of integration, of trust boundaries (external input, the solver and the compiler themselves), and of configuration outside the code (settings, deployment) remain.
- "Zero tests" is a direction, not a claim.

### 3. Out of scope, stated (DESIGN.md non-goals)

- Failures outside the code (settings, deployment, retry storms, N5) are out of scope.
- The security of agent tools (prompt injection, supply chain, N7) is also outside what the language guarantees. BMB's own tool surface (the MCP server) has a threat model (2D-20).

### 4. New hypotheses and refutation conditions

| ID | Hypothesis | Refutation condition | Owner |
| --- | --- | --- | --- |
| H-SPEC | With the BMB tools (counterexamples, spec strength, lints), the mutant kill rate of the agent's specs is higher than with the default Verus toolchain | On the same model and tasks, BSB kill rate difference ≤ 0 | BSB, BVB (spec strength column) |
| H-ORACLE | Passing the proof misses fewer hidden defects than passing tests | In BRB, the share of "gate passed & hidden failure" ≥ the share of "tests passed & hidden failure" | BRB |
| H-GAME | With escape hatches banned and spec changes audited, gate circumvention is 0 | On tasks where the spec and the hidden reference contradict each other, gate pass rate > 0 | BMB-Impossible (new) |
| H-STABLE | Proofs are stable under meaning-preserving transformations and seeds | Share of unstable queries ≥ 2.6% (the Mariposa baseline) | Stability fuzzer (new, G1) |
| H-MAINT | Re-proving per change is localized within the impact set | A re-proved definition leaks outside the `impact` set | Edit replay test (new) |
| H-FAMILIAR | The "not trained on" cost of the BMB surface is smaller than the dual representation cost of Verus | In BVB, BMB one-shot success rate ≤ Verus | BVB |
| H-SCALE | In BMB the success rate and tokens are independent of codebase size, and file-based they are not | The slope over the number of definitions (1k→10k) is the same as the Rust twin's | BNB (size axis) |
| H-REUSE | The deterministic map reduces duplicate creation and scope overrun | Share of duplicate definitions and definitions beyond what was asked ≥ the Rust twin's | BNB (new metric) |
| H-AMORT | Over a change history, BMB's accumulated verification cost (incremental re-proving + remaining tests) is smaller than the Rust test run cost at the same defect detection power | In a commit history replay, accumulated CPU time is at or above the Rust twin with test selection and caching | BCB (new) |

### 5. Mission goal G7 added (extension of ADR-0034)

| Goal | Metric | M2 | M3 |
| --- | --- | --- | --- |
| G7 Accumulated verification cost | Accumulated CPU time and defect detection of a commit history replay (BCB) | Measurement starts, the ratio against the Rust twin (full run) is reported | At or below the Rust twin with test selection and caching, defect detection equal or better |

The target values are provisional. They are fixed once by the same rule as ADR-0034.

## Rationale

- The "piling up tests" of E2 is backed by the maintainer's experience and the records of large companies (quadratic growth, about 1% failure detection). But it is small for a team of ordinary size, and selection and caching recover part of it. So profit and loss are measured directly in BMB against a fair baseline (Rust with selection and caching) (H-AMORT, G7).
- Oracle insufficiency is a gain that is independent of run cost and cannot be had by running. The two grounds are kept together. In short, tests buy a weak verdict again on every run, and proofs buy a stronger verdict again only when there is a change.
- The differentiator unique to BMB is the contract firewall (ADR-0032). If only the body changes, callers are not verified again, so the invalidation scope is narrower than test impact analysis (all dependents). The external literature has no measurement of this, so BCB measures it.
- The industry already recovers about 40~67% of the waste of repeated runs through test selection, caching and scheduling (Meta, Uber, Google, source not checked). That is run optimization, running existing tests less. BMB's claim is structural removal, making it unnecessary for the tests to exist, which is a different dimension (ADR-0041 §1). For the accumulated run cost comparison (BCB), selection and caching stay as the fair baseline, but the main metric is the amount of test code (G8).
- Proofs also have a repeated cost. When the compiler identity or the Z3 version changes, the whole cache is invalid, and proof maintenance (solver instability, proof repair) also costs. BCB puts these events in the history and measures them.
- Spec validity (N1) is BMB's largest internal risk. The priority of BSB is raised and it is measured together with gaming (H-GAME).

## Consequences

- DESIGN.md §2·§3·non-goals, the ROADMAP mission goals (G7) and phase table, and the backlog (2A-11~2A-15, 1B-40, 1B-41, 2D-19, 2D-20, 2H-04, 2H-05, 3A-06~3A-08) change.
- Reverting needs an ADR that supersedes this one (the maintainer's approval).
