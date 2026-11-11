#!/usr/bin/env python3
"""Repeated runs of one set and the target rule (ADR-0077, backlog T0-12).

  repeat.py summary [--target 0.6] FILES...    Markdown: per set and task kind
  repeat.py diff --a FILES... --b FILES...     Markdown: B minus A (ADR-0112)

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

`diff` compares two arms on the same tasks (ADR-0112): the repeated runs of A
and of B (each side one set, model, commit and condition; the sides may differ
in any of them). For write tasks it reports the difference of the mean run
rates, B - A, with its 95% interval and the exact two-sided permutation p over
the run labels. The interval is the Welch interval of the run-level rates
(t quantile at the degrees of freedom rounded down) and never narrower than
the Newcombe interval of the pooled trials, the same floor as the single-arm
rule. A direction is claimed only when the interval excludes 0; with fewer
than four runs in an arm the claim is marked provisional (three against three
cannot reach p < 0.05 in the exact test). Tokens are compared per task: the
ratio B / A of the task's mean session tokens, then the median over tasks with
the exact order-statistic interval, and the geometric mean.
"""
import argparse
import itertools
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


def welch(ra, rb):
    """(B - A, lo, hi): Welch 95% interval of the difference of run-level means."""
    ma, mb = sum(ra) / len(ra), sum(rb) / len(rb)
    va = sum((r - ma) ** 2 for r in ra) / (len(ra) - 1) if len(ra) > 1 else 0.0
    vb = sum((r - mb) ** 2 for r in rb) / (len(rb) - 1) if len(rb) > 1 else 0.0
    sa, sb = va / len(ra), vb / len(rb)
    d = mb - ma
    if sa + sb == 0:
        return d, d, d
    den = (sa ** 2 / (len(ra) - 1) if len(ra) > 1 else 0.0) + (sb ** 2 / (len(rb) - 1) if len(rb) > 1 else 0.0)
    df = (sa + sb) ** 2 / den if den else 1
    h = T975[max(1, min(int(df), 10))] * math.sqrt(sa + sb)  # df rounded down: the wider side
    return d, d - h, d + h


def newcombe(ka, na, kb, nb):
    """Newcombe's score interval of pB - pA over the pooled trials."""
    pa, pb = ka / na, kb / nb
    (la, ua), (lb, ub) = wilson(ka, na), wilson(kb, nb)
    d = pb - pa
    return d - math.sqrt((pb - lb) ** 2 + (ua - pa) ** 2), d + math.sqrt((ub - pb) ** 2 + (pa - la) ** 2)


def permutation_p(ra, rb):
    """Exact two-sided p of |mean(B) - mean(A)| over all relabelings of the runs."""
    allr = ra + rb
    if math.comb(len(allr), len(ra)) > 200_000:
        return None
    obs = abs(sum(rb) / len(rb) - sum(ra) / len(ra))
    total = sum(allr)
    hit = n = 0
    for idx in itertools.combinations(range(len(allr)), len(ra)):
        a = sum(allr[i] for i in idx)
        n += 1
        hit += abs((total - a) / len(rb) - a / len(ra)) >= obs - 1e-12
    return hit / n


def diff_verdict(lo, hi, na, nb):
    """ADR-0112 §1-§2: a direction only when the interval excludes 0."""
    if lo <= 0 <= hi:
        return "변동 안"
    side = "B가 높음" if lo > 0 else "A가 높음"
    return side if min(na, nb) >= 4 else f"{side}(잠정: 팔당 4회 미만)"


def session_tokens(r):
    if "total_tokens" in r:
        return r["total_tokens"]
    u = r.get("model_usage") or {}
    return u.get("input_tokens", 0) + u.get("output_tokens", 0)


