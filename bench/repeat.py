#!/usr/bin/env python3
"""Repeated runs of one set and the target rule (ADR-0077, backlog T0-12).

  repeat.py summary [--target 0.6] FILES...    Markdown: per set and task kind

FILES are results of `run_models.py --repeat K` (m1 harness rows): the same
set, model and commit, each with its own `repeat`. Files of different sets are
grouped per set; within a set, a different model, commit or condition, or a
repeated `repeat` number, is refused.

For each set and task kind (write, navigate) it reports every run's one-shot
success rate, their mean, the sample standard deviation, the 95% interval of
the mean, the range (max - min) and how many tasks succeeded in every run, in
none, or in some (unstable). The interval is the run-level t-interval,
mean +- t(0.975, N-1) * s / sqrt(N), and never narrower than the Wilson
interval of the pooled N x n trials (when the runs agree, s is 0 and the
t-interval says nothing about sampling).

The target rule (ADR-0077 §3): a target is met when the interval's lower end is
at or above it, missed when its upper end is below it, and undetermined
otherwise. The minimum difference for comparing single runs (§4) is the
largest range over the sets' write tasks, in whole percentage points rounded
up. Each set's own minimum difference (ADR-0086 §2) is its write range the
same way. BRB results (no task `kind`, one arm per file) count as writes of
the set named by `task_set` and `arm` (`brb-write-bmb`). Standard library only.
"""
import argparse
import json
import math
import sys

# Two-sided 95% quantiles of Student's t, by degrees of freedom.
T975 = {1: 12.706, 2: 4.303, 3: 3.182, 4: 2.776, 5: 2.571, 6: 2.447, 7: 2.365, 8: 2.306, 9: 2.262, 10: 2.228}
Z975 = 1.959964


def load(path):
    with open(path) as f:
        rows = [json.loads(l) for l in f if l.strip()]
    head = next((r for r in rows if r.get("summary")), {})
    return head, [r for r in rows if not r.get("summary") and not r.get("partial")]


def wilson(k, n):
    if n == 0:
        return 0.0, 1.0
    p = k / n
    d = 1 + Z975 ** 2 / n
    c = (p + Z975 ** 2 / (2 * n)) / d
    h = Z975 * math.sqrt(p * (1 - p) / n + Z975 ** 2 / (4 * n * n)) / d
    return max(0.0, c - h), min(1.0, c + h)


def interval(rates, k, n):
    """95% interval of the mean rate over runs (see the module doc)."""
    m = sum(rates) / len(rates)
    lo, hi = wilson(k, n)
    if len(rates) > 1:
        s = math.sqrt(sum((r - m) ** 2 for r in rates) / (len(rates) - 1))
        h = T975.get(len(rates) - 1, Z975) * s / math.sqrt(len(rates))
        lo, hi = min(lo, max(0.0, m - h)), max(hi, min(1.0, m + h))
    return lo, hi


def verdict(lo, hi, target):
    """ADR-0077 §3: met, missed, or undetermined."""
    if lo >= target:
        return "충족"
    if hi < target:
        return "미충족"
    return "미정"


def group(files):
    """{set: [(repeat, rows)]}, refusing mixed models, commits, conditions or repeats."""
    sets = {}
    for path in files:
        head, rows = load(path)
        if "repeat" not in head:
            sys.exit(f"{path}: not a repeated run (no `repeat` in its header, ADR-0077)")
        name = head.get("task_set", "m1") + (f"-{head['arm']}" if head.get("arm") else "")
        key = (head.get("agent"), head.get("commit"), json.dumps(head.get("condition"), sort_keys=True))
        runs = sets.setdefault(name, {"key": key, "runs": []})
        if runs["key"] != key:
            sys.exit(f"{path}: {name} runs differ in model, commit or condition (ADR-0077 §1)")
        if any(r == head["repeat"] for r, _ in runs["runs"]):
            sys.exit(f"{path}: {name} repeat {head['repeat']} given twice")
        runs["runs"].append((head["repeat"], rows))
    return {k: sorted(v["runs"]) for k, v in sets.items()}


def kind_stats(runs, kind):
    """Per-run (k, n), interval, range and task stability for one task kind."""
    per = []
    by_task = {}
    for _, rows in runs:
        rs = [r for r in rows if r.get("kind", "write") == kind]  # BRB rows are all writes
        per.append((sum(1 for r in rs if r.get("success")), len(rs)))
        for r in rs:
            by_task.setdefault(r["id"], []).append(bool(r.get("success")))
    if not per or not per[0][1]:
        return None
    rates = [k / n for k, n in per]
    k, n = sum(k for k, _ in per), sum(n for _, n in per)
    lo, hi = interval(rates, k, n)
    always = sum(1 for v in by_task.values() if all(v))
    never = sum(1 for v in by_task.values() if not any(v))
    return {"per_run": per, "mean": sum(rates) / len(rates), "lo": lo, "hi": hi,
            "range": max(rates) - min(rates), "always": always, "never": never,
            "unstable": sorted(t for t, v in by_task.items() if any(v) and not all(v)), "tasks": len(by_task)}


def summary(files, target):
    sets = group(files)
    out = []
    ranges = []
    per_set = []
    for name, runs in sorted(sets.items()):
        out += [f"## {name}: 반복 {len(runs)}회", "",
                "| 과제 종류 | 실행별 성공 | 평균 | 95% 구간 | 범위 | 늘 성공 / 늘 실패 / 흔들림 | 목표 " +
                f"{target:.0%} |", "| --- | --- | --- | --- | --- | --- | --- |"]
        for kind, label in (("write", "작성"), ("navigate", "탐색")):
            st = kind_stats(runs, kind)
            if not st:
                continue
            if kind == "write":
                ranges.append(st["range"])
                per_set.append((name, st["range"]))
            out.append(f"| {label} | {', '.join(f'{k}/{n}' for k, n in st['per_run'])} | {st['mean']:.1%} | "
                       f"{st['lo']:.1%}~{st['hi']:.1%} | {st['range'] * 100:.1f}%p | "
                       f"{st['always']} / {st['never']} / {len(st['unstable'])} | {verdict(st['lo'], st['hi'], target)} |")
        unstable = sorted({t for kind in ("write", "navigate") if (st := kind_stats(runs, kind)) for t in st["unstable"]})
        out += ["", f"흔들린 과제: {', '.join(unstable) if unstable else '없음'}", ""]
    if ranges:
        out.append(f"단일 실행 비교의 최소 차이(ADR-0077 §4): {math.ceil(max(ranges) * 100 - 1e-9)}%p")
        out.append("")
        out.append("집합별 최소 차이(ADR-0086 §2): " + ", ".join(f"{n} {points(r)}%p" for n, r in per_set))
    return "\n".join(out) + "\n"


def points(r):
    """A range in whole percentage points, rounded up."""
    return math.ceil(r * 100 - 1e-9)


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    sub = ap.add_subparsers(dest="cmd", required=True)
    sm = sub.add_parser("summary")
    sm.add_argument("--target", type=float, default=0.6, help="target rate (G2: 0.6)")
    sm.add_argument("files", nargs="+")
    args = ap.parse_args(argv)
    sys.stdout.write(summary(args.files, args.target))


if __name__ == "__main__":
    main()
