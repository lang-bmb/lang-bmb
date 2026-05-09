# bmb-ai-bench

100-problem AI-friendly benchmark suite for BMB. Measures how well LLMs (Claude, GPT, Llama, ...) generate BMB code that compiles, passes tests, and approaches C-baseline performance.

## Quick Start

```bash
pip install -e .
bmb-ai-bench doctor                          # check environment prerequisites
bmb-ai-bench list                            # list all 100 problems
bmb-ai-bench list --category algorithm      # filter by category
bmb-ai-bench list --json                     # machine-readable output
bmb-ai-bench dashboard                       # stats by category
bmb-ai-bench validate                        # build + test all solutions (requires BMB)
bmb-ai-bench validate --category contract   # validate a category only
```

LLM experiment commands require an API key and are not yet implemented in this release:

```bash
# Future: bmb-ai-bench run --model claude-opus-4-7 --category algorithm
# Future: bmb-ai-bench analyze --results-dir results/
```

## Problem Layout

```
problems/<NN>_<name>/
  problem.md          # natural-language description
  tests.json          # input/output test cases
  baseline.c          # reference C implementation
  solution.bmb        # canonical BMB solution
  solution.rs         # Rust comparator
  metadata.json       # category, difficulty, perf_target_ratio, ...
```

## Scoring Policy: Tracking, Not Gating

Performance vs `baseline.c` is **tracked, not gated**. There is no
hard pass/fail threshold — slow solutions are reported and scored
lower, not rejected.

`perf_target_ratio` in `metadata.json` (default 1.10) is informational.
The scorer in `bmb_ai_bench/analysis/report.py` awards tiered points:

| Ratio (BMB / C baseline) | Score |
|--------------------------|-------|
| ≤ 1.05× | 15 pts |
| ≤ 1.10× | 10 pts |
| ≤ 1.20× | 5 pts |
| > 1.20× | 0 pts |

The full per-problem score also folds in correctness, build success,
and code quality. See `analysis/report.py::score_run`.

## Why no hard gate?

LLM-generated code distribution is heavy-tailed — a 1.30× outlier
on one problem says less about the model than a regression of the
median across 100 problems. We track the distribution and trend, not
individual thresholds.

C baseline flags: `-O2 -march=native` unless overridden per-problem.

## Relationship to ai-proof (historical)

`ecosystem/ai-proof/` was the earlier (smaller) experiment harness.
It was **removed in Cycle 2526** (2026-05-01) after the 3-cycle
deprecation window from Cycle 2523. The package is preserved in git
history (commit before Cycle 2526 removal) for any historical
reference; new work happens here.
