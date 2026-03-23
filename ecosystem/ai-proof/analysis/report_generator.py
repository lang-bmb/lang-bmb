"""Generate AI_NATIVE_PROOF.md from experiment results."""
import json
from pathlib import Path
from datetime import date


def generate_report(results_dir: Path, output_path: Path, protocol_hash: str = ""):
    """Generate the full analysis report."""
    summary_path = results_dir / "summary.json"
    if not summary_path.exists():
        raise FileNotFoundError(f"No summary.json in {results_dir}")

    summary = json.loads(summary_path.read_text(encoding="utf-8"))

    # Load all individual run results
    records = _load_all_records(results_dir / "raw")

    # Import analysis functions (lazy to avoid import-time failures)
    from analysis.stats import compute_aggregates, wilcoxon_test, friedman_test, pairwise_wilcoxon

    aggregates = compute_aggregates(records)

    # Separate H1 and H2 records
    h1_contract = [r for r in records if r["condition"] == "bmb_contract"]
    h1_nocontract = [r for r in records if r["condition"] == "bmb_nocontract"]
    h2_bmb = [r for r in records if r["condition"] == "bmb_contract"]
    h2_rust = [r for r in records if r["condition"] == "rust"]
    h2_python = [r for r in records if r["condition"] == "python"]

    # Build report sections
    sections = []
    sections.append(_section_header(summary, protocol_hash))
    sections.append(_section_executive_summary(aggregates, h1_contract, h1_nocontract))
    sections.append(_section_experiment_design(summary))
    sections.append(_section_h1_results(h1_contract, h1_nocontract))
    sections.append(_section_h2_results(h2_bmb, h2_rust, h2_python, aggregates))
    sections.append(_section_performance(aggregates))
    sections.append(_section_limitations())
    sections.append(_section_reproduction())

    report = "\n\n---\n\n".join(sections)
    output_path.parent.mkdir(parents=True, exist_ok=True)
    output_path.write_text(report, encoding="utf-8")
    return output_path


def _load_all_records(raw_dir: Path) -> list[dict]:
    """Load all *_result.json files from raw results directory."""
    records = []
    if not raw_dir.exists():
        return records
    for problem_dir in sorted(raw_dir.iterdir()):
        if not problem_dir.is_dir():
            continue
        for run_dir in sorted(problem_dir.iterdir()):
            if not run_dir.is_dir():
                continue
            for result_file in run_dir.glob("*_result.json"):
                data = json.loads(result_file.read_text(encoding="utf-8"))
                data["problem"] = problem_dir.name
                records.append(data)
    return records


def _section_header(summary: dict, protocol_hash: str) -> str:
    return f"""# BMB AI-Native Proof — Results Report

**Date:** {summary.get('date', date.today().isoformat())}
**LLM:** {summary.get('llm', {}).get('model', 'unknown')}
**Protocol hash:** `{protocol_hash or 'not registered'}`
**Problems:** {summary.get('total_problems', '?')}
**Runs per condition:** {summary.get('runs_per_condition', '?')}"""


def _section_executive_summary(aggregates: dict, h1_c: list, h1_nc: list) -> str:
    """One-paragraph summary of key findings."""
    bmb_c = aggregates.get("bmb_contract", {})
    bmb_nc = aggregates.get("bmb_nocontract", {})
    rust = aggregates.get("rust", {})

    lines = ["## Executive Summary\n"]
    if bmb_c and bmb_nc:
        nc_median = bmb_nc.get("median_loops", 0)
        c_median = bmb_c.get("median_loops", 0)
        reduction = ((nc_median - c_median) / nc_median * 100) if nc_median > 0 else 0
        lines.append(f"BMB with contracts required **{c_median}** median loops "
                     f"vs **{nc_median}** without contracts "
                     f"({reduction:.0f}% reduction).")
    if rust:
        lines.append(f"Rust required **{rust.get('median_loops', '?')}** median loops.")
    if bmb_c.get("median_perf"):
        lines.append(f"BMB performance: **{bmb_c['median_perf']:.2f}x** vs C baseline.")
    return "\n".join(lines)


