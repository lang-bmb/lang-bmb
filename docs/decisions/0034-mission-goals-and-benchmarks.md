# ADR-0034: Mission goals and the benchmark system

- Status: Accepted (approved by the maintainer 2026-10-24, the recommendation). The target values are fixed by ADR-0115 (2026-11-11)
- Date: 2026-10-24
- Related: DESIGN.md §2 (premises, goals)·§3 (principles)·§9 R4, ADR-0021 (measurement procedure), ADR-0025 (`m1b`), ROADMAP 2A·2H·3E

## Context
Measurement is currently closed inside BMB (the m1·m1b tasks, the Rust performance baseline, the fuzzers). To judge "has BMB achieved its mission", two more things are needed.

1. **External position:** it must be possible to compare against how well the same model does in other verified languages (Verus, Dafny, Lean) on public benchmarks.
2. **Premise verification:** there is no comparison experiment that directly measures premises E2·E3 of DESIGN.md ("the bottleneck is verification and navigation", "the trade of raising the cost of writing to lower the cost of verification and navigation is a gain"). Current measurement only shows that BMB "works", not that it is "better".

## Survey: public benchmarks (as of 2026-10; the figures are the values reported by each paper and cannot be compared directly with each other)

### Verified code generation (the closest comparison targets)
| Benchmark | Language | Size | Task | Reported level |
| --- | --- | --- | --- | --- |
| Verus-Bench (AutoVerus, OOPSLA 2025) | Verus/Rust | 150 | Write a proof for given code and spec | Agent system 91% (137/150), GPT-4o direct 45%. In the later KVerus reproduction, AutoVerus 87/150 |
| Vericoding (2025-09) | Dafny·Verus·Lean | 12,504 specs (Verus 2,334) | Formal spec → code + proof | Off-the-shelf LLMs: Dafny 82%, Verus 44%, Lean 27% |
| VeriContest (NeurIPS 2026) | Verus/Rust | 946 (later 1,007) | Natural language → code, spec, proof, with positive and negative tests | Best model: code 92%, spec 48%, proof 14%, end to end 5.3% |
| VERINA (2025) | Lean 4 | 189 | Generate code, spec, proof | o4-mini: code 61%, spec 51%, proof 3.6% |
| DafnyBench / Clover | Dafny | Hundreds / tens | Restore annotations (invariants etc.) / three-way consistency | For reference |

**Observation:** in public results the bottleneck is not the code but **the spec and the proof** (VeriContest: code 92% versus end to end 5%). This is exactly where BMB's differentiators (counterexamples, hints, `#[uses]` auto-fill, lints, impact analysis) aim. Also, Verus has the same surface as BMB, so **tasks can be ported and compared directly with the same model**.

### Spec quality
| Benchmark | Measurement |
| --- | --- |
| nl2postcond (FSE 2024) | Correctness and discriminating power of postconditions (do they reject a wrong implementation) |
| POSTCONDBENCH (ACL 2026 Findings) | Completeness measured by discriminating mutants, a large gap between correctness and completeness |

BMB already has the spec strength check (ADR-0021, implication of the hidden reference contract) and the mutation lint (W0904), so it can compute the same metrics.

### General coding and tool use (context, not a direct goal)
| Benchmark | Use |
| --- | --- |
| SWE-bench Verified/Pro, Terminal-Bench 2.x, Aider polyglot (including Rust), LiveCodeBench | The model's general coding ability. BMB has no ecosystem, so not applicable before phase 4. A reference for model choice |
| Multi-SWE-bench (Rust 239), Rust-SWE-bench (500) | Candidates after phase 4 (ecosystem) |
| RustEvo² | Time dependence of Rust knowledge (performance drops sharply on APIs after training) — evidence for the risk of "a language not trained on" |
| MCP-Bench, MCPAgentBench | Borrow the definitions of the number of interactions of a tool-using agent and **score per 1k tokens** (token efficiency) |

### Performance
The Rust baseline harness (`bench/baseline`, against safe and unchecked Rust) already exists. For external comparison it is enough to port Benchmarks Game style tasks to the same harness.

## Decision (proposed)

### 1. North star
> **With the same model and the same budget, code an agent writes in BMB has fewer defects than code written in Rust+tests, has a higher one-shot success rate than code written in a verified language (Verus), its navigation cost does not grow as the codebase grows, and its performance equals safe Rust.**

This is divided into five goals. Each goal has a metric, a benchmark, and a target value per phase.

| Goal | Metric | Benchmark | M1 (now) | M2 | M3 |
| --- | --- | --- | --- | --- | --- |
| G1 Verdict trust | Number of soundness violations, number of false counterexamples | 7 fuzzers (gate, nightly) | 0 | 0 (including new features) | 0 |
| G2 One-shot correctness | One-shot success rate of authoring tasks (including the spec strength check) | m1·m1b, **BVB** (below) | m1 ≥ 60% | m1b ≥ 60%, +10%p over Verus on BVB | m2 ≥ 60% |
| G3 Spec quality | Mutant kill rate (the share of hidden mutants the spec rejects) | m1b authoring tasks + mutation | Measurement starts | ≥ 90% | ≥ 90% |
| G4 Navigation scalability | Size independence of tokens and time per query, navigation accuracy, token ratio against file-based | m3 (10,000 definitions), **BNB** (below) | Response size independent ✅, time independent ✅ | Navigation P·R=1 ≥ 80% | At 10,000 definitions, tokens 1/3 or less of file-based, accuracy equal or better |
| G5 Premises E2·E3 | Hidden defect rate, total tokens (writing + verification), wall clock | **BRB** (below) | — | Defect rate 1/2 or less of Rust+tests, total tokens within 2 times | Under the same conditions, total tokens at or below Rust+tests |
| G6 Performance | Time ratio against safe Rust | `bench/baseline` + ported tasks | ±5% ✅ | ±5% | Claim "faster" only when proof-based optimization is measured |
| (Common) Rust habit error rate | E02xx / commit | Every agent measurement | Recorded | Downward trend | Downward trend |

