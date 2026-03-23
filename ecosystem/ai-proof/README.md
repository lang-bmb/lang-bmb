# BMB AI-Native Proof

Reproducible experiment framework proving BMB's contract system reduces AI code generation feedback loops, with C-equivalent performance.

## Quick Start

```bash
pip install -r requirements.txt
python scripts/run_experiment.py --pilot --runs 1   # 3 pilot problems
python scripts/run_experiment.py --phase 1 --runs 3  # Full Phase 1 (30 problems)
```

## Experiments

- **H1 (Primary)**: BMB+contract vs BMB-contract — isolates contract effect
- **H2 (Secondary)**: BMB vs Rust vs Python — cross-language comparison

## Spec

See `docs/superpowers/specs/2026-03-24-ai-native-proof-design.md`
