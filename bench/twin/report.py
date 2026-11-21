#!/usr/bin/env python3
"""Tables of the port twin's result rows (backlog 3P-05).

    report.py FILE.jsonl..          a Markdown table per axis
    report.py diff FILE.jsonl..     the two arms compared per axis (ADR-0112)
    report.py worked DIR FILE.jsonl..   change tasks: what the sessions judged
                                    successful changed (DIR holds the tasks)

A file holds the rows of one run (`pair.py run --out FILE`: one agent, one
documentation condition). Rows are grouped by agent, condition and
family; an axis (comprehension, refactoring, replay, history) is the sum
of its families. Per group and arm:

    n          sessions
    ok         sessions that succeeded (first attempt, no retry)
    read       tokens of the tool results (ceil(bytes / 4), ADR-0021)
    in / out   tokens the model's API counted (prompt and completion)
    calls      tool calls
    limit      sessions that reached the call limit

`diff` takes the runs of one agent and condition (a file per run) and
compares the arms, BMB minus Rust, per axis and family: the difference of
the run-level success rates with its 95% interval (Welch, never narrower
than Newcombe's interval of the pooled sessions) and the exact permutation p,
then per task the ratio BMB / Rust of the mean session tokens (the read
tokens, and the tokens the API counted), its median over the tasks with the
exact order-statistic interval, and the geometric mean. The rules and the
functions are those of `bench/repeat.py`.

`worked` is a check after the fact on the change tasks whose scope is a list
(refactoring): the judge asks for the build, the corpus and nothing changed
out of scope, so a session that changed nothing passes. Per run and arm it
counts the sessions judged successful, those of them that changed exactly
the definitions of the task's scope (what the reference solution changes),
those that changed nothing, and the rest.

A history's row counts once in `n`/`ok` (its last step's judgement) and
has `steps` (steps that passed of all steps).
"""
import json
import math
import os
import sys
from collections import defaultdict

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
import repeat  # noqa: E402

AXES = {"callers": "comprehend", "impact": "comprehend", "diag": "comprehend", "state": "comprehend",
        "rename": "refactor", "swap": "refactor", "inline": "refactor", "extract": "refactor", "move": "refactor",
        "replay": "replay", "history": "history"}


def family(task_id):
    return task_id.split("-")[1]


def usage(row, key):
    u = row.get("model_usage") or {}
    names = {"in": ("input_tokens", "prompt_tokens"), "out": ("output_tokens", "completion_tokens")}[key]
    return sum(u.get(n, 0) or 0 for n in names)


def table(rows):
    groups = defaultdict(lambda: defaultdict(list))
    for r in rows:
        fam = family(r["id"])
        for name in (AXES[fam], f"{AXES[fam]}/{fam}"):
            groups[(r["agent"], r.get("docs"), name)][r["arm"]].append(r)
    out = ["| agent | docs | group | arm | n | ok | read | in | out | calls | limit | steps |",
           "| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |"]
    for (agent, docs, name), arms in sorted(groups.items()):
        for arm in ("bmb", "rust"):
            rs = arms.get(arm, [])
            if not rs:
                continue
            steps = [st for r in rs for st in r.get("steps", [])]
            out.append(f"| {agent} | {docs} | {name} | {arm} | {len(rs)} | {sum(bool(r['success']) for r in rs)} | "
                       f"{sum(r['tokens'] for r in rs)} | {sum(usage(r, 'in') for r in rs)} | "
                       f"{sum(usage(r, 'out') for r in rs)} | {sum(r['tool_calls'] for r in rs)} | "
                       f"{sum(bool(r.get('limit_hit') or any(s.get('limit_hit') for s in r.get('steps', []))) for r in rs)} | "
                       f"{str(sum(bool(s['success']) for s in steps)) + '/' + str(len(steps)) if steps else ''} |")
    return "\n".join(out)


def ratio_cell(runs, group, value):
    """Median, interval and geometric mean of BMB / Rust per task."""
    means = {}
    for arm in ("bmb", "rust"):
        acc = defaultdict(list)
        for rows in runs:
            for r in rows:
                if r["arm"] == arm and group(r):
                    acc[r["id"]].append(value(r))
        means[arm] = {t: sum(v) / len(v) for t, v in acc.items()}
    ratios = [means["bmb"][t] / means["rust"][t] for t in sorted(means["bmb"])
              if means["rust"].get(t) and means["bmb"][t]]
    if not ratios:
        return "—"
    med, iv = repeat.median_interval(ratios)
    gm = math.exp(sum(math.log(x) for x in ratios) / len(ratios))
    return f"{med:.2f} ({f'{iv[0]:.2f}~{iv[1]:.2f}' if iv else '—'}), 기하 {gm:.2f}"


