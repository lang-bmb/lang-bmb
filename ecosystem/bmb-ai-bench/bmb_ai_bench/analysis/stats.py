"""Statistical significance analysis for cross-language experiment results."""
from __future__ import annotations

import json
import math
import sys
from collections import defaultdict
from pathlib import Path


def wilson_ci(successes: int, n: int, z: float = 1.96) -> tuple[float, float]:
    """Wilson score confidence interval for a proportion."""
    if n == 0:
        return (0.0, 0.0)
    p = successes / n
    center = (p + z * z / (2 * n)) / (1 + z * z / n)
    margin = z * math.sqrt(p * (1 - p) / n + z * z / (4 * n * n)) / (1 + z * z / n)
    return (max(0.0, center - margin), min(1.0, center + margin))


def mcnemar_test(a_only: int, b_only: int) -> tuple[float, float]:
    """McNemar's test with continuity correction. Returns (chi2, p_approx)."""
    n = a_only + b_only
    if n == 0:
        return (0.0, 1.0)
    chi2 = (abs(a_only - b_only) - 1) ** 2 / n
    # Chi-squared df=1 approximation: p = exp(-chi2/2)
    p_approx = math.exp(-chi2 / 2)
    return (chi2, p_approx)


def _problem_passes(data: dict, lang: str, pid: str, threshold: int = 2) -> bool:
    """True if lang solved pid in at least threshold runs."""
    runs = data[lang].get(pid, [])
    return sum(runs) >= threshold


def run_stats(results_dir: str | Path, json_output: bool = False) -> dict:
    """Compute statistics for a crosslang results directory."""
    r = Path(results_dir)
    data: dict = defaultdict(lambda: defaultdict(list))

    for f in sorted(r.glob("*.json")):
        try:
            d = json.loads(f.read_text(encoding="utf-8"))
        except Exception:
            continue
        pid = d.get("problem_id")
        lang = d.get("lang")
        correct = d.get("final_correct", False)
        if pid and lang:
            data[lang][pid].append(bool(correct))

    langs = sorted(data.keys())
    problems = sorted({pid for lang in langs for pid in data[lang]})

    lang_stats = {}
    for lang in langs:
        n = sum(len(v) for v in data[lang].values())
        s = sum(sum(v) for v in data[lang].values())
        lo, hi = wilson_ci(s, n)
        lang_stats[lang] = {
            "passed": s,
            "total": n,
            "rate": round(s / n * 100, 1) if n else 0.0,
            "ci_lo": round(lo * 100, 1),
            "ci_hi": round(hi * 100, 1),
        }

    pairwise = {}
    for a, b in [("bmb", "c"), ("bmb", "python"), ("c", "python")]:
        if a not in data or b not in data:
            continue
        a_only = sum(1 for p in problems if _problem_passes(data, a, p) and not _problem_passes(data, b, p))
        b_only = sum(1 for p in problems if _problem_passes(data, b, p) and not _problem_passes(data, a, p))
        both = sum(1 for p in problems if _problem_passes(data, a, p) and _problem_passes(data, b, p))
        neither = sum(1 for p in problems if not _problem_passes(data, a, p) and not _problem_passes(data, b, p))
        chi2, p_val = mcnemar_test(a_only, b_only)
        pairwise[f"{a}_vs_{b}"] = {
            "a_only": a_only, "b_only": b_only, "both": both, "neither": neither,
            "mcnemar_chi2": round(chi2, 3),
            "mcnemar_p": round(p_val, 4),
            "significant_05": p_val < 0.05,
        }

    result = {
        "dataset": str(r),
        "n_problems": len(problems),
        "languages": lang_stats,
        "pairwise_tests": pairwise,
    }

    if json_output:
        print(json.dumps(result, indent=2))
    else:
        print("=== BMB AI-Bench Statistical Analysis ===")
        print("Dataset: %s, %d problems" % (r.name, len(problems)))
        print()
        for lang, s in lang_stats.items():
            print("%s: %d/%d = %.1f%% [95%% CI: %.1f%%--%.1f%%]" % (
                lang, s["passed"], s["total"], s["rate"], s["ci_lo"], s["ci_hi"]))
        print()
        print("=== Pairwise McNemar Tests ===")
        for key, t in pairwise.items():
            a, _, b = key.partition("_vs_")
            sig = "(significant at alpha=0.05)" if t["significant_05"] else "(not significant)"
            print("%s vs %s: %s_only=%d, %s_only=%d, both=%d, neither=%d" % (
                a, b, a, t["a_only"], b, t["b_only"], t["both"], t["neither"]))
            print("  McNemar x2=%.3f, p=%.4f %s" % (
                t["mcnemar_chi2"], t["mcnemar_p"], sig))
    return result


if __name__ == "__main__":
    import argparse
    p = argparse.ArgumentParser(description="Statistical analysis for crosslang results")
    p.add_argument("results_dir", help="Path to crosslang results directory")
    p.add_argument("--json", action="store_true", help="JSON output")
    args = p.parse_args()
    run_stats(args.results_dir, json_output=args.json)
