# ADR-0041: Trade-off ledger (costs accepted, axes maximized, rules)

- Status: Accepted (the maintainer's instruction 2026-10-25)
- Date: 2026-10-25
- Related: DESIGN.md §2 (goals)·§4.1 (ledger of conveniences given up), ADR-0034 (G1~G7), ADR-0039 (continuous management), ADR-0040 (premise revision)

## Context

The maintainer's instruction (2026-10-25):

> Reducing with a cache and making tests unnecessary by the structure of the language are stories of different dimensions.

The trade-offs were instructed as follows.

- **Accepted:** the language is hard to learn and write. The first build takes long (there is no map). Builds take some time.
- **Rule:** reading and writing go through tools. Editing code files directly is not recommended.
- **Maximized:** stability and performance highest, test code lowest, AI coding efficiency highest (token efficiency, navigation, writing, from several angles), binary size lowest, code reuse highest, and others.

DESIGN.md §4.1 records what each individual syntax convenience is traded for. But nowhere was it written in one table what the language as a whole gives away and what it pushes to the limit. ADR-0040 set that repeated run cost is claimed only "against the selection and caching baseline". But that wording put running tests less (run optimization) and not needing tests (structural removal) on the same axis.

## Decision

### 1. Two dimensions of test cost

| Dimension | What it reduces | Who | BMB's claim |
| --- | --- | --- | --- |
| **Run optimization** | Existing tests are run less (selection, caching, scheduling) | Build system, CI | Not claimed. The industry already does it |
| **Structural removal** | The tests need not exist. Writing, maintenance, running and oracle judgement all disappear | The language (proof gate, total functions, effects, contract firewall) | **BMB's original claim** |

- A cached test must still be written and maintained. It runs again when a dependency changes. The strength of its verdict also stays at the inputs the test covers.
- For a structurally removed test, all four of these costs are 0. A proof covers every input.
- So the main metric is **not the number of runs but the amount of test code** (G8).
- Accumulated run cost (G7·BCB) is an auxiliary metric. Per the operating rule (a fair baseline), the selection and caching baseline stays in BCB. But that comparison looks at whether "structural removal + running the remaining tests" is cheaper than "optimized running of the written tests". The industry recovery rate (40~67%) is not an upper bound on BMB's claim.

### 2. The ledger

**Axes maximized.** The target level and the metric are given together. An axis without a metric is not claimed.

| Axis | Level | Metric | Current |
| --- | --- | --- | --- |
| Stability (correctness) | Highest | G1 0 soundness violations, proof gate, V3 | 0 violations ✅ |
| Performance | Highest | G6 time against safe Rust, proof-based check removal | ±5% ✅ |
| Test code | Lowest | **G8**: lines of test code needed to pass the same hidden judgement. BMB (the remaining test area derived by the compiler) versus Rust (BRB agent tests) | Not measured (2H-06) |
| AI coding efficiency | Highest | Token efficiency (G5 total tokens, G7), navigation (G4), one-shot writing success (G2), spec strength (G3) | Waiting for real model measurement |
| Binary size | Lowest | **G9**: stripped size of the same program. Baseline: safe Rust (`-O`, `panic=abort`, strip). Ratchet `health.py` B1 | 7 benchmarks, BMB about 14.5KB versus Rust about 342KB (2026-10-25) |
| Code reuse | Highest | H-REUSE (duplicate and scope overrun rate), the `duplicates` query, merging isomorphic functions (3D) | Not measured (3A-07) |
| Determinism and reproducibility | Highest | V6 same input same output, reproducible build (5C) | verify determinism ✅ |
| Change safety | Highest | Contract firewall invalidation scope, `impact`, H-MAINT | Not measured (1B-41) |
| Runtime dependencies | Lowest | S3 generated binaries only libc | ✅ |
| A way to fix (diagnostics) | Highest | P3 share with a counterexample or fix suggestion | 0.46 |

**Costs accepted.** They are not optimized to be reduced. But they are observed, and if they encroach on a maximized axis, a cause investigation task is created.

| Cost | Why accepted | Observation |
| --- | --- | --- |
| Hard to learn and write (a surface not trained on, verbose explicitness) | Paid once at writing time to remove repeated verification and navigation (E3). The labor of explicitness is taken by tools (`fix`·`uses`) | Write tokens. But **total tokens** belong to a maximized axis (AI efficiency). Even if the write cost grows, the sum of writing + verification + navigation + modification must shrink (G5) |
| The first build takes long (map, full proof) | The map and the proof cache make every later query and build proportional to scope | First build time per definition (INF-13) |
| Builds take somewhat long (proof) | Proof replaces tests. Only changed definitions are proved again | V4 proof cost budget (+10%), incremental build time (INF-13) |
| Limited expressiveness (a feature that cannot be proved is not added, escape hatches require a rationale) | A feature that is not verified is not put in the language (operating rule) | Escape hatch area (2E) |
| External toolchain dependence (Z3, LLVM) | Stated as the trust base (HEALTH.md §3) | S2 fixed |
| No ecosystem at the start | The ecosystem comes after principles 4·6 are demonstrated (5E) | — |

**Rules.** These are not trade-offs but kept.

| Rule | Rationale |
| --- | --- |
| Reading and writing go through tools. Editing code files directly is not recommended | Principle 6. The tools keep the normal form, hashes, map and proof cache consistent |
| One expression per job, one normal form | Principle 8 |
| Escape hatches require a rationale declaration and are counted as unproven area | §4.1 |

### 3. Tensions in the ledger

- **"Hard to write (accepted)" versus "AI efficiency highest":** the unit of judgement is not one act of writing but the total tokens of the whole task. If total tokens exceed Rust+tests (G5 refuted), it is seen as beyond the accepted range and the surface and tools are fixed.
- **"Slow build (accepted)" versus "tests lowest":** the unit of comparison is not build time but the accumulation of build + tests (G7).
- **"Binary lowest" versus "performance highest":** size growth for performance, such as inlining, is allowed. The size ratchet (B1) prevents growth without a feature.

## Consequences

- A summary of the ledger goes into the goals of DESIGN.md §2.
- G8 (test code)·G9 (binary size) are added to the ROADMAP mission goals (provisional).
- B1 (binary size, budget +5%) goes into `health.py`.
- 2H-06 (G8 measurement), 2D-21 (remaining test area query) and INF-13 (build time observation) are added to the backlog.
- The repeated run cost wording of ADR-0040 follows the two-dimension distinction of §1.
- Reverting needs an ADR that supersedes this one (the maintainer's approval).
