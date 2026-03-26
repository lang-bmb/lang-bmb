#!/usr/bin/env python3
"""Analyze cross-language experiment results with statistical tests."""
from __future__ import annotations

import json
import sys
from collections import defaultdict
from pathlib import Path

_BASE = Path(__file__).resolve().parent.parent


def load_results(results_dir: Path) -> list[dict]:
    """Load all result JSON files."""
    results = []
    for f in sorted(results_dir.glob("*_run*.json")):
        if "summary" in f.name:
            continue
        try:
            results.append(json.loads(f.read_text()))
        except Exception:
            pass
    return results


def analyze(results: list[dict]) -> dict:
    """Compute per-language and per-category statistics."""
    # Group by language
    by_lang = defaultdict(list)
    # Group by (lang, category)
    by_lang_cat = defaultdict(list)
    # Group by (problem, lang) for paired comparison
    by_prob_lang = defaultdict(list)

    for r in results:
        lang = r.get("lang", "bmb")
        cat = r.get("category", "")
        pid = r.get("problem_id", "")
        if not cat and pid:
            # Infer category from problem number
            num = int(pid.split("_")[0])
            meta_file = _BASE / "problems" / pid / "metadata.json"
            if meta_file.exists():
                cat = json.loads(meta_file.read_text()).get("category", "")
        r["category"] = cat

        by_lang[lang].append(r)
        by_lang_cat[(lang, cat)].append(r)
        by_prob_lang[(pid, lang)].append(r)

    # Per-language stats
    lang_stats = {}
    for lang, rs in by_lang.items():
        total = len(rs)
        passed = sum(1 for r in rs if r.get("final_correct"))
        loops = [r["loop_count"] for r in rs if r.get("final_correct")]
        one_shot = sum(1 for l in loops if l == 1)
        lang_stats[lang] = {
            "total": total,
            "passed": passed,
            "success_rate": passed / total if total else 0,
            "median_loops": sorted(loops)[len(loops)//2] if loops else 0,
            "avg_loops": sum(loops)/len(loops) if loops else 0,
            "one_shot": one_shot,
            "one_shot_rate": one_shot / total if total else 0,
        }

    # Per-category-language stats
    cat_lang_stats = {}
    for (lang, cat), rs in by_lang_cat.items():
        total = len(rs)
        passed = sum(1 for r in rs if r.get("final_correct"))
        loops = [r["loop_count"] for r in rs if r.get("final_correct")]
        cat_lang_stats[(lang, cat)] = {
            "total": total, "passed": passed,
            "success_rate": passed / total if total else 0,
            "median_loops": sorted(loops)[len(loops)//2] if loops else 0,
            "avg_loops": sum(loops)/len(loops) if loops else 0,
        }

    # Paired comparison: for each problem, compare BMB vs C vs Python success
    problems = set(pid for (pid, _) in by_prob_lang)
    paired = {"bmb_vs_c": {"bmb_wins": 0, "c_wins": 0, "tie": 0},
              "bmb_vs_python": {"bmb_wins": 0, "python_wins": 0, "tie": 0}}

    for pid in problems:
        bmb_runs = by_prob_lang.get((pid, "bmb"), [])
        c_runs = by_prob_lang.get((pid, "c"), [])
        py_runs = by_prob_lang.get((pid, "python"), [])

        bmb_rate = sum(1 for r in bmb_runs if r.get("final_correct")) / len(bmb_runs) if bmb_runs else 0
        c_rate = sum(1 for r in c_runs if r.get("final_correct")) / len(c_runs) if c_runs else 0
        py_rate = sum(1 for r in py_runs if r.get("final_correct")) / len(py_runs) if py_runs else 0

        if bmb_rate > c_rate: paired["bmb_vs_c"]["bmb_wins"] += 1
        elif c_rate > bmb_rate: paired["bmb_vs_c"]["c_wins"] += 1
        else: paired["bmb_vs_c"]["tie"] += 1

        if bmb_rate > py_rate: paired["bmb_vs_python"]["bmb_wins"] += 1
        elif py_rate > bmb_rate: paired["bmb_vs_python"]["python_wins"] += 1
        else: paired["bmb_vs_python"]["tie"] += 1

    return {
        "lang_stats": lang_stats,
        "cat_lang_stats": {f"{l}:{c}": v for (l, c), v in cat_lang_stats.items()},
        "paired": paired,
        "total_runs": len(results),
        "unique_problems": len(problems),
    }


def print_report(analysis: dict):
    langs = sorted(analysis["lang_stats"].keys())

    print("=" * 80)
    print("BMB AI-BENCH CROSS-LANGUAGE COMPARISON REPORT")
    print("=" * 80)

    # Overall
    print(f"\n## Overall ({analysis['total_runs']} runs, {analysis['unique_problems']} problems)")
    print(f"{'Language':<10} {'Pass':>6} {'Total':>6} {'Rate':>8} {'Med.Loop':>9} {'Avg.Loop':>9} {'1-shot':>8}")
    print("-" * 62)
    for lang in langs:
        s = analysis["lang_stats"][lang]
        print(f"{lang:<10} {s['passed']:>6} {s['total']:>6} {s['success_rate']*100:>7.1f}% "
              f"{s['median_loops']:>9} {s['avg_loops']:>9.2f} {s['one_shot_rate']*100:>7.1f}%")

    # Category breakdown
    cats = sorted(set(k.split(":")[1] for k in analysis["cat_lang_stats"]))
    print(f"\n## Per-Category Success Rates")
    header = f"{'Category':<14}" + "".join(f" {l:>10}" for l in langs)
    print(header)
    print("-" * len(header))
    for cat in cats:
        row = f"{cat:<14}"
        for lang in langs:
            key = f"{lang}:{cat}"
            s = analysis["cat_lang_stats"].get(key, {})
            rate = s.get("success_rate", 0) * 100
            row += f" {rate:>9.1f}%"
        print(row)

    # Paired comparison
    print(f"\n## Paired Comparison (per-problem win/tie/loss)")
    for key, v in analysis["paired"].items():
        lang_a, lang_b = key.split("_vs_")
        print(f"  {lang_a} vs {lang_b}: "
              f"{lang_a} wins {v[f'{lang_a}_wins']}, "
              f"{lang_b} wins {v[f'{lang_b}_wins']}, "
              f"tie {v['tie']}")

    # Verdict
    print(f"\n## Verdict")
    bmb = analysis["lang_stats"].get("bmb", {})
    c = analysis["lang_stats"].get("c", {})
    py = analysis["lang_stats"].get("python", {})

    bmb_rate = bmb.get("success_rate", 0)
    c_rate = c.get("success_rate", 0)
    py_rate = py.get("success_rate", 0)

    if bmb_rate > c_rate and bmb_rate > py_rate:
        print("  BMB outperforms both C and Python in success rate.")
    elif bmb_rate >= c_rate - 0.05 and bmb_rate >= py_rate - 0.05:
        print("  BMB is comparable to C and Python (within 5% margin).")
    else:
        best = "C" if c_rate > py_rate else "Python"
        print(f"  {best} outperforms BMB. BMB AI-friendly claim needs qualification.")

    print("=" * 80)


def main():
    results_dir = Path(sys.argv[1]) if len(sys.argv) > 1 else _BASE / "results" / "crosslang-2026-03-26"
    if not results_dir.exists():
        print(f"Results dir not found: {results_dir}")
        return 1

    results = load_results(results_dir)
    if not results:
        print("No results found")
        return 1

    analysis = analyze(results)
    print_report(analysis)

    # Save analysis
    (results_dir / "analysis.json").write_text(json.dumps(analysis, indent=2))
    return 0


if __name__ == "__main__":
    sys.exit(main())
