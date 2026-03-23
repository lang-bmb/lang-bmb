from scipy import stats
import numpy as np

def wilcoxon_test(contract_loops: list[float], nocontract_loops: list[float]) -> dict:
    """Paired Wilcoxon signed-rank test for H1 (BMB+contract vs BMB-contract).
    Returns {statistic, p_value, effect_size, significant, direction}."""
    if len(contract_loops) != len(nocontract_loops):
        raise ValueError("Paired test requires equal-length arrays")
    diffs = [nc - c for c, nc in zip(contract_loops, nocontract_loops)]
    if all(d == 0 for d in diffs):
        return {"statistic": 0, "p_value": 1.0, "effect_size": 0, "significant": False, "direction": "equal"}
    stat, p = stats.wilcoxon(contract_loops, nocontract_loops, alternative="less")
    n = len(contract_loops)
    effect_size = stat / (n * (n + 1) / 2)  # rank-biserial
    return {"statistic": float(stat), "p_value": float(p), "effect_size": float(effect_size),
            "significant": bool(p < 0.05), "direction": "contract_fewer" if np.median(diffs) > 0 else "nocontract_fewer"}

def friedman_test(groups: dict[str, list[float]]) -> dict:
    """Friedman test for H2 (3+ repeated measures).
    groups = {"bmb": [...], "rust": [...], "python": [...]}.
    Returns {statistic, p_value, significant}."""
    arrays = list(groups.values())
    stat, p = stats.friedmanchisquare(*arrays)
    return {"statistic": float(stat), "p_value": float(p), "significant": bool(p < 0.05)}

def pairwise_wilcoxon(groups: dict[str, list[float]], alpha: float = 0.05) -> list[dict]:
    """Post-hoc pairwise Wilcoxon tests with Bonferroni correction."""
    keys = list(groups.keys())
    n_comparisons = len(keys) * (len(keys) - 1) // 2
    corrected_alpha = alpha / n_comparisons
    results = []
    for i in range(len(keys)):
        for j in range(i + 1, len(keys)):
            a, b = groups[keys[i]], groups[keys[j]]
            try:
                stat, p = stats.wilcoxon(a, b)
                results.append({"pair": f"{keys[i]} vs {keys[j]}", "statistic": float(stat),
                                "p_value": float(p), "significant": bool(p < corrected_alpha),
                                "corrected_alpha": corrected_alpha})
            except ValueError:
                results.append({"pair": f"{keys[i]} vs {keys[j]}", "error": "identical distributions"})
    return results

def compute_aggregates(records: list[dict]) -> dict:
    """Compute per-condition aggregates from experiment records.
    Each record has: condition, loop_count, final_correct, perf_ratio, loop_types."""
    by_condition = {}
    for r in records:
        cond = r["condition"]
        by_condition.setdefault(cond, []).append(r)

    agg = {}
    for cond, runs in by_condition.items():
        loops = [r["loop_count"] for r in runs]
        correct = [r["final_correct"] for r in runs]
        perfs = [r["perf_ratio"] for r in runs if r.get("perf_ratio") is not None]
        agg[cond] = {
            "median_loops": float(np.median(loops)),
            "mean_loops": float(np.mean(loops)),
            "correctness_rate": sum(correct) / len(correct) if correct else 0,
            "median_perf": float(np.median(perfs)) if perfs else None,
            "n": len(runs),
        }
    return agg
