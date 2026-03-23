import matplotlib
matplotlib.use('Agg')  # Non-interactive backend
import matplotlib.pyplot as plt
import numpy as np
from pathlib import Path

def plot_loop_comparison(aggregates: dict, output_path: Path):
    """Bar chart: median loop count per condition."""
    conditions = list(aggregates.keys())
    medians = [aggregates[c]["median_loops"] for c in conditions]

    fig, ax = plt.subplots(figsize=(8, 5))
    colors = {"bmb_contract": "#2196F3", "bmb_nocontract": "#FF9800",
              "rust": "#4CAF50", "python": "#9C27B0"}
    bars = ax.bar(conditions, medians, color=[colors.get(c, "#999") for c in conditions])
    ax.set_ylabel("Median Loop Count")
    ax.set_title("AI Code Generation: Feedback Loops by Condition")
    ax.bar_label(bars, fmt='%.1f')
    fig.tight_layout()
    fig.savefig(output_path / "loop_comparison.png", dpi=150)
    plt.close(fig)

def plot_loop_types(records: list[dict], output_path: Path):
    """Stacked bar chart: loop types (A/B/C/D) per condition."""
    # Aggregate loop_types per condition
    by_cond = {}
    for r in records:
        cond = r["condition"]
        by_cond.setdefault(cond, {"A": 0, "B": 0, "C": 0, "D": 0})
        for lt in ["A", "B", "C", "D"]:
            by_cond[cond][lt] += r["loop_types"].get(lt, 0)

    conditions = list(by_cond.keys())
    fig, ax = plt.subplots(figsize=(8, 5))
    bottom = np.zeros(len(conditions))
    colors = {"A": "#E53935", "B": "#FB8C00", "C": "#FDD835", "D": "#43A047"}
    labels = {"A": "Contract (A)", "B": "Syntax (B)", "C": "Semantic (C)", "D": "Test Fail (D)"}
    for lt in ["A", "B", "C", "D"]:
        vals = [by_cond[c][lt] for c in conditions]
        ax.bar(conditions, vals, bottom=bottom, label=labels[lt], color=colors[lt])
        bottom += np.array(vals)
    ax.set_ylabel("Total Loops")
    ax.set_title("Loop Types by Condition")
    ax.legend()
    fig.tight_layout()
    fig.savefig(output_path / "loop_types.png", dpi=150)
    plt.close(fig)

def plot_perf_ratio(aggregates: dict, output_path: Path):
    """Bar chart: performance ratio vs C baseline."""
    conditions = [c for c in aggregates if aggregates[c].get("median_perf") is not None]
    if not conditions:
        return
    perfs = [aggregates[c]["median_perf"] for c in conditions]

    fig, ax = plt.subplots(figsize=(8, 5))
    bars = ax.bar(conditions, perfs, color="#2196F3")
    ax.axhline(y=1.0, color='red', linestyle='--', label='C baseline')
    ax.set_ylabel("Performance Ratio (vs C)")
    ax.set_title("Performance: Final Code vs C Baseline")
    ax.bar_label(bars, fmt='%.2fx')
    ax.legend()
    fig.tight_layout()
    fig.savefig(output_path / "perf_ratio.png", dpi=150)
    plt.close(fig)