def _section_experiment_design(summary: dict) -> str:
    return """## Experiment Design

- **H1 (Primary):** BMB+contract vs BMB-contract — isolates contract effect
- **H2 (Secondary):** BMB vs Rust vs Python — cross-language comparison
- **Loop types:** A (contract), B (syntax), C (semantic), D (test failure)
- **Max loops:** 10 per problem per condition"""


def _section_h1_results(h1_c: list, h1_nc: list) -> str:
    """H1 results with Wilcoxon test."""
    lines = ["## H1 Results: Contract Effect (Primary)\n"]
    if not h1_c or not h1_nc:
        lines.append("*No H1 data available.*")
        return "\n".join(lines)

    # Pair by problem name, compute stats
    from analysis.stats import wilcoxon_test
    import numpy as np

    c_by_prob: dict[str, list] = {}
    nc_by_prob: dict[str, list] = {}
    for r in h1_c:
        c_by_prob.setdefault(r.get("problem", ""), []).append(r["loop_count"])
    for r in h1_nc:
        nc_by_prob.setdefault(r.get("problem", ""), []).append(r["loop_count"])

    # Use median per problem for pairing
    common = sorted(set(c_by_prob.keys()) & set(nc_by_prob.keys()))
    c_medians = [float(np.median(c_by_prob[p])) for p in common]
    nc_medians = [float(np.median(nc_by_prob[p])) for p in common]

    lines.append("| Problem | +Contract | -Contract | Diff |")
    lines.append("|---------|-----------|-----------|------|")
    for p, c, nc in zip(common, c_medians, nc_medians):
        diff = nc - c
        lines.append(f"| {p} | {c:.1f} | {nc:.1f} | {diff:+.1f} |")

    if len(common) >= 5:
        result = wilcoxon_test(c_medians, nc_medians)
        lines.append(f"\n**Wilcoxon signed-rank test:** p = {result['p_value']:.4f}, "
                     f"effect size = {result['effect_size']:.3f}, "
                     f"significant = {'Yes' if result['significant'] else 'No'}")
    else:
        lines.append(f"\n*N={len(common)} too small for Wilcoxon test (need >= 5).*")

    return "\n".join(lines)


def _section_h2_results(bmb: list, rust: list, python: list, aggregates: dict) -> str:
    lines = ["## H2 Results: Cross-Language (Secondary)\n"]
    lines.append("| Condition | Median Loops | Correctness | Median Perf |")
    lines.append("|-----------|-------------|-------------|-------------|")
    for cond in ["bmb_contract", "rust", "python"]:
        a = aggregates.get(cond, {})
        perf = f"{a['median_perf']:.2f}x" if a.get("median_perf") else "N/A"
        lines.append(f"| {cond} | {a.get('median_loops', '?')} | {a.get('correctness_rate', 0):.0%} | {perf} |")
    return "\n".join(lines)


def _section_performance(aggregates: dict) -> str:
    lines = ["## Performance vs C Baseline\n"]
    for cond in ["bmb_contract", "rust"]:
        a = aggregates.get(cond, {})
        if a.get("median_perf"):
            lines.append(f"- **{cond}:** {a['median_perf']:.2f}x C baseline")
    return "\n".join(lines)


def _section_limitations() -> str:
    return """## Limitations

- BMB has no LLM training data — language reference provided as prompt context
- Single LLM (Claude) — model-specific biases possible
- Contract category problems (21-30) intentionally favor BMB — reported separately
- Error message quality differs across compilers — normalized format mitigates but doesn't eliminate"""


def _section_reproduction() -> str:
    return """## Reproduction

```bash
cd ecosystem/ai-proof
pip install -r requirements.txt
python scripts/run_experiment.py --phase 1 --runs 3
```"""