def median_interval(xs):
    """Median and its exact 95% order-statistic interval (None when n is too small)."""
    xs = sorted(xs)
    n = len(xs)
    med = xs[n // 2] if n % 2 else math.sqrt(xs[n // 2 - 1] * xs[n // 2])
    # Largest j with P(Bin(n, 1/2) < j) <= 0.025: [x_(j), x_(n+1-j)] covers >= 95%.
    cum, j = 0.0, 0
    for k in range(n + 1):
        cum += math.comb(n, k) / 2 ** n
        if cum > 0.025:
            j = k
            break
    return med, ((xs[j - 1], xs[n - j]) if j >= 1 else None)


def token_ratios(runs_a, runs_b):
    """Per write task: mean session tokens in B over mean in A (tasks in both)."""
    def per_task(runs):
        acc = {}
        for _, rows in runs:
            for r in rows:
                if r.get("kind", "write") == "write" and session_tokens(r):
                    acc.setdefault(r["id"], []).append(session_tokens(r))
        return {t: sum(v) / len(v) for t, v in acc.items()}
    a, b = per_task(runs_a), per_task(runs_b)
    return [b[t] / a[t] for t in sorted(a) if t in b]


def one_side(files, label):
    sets = group(files)
    if len(sets) != 1:
        sys.exit(f"diff: side {label} must be the runs of one set, got {sorted(sets)}")
    return next(iter(sets.items()))


def diff(files_a, files_b):
    (name_a, runs_a), (name_b, runs_b) = one_side(files_a, "A"), one_side(files_b, "B")
    sa, sb = kind_stats(runs_a, "write"), kind_stats(runs_b, "write")
    if not sa or not sb:
        sys.exit("diff: no write tasks on one side")
    if sa["tasks"] != sb["tasks"]:
        sys.exit(f"diff: the sides have different task counts ({sa['tasks']} and {sb['tasks']})")
    ra, rb = [k / n for k, n in sa["per_run"]], [k / n for k, n in sb["per_run"]]
    d, lo, hi = welch(ra, rb)
    nl, nh = newcombe(sum(k for k, _ in sa["per_run"]), sum(n for _, n in sa["per_run"]),
                      sum(k for k, _ in sb["per_run"]), sum(n for _, n in sb["per_run"]))
    lo, hi = min(lo, nl), max(hi, nh)
    p = permutation_p(ra, rb)
    out = [f"## 차이: B({name_b}, {len(rb)}회) − A({name_a}, {len(ra)}회), 작성 과제 {sa['tasks']}개 (ADR-0112)", "",
           "| 팔 | 실행별 성공 | 평균 |", "| --- | --- | --- |",
           f"| A | {', '.join(f'{k}/{n}' for k, n in sa['per_run'])} | {sa['mean']:.1%} |",
           f"| B | {', '.join(f'{k}/{n}' for k, n in sb['per_run'])} | {sb['mean']:.1%} |", "",
           f"성공률 차이: {d * 100:+.1f}%p, 95% 구간 {lo * 100:+.1f}~{hi * 100:+.1f}%p, "
           f"정확 순열 p = {'—' if p is None else f'{p:.3f}'} → {diff_verdict(lo, hi, len(ra), len(rb))}", ""]
    ratios = token_ratios(runs_a, runs_b)
    if ratios:
        med, iv = median_interval(ratios)
        gm = math.exp(sum(math.log(r) for r in ratios) / len(ratios))
        ivs = f"{iv[0]:.2f}~{iv[1]:.2f}" if iv else "—(과제 수 부족)"
        out += [f"과제별 토큰 비율 B/A({len(ratios)}과제): 중앙값 {med:.2f}, 95% 구간 {ivs}, 기하 평균 {gm:.2f}", ""]
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
    df = sub.add_parser("diff")
    df.add_argument("--a", nargs="+", required=True, help="repeated runs of arm A")
    df.add_argument("--b", nargs="+", required=True, help="repeated runs of arm B")
    args = ap.parse_args(argv)
    sys.stdout.write(diff(args.a, args.b) if args.cmd == "diff" else summary(args.files, args.target))


if __name__ == "__main__":
    main()