The target values are a starting point. A first calibration is done when the 1E-10 (real model) results arrive, and after that they change only at the end of a phase (2H, 3E). The rules of the measurement procedure (frozen, one shot, no retry, biases stated) follow ADR-0021.

### 2. Benchmarks to build

| Name | Content | Compared against | Phase |
| --- | --- | --- | --- |
| **BVB** (BMB-Verus Bench) | Port the public Verus tasks (Verus-Bench, Vericoding Verus, VeriContest) that fall within BMB's scope. The original spec is the hidden reference contract. The same model is also run on the original Verus toolchain | Verus results for the same tasks and model (run directly) + values reported in papers (for reference) | Right after 2A (M1-scope tasks only), extended after 2B |
| **BRB** (BMB vs Rust+tests) | Have the same tasks solved in (a) BMB+proof and (b) Rust+tests written by the agent, and measure defects with hidden adversarial tests and mutants. Tokens, time, defect rate | Rust+tests | The core of the M2 measurement (2H) |
| **BNB** (BMB Navigation Bench) | A 10,000-definition codebase made by the m3 generator, and a Rust twin generated with the same structure. The same navigation and modification questions in BMB (MCP map tools) and Rust (file and grep tools) | File-based agent | 3A·3E |
| **BSB** (Spec strength) | A hidden mutant set per m1b authoring task. The kill rate of the agent's spec | The definitions of nl2postcond·POSTCONDBENCH | 2A |
| m1 / m1b / m2 / m3 | Existing and planned internal task sets | Itself (trend) | In progress |

Fairness rules (extension of ADR-0021): the comparison arms use the same model, the same token cap and the same one-shot rule. The Rust side tools are supported at the same level (compiler errors, a test run tool). Ported tasks are committed together with the original source and the conversion rules, and the share of tasks dropped in porting and the reasons are reported (selection bias disclosed).

### 3. What is not done
- Scoring on general coding benchmarks (SWE-bench etc.) is not a goal. BMB does not have that ecosystem (before phase 4).
- Values reported in papers and our measurements are not ranked in one table. Comparison is only of what we ran ourselves under the same conditions.

## Rationale
- The bottleneck of public verified-code benchmarks (spec and proof) overlaps exactly with BMB's differentiators. A porting comparison is the cheapest external validation.
- E2·E3 are the reason BMB exists, yet no experiment measures them directly. Without BRB, "a proof language is better than tests" cannot be claimed (genesis §3.5: claims only by measurement).
- Principle 7 was confirmed in m3 to hold for both response size and time (`bench/m3/results`). What remains is the comparison with solving the same questions file-based (BNB).

## Consequences
- If accepted, the target values go into the pass criteria of ROADMAP 2A·2H·3E, and BVB·BSB (2A), BRB (2H preparation) and BNB (3A) tasks are created in the backlog.
- Paid runs increase (every comparison arm must be run with the same model). Runs are on the user's machine, and the maintainer sets the budget.
- The measurement environment needs a Verus toolchain installation (the BVB comparison arm).

## Approved decisions (2026-10-24, recommendation adopted)
1. The north star is adopted. The target values go into the ROADMAP as **provisional**, and are fixed once when the 1E-10 results arrive (after that they change only at the end of a phase).
2. G5 criterion: phase 2 is "defect rate 1/2 or less of Rust+tests, total tokens within 2 times", phase 3 is "total tokens at or below".
3. Measurement models: the main measurement is one frontier model, plus one small model to check the scale effect. The token cap and the 40 tool calls are as in ADR-0021.
4. BVB: draw a sample of 30~50 from the public Verus tasks and run the original Verus directly with the same model. Values reported in papers are used only for reference.
5. BRB: a property-based testing tool (proptest) is allowed on the Rust side. The comparison is against the strongest realistic testing practice.
6. MCP tool descriptions: the first runs (m1, m1b) use the frozen descriptions. Then one more m1b run that exposes the new tools (`impact`·`uses`·`lint`·`fix`) measures the tool effect separately.

## Sources
- AutoVerus: https://dl.acm.org/doi/10.1145/3763174 · KVerus: https://arxiv.org/html/2605.03822v1 · SAFE: https://www.microsoft.com/en-us/research/wp-content/uploads/2024/10/ICLR_25_SelfEvolveAutoVerus.pdf
- Vericoding: https://arxiv.org/html/2509.22908v1 · https://github.com/Beneficial-AI-Foundation/vericoding-benchmark
- VeriContest: https://arxiv.org/abs/2605.08553
- VERINA: https://arxiv.org/abs/2505.23135 · DafnyBench: https://namin.seas.harvard.edu/pubs/dafnybench.pdf
- nl2postcond: https://arxiv.org/abs/2310.01831 · POSTCONDBENCH: https://arxiv.org/pdf/2605.03356
- SWE-bench/Terminal-Bench overview: https://www.digitalapplied.com/blog/swe-bench-terminal-bench-benchmark-guide-2026 · Rust-SWE-bench·Multi-SWE-bench: https://arxiv.org/html/2602.22764v1
- RustEvo²: https://arxiv.org/abs/2503.16922
- MCP-Bench: https://arxiv.org/pdf/2508.20453 · MCPAgentBench: https://arxiv.org/abs/2512.24565