def diff(files):
    runs = []
    for f in files:
        with open(f, encoding="utf-8") as h:
            runs.append([json.loads(ln) for ln in h if ln.strip()])
    heads = {(r["agent"], r.get("docs")) for rows in runs for r in rows}
    if len(heads) != 1:
        sys.exit(f"diff: the runs of one agent and condition, got {sorted(heads)}")
    agent, docs = next(iter(heads))
    names = []
    for rows in runs:
        for r in rows:
            fam = family(r["id"])
            for name in (AXES[fam], f"{AXES[fam]}/{fam}"):
                if name not in names:
                    names.append(name)
    out = [f"{agent}, docs `{docs}`, {len(runs)}회: BMB − Rust (ADR-0112)", "",
           "| group | BMB 성공(실행별) | Rust 성공(실행별) | 차이 %p | 95% 구간 %p | p | 판정 | 읽은 토큰 BMB/Rust: 중앙값 (구간) | API 토큰 BMB/Rust |",
           "| --- | --- | --- | --- | --- | --- | --- | --- | --- |"]
    for name in sorted(names):
        def group(r, name=name):
            fam = family(r["id"])
            return name in (AXES[fam], f"{AXES[fam]}/{fam}")
        per = {}
        for arm in ("bmb", "rust"):
            per[arm] = [(sum(bool(r["success"]) for r in rows if r["arm"] == arm and group(r)),
                         sum(1 for r in rows if r["arm"] == arm and group(r))) for rows in runs]
        if any(n == 0 for arm in per for _, n in per[arm]):
            sys.exit(f"diff: {name}: a run without sessions of an arm")
        ra, rb = [k / n for k, n in per["rust"]], [k / n for k, n in per["bmb"]]
        d, lo, hi = repeat.welch(ra, rb)
        nl, nh = repeat.newcombe(sum(k for k, _ in per["rust"]), sum(n for _, n in per["rust"]),
                                 sum(k for k, _ in per["bmb"]), sum(n for _, n in per["bmb"]))
        lo, hi = min(lo, nl), max(hi, nh)
        p = repeat.permutation_p(ra, rb) if len(runs) > 1 else None
        verdict = repeat.diff_verdict(lo, hi, len(ra), len(rb)).replace("B가", "BMB가").replace("A가", "Rust가")
        out.append(f"| {name} | {', '.join(f'{k}/{n}' for k, n in per['bmb'])} | "
                   f"{', '.join(f'{k}/{n}' for k, n in per['rust'])} | {d * 100:+.1f} | {lo * 100:+.1f}~{hi * 100:+.1f} | "
                   f"{'—' if p is None else f'{p:.3f}'} | {verdict} | {ratio_cell(runs, group, lambda r: r['tokens'])} | "
                   f"{ratio_cell(runs, group, lambda r: usage(r, 'in') + usage(r, 'out'))} |")
    return "\n".join(out)


def worked(tasks_dir, files):
    out = ["| run | arm | judged ok | changed the scope | changed nothing | other | sessions cut by an API error |",
           "| --- | --- | --- | --- | --- | --- | --- |"]
    for f in files:
        with open(f, encoding="utf-8") as h:
            rows = [json.loads(ln) for ln in h if ln.strip()]
        for arm in ("bmb", "rust"):
            n = ok = full = none = cut = 0
            for r in rows:
                if r["arm"] != arm or r["kind"] != "change":
                    continue
                with open(os.path.join(tasks_dir, r["id"] + ".json"), encoding="utf-8") as h:
                    scope = json.load(h).get("scope")
                if not isinstance(scope, dict):
                    continue
                n += 1
                cut += r.get("stop") == "api_error"
                if r["success"]:
                    ok += 1
                    got = set(r.get("changed") or [])
                    full += got == set(scope.get(arm, []))
                    none += not got
            if n:
                out.append(f"| {os.path.basename(f)[:-6]} | {arm} | {ok}/{n} | {full} | {none} | {ok - full - none} | {cut} |")
    return "\n".join(out)


def main():
    if sys.argv[1:2] == ["worked"]:
        print(worked(sys.argv[2], sys.argv[3:]))
        return
    if sys.argv[1:2] == ["diff"]:
        print(diff(sys.argv[2:]))
        return
    rows = []
    for f in sys.argv[1:]:
        with open(f, encoding="utf-8") as h:
            rows += [json.loads(ln) for ln in h if ln.strip()]
    print(table(rows))


if __name__ == "__main__":
    main()
