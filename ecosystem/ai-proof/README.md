# BMB AI-Native Proof — DEPRECATED

> **Status**: Deprecated as of 2026-05-01 (Cycle 2523). Use
> [`ecosystem/bmb-ai-bench/`](../bmb-ai-bench/) for new work.

## Why deprecated

This package was the original experiment harness (~30 pilot
problems, two-hypothesis design: H1 contract effect, H2 cross-lang).
Its functionality is now covered — and substantially extended — by
`bmb-ai-bench`:

| Capability | ai-proof | bmb-ai-bench |
|------------|----------|--------------|
| Problem count | ~30 | 100 |
| Multi-LLM runner | manual | `bmb_ai_bench.runner.llm_client` |
| Tiered perf scoring | hard gate | tracking-only (≤1.05/1.10/1.20×) |
| CLI | scripts/ | `bmb-ai-bench` entry point |
| Active maintenance | no | yes |

Maintaining two parallel harnesses violates BMB's single-source-of-truth
principle and creates drift in problem semantics.

## Migration

If you have analysis code or notebooks pointing at this package:

1. Move problem references to `ecosystem/bmb-ai-bench/problems/`
2. Replace `scripts/run_experiment.py` calls with `bmb-ai-bench run`
3. Replace gate-based pass/fail with tiered scoring (see
   `bmb-ai-bench/README.md` § "Scoring Policy")

## Removal timeline

This package will be deleted after Cycle 2526 (~3 cycles) absent
objection. Existing results in `results/` are preserved as
historical artifacts in `claudedocs/`.

## Original spec

See `docs/superpowers/specs/2026-03-24-ai-native-proof-design.md`
for the original design and the H1/H2 hypothesis statements that
motivated this harness.
