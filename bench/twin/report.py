#!/usr/bin/env python3
"""Tables of the port twin's result rows (backlog 3P-05).

    report.py FILE.jsonl..          a Markdown table per axis

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

A history's row counts once in `n`/`ok` (its last step's judgement) and
has `steps` (steps that passed of all steps).
"""
import json
import sys
from collections import defaultdict

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


def main():
    rows = []
    for f in sys.argv[1:]:
        with open(f, encoding="utf-8") as h:
            rows += [json.loads(ln) for ln in h if ln.strip()]
    print(table(rows))


if __name__ == "__main__":
    main()
