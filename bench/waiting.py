#!/usr/bin/env python3
"""What waits for the measurement machine, and how much it asks for
(backlog INF-39).

    waiting.py table [--price MODEL=IN,OUT]... [--cap USD]
    waiting.py --write FILE [same options]

`bench/waiting.json` lists the waiting runs in the order of docs/ROADMAP.md
"우선순위" 1. A run's tokens are estimated from committed result files of
the same kind (`like`): the mean of the files a glob matches, summed over
the globs when the run covers several sets (`sum`). Nothing is estimated
without such a file. Prices (USD per million input and output tokens) and
the gateway's daily cap are the person's to give; with them the table adds
the cost and the days the cap allows, tier by tier.
"""
import argparse
import glob
import json
import math
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(HERE)
sys.path.insert(0, HERE)
sys.path.insert(0, os.path.join(HERE, "m1"))
import run_models  # noqa: E402


def estimate(run, root=ROOT, usage=run_models.usage_of):
    """(sessions, input tokens, output tokens, files read) of one execution
    of `run`, or None when no earlier run of its kind is committed."""
    total, files = [0, 0, 0], 0
    for pattern in run.get("like", []):
        paths = sorted(glob.glob(os.path.join(root, pattern)))
        rows = [usage(p) for p in paths]
        rows = [r for r in rows if r[0]]
        if not rows:
            continue
        files += len(rows)
        mean = [sum(r[k] for r in rows) / len(rows) for k in range(3)]
        if not run.get("sum") and total != [0, 0, 0]:
            raise ValueError(f"{run['what']}: several globs need \"sum\": true")
        total = [a + b for a, b in zip(total, mean)]
    if not files:
        return None
    return round(total[0]), round(total[1]), round(total[2]), files


def rows_of(spec, root=ROOT, usage=run_models.usage_of):
    out = []
    for entry in spec["entries"]:
        for run in entry["runs"]:
            est = None if run.get("cpu") else estimate(run, root, usage)
            out.append({
                "task": entry["task"], "tier": entry["tier"], "what": run["what"],
                "model": run["model"], "times": run["times"], "cpu": bool(run.get("cpu")),
                "proxy": run.get("proxy"), "est": est,
            })
    return out


def n(x):
    return f"{x:,}"


def table(rows, prices, cap):
    lines = [
        "| 순서 | 태스크 | 실행 | 모델 | 횟수 | 세션/회 | 입력 토큰/회 | 출력 토큰/회 | 입력 합 | 출력 합 | 추정의 근거 |",
        "| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |",
    ]
    for i, r in enumerate(rows, 1):
        if r["cpu"]:
            cells = ["—"] * 5 + ["모델 없음"]
        elif r["est"] is None:
            cells = ["?"] * 5 + ["지난 실행 없음"]
        else:
            s, ti, to, files = r["est"]
            basis = f"결과 파일 {files}개" + (f"({r['proxy']}의 것)" if r["proxy"] else "")
            cells = [n(s), n(ti), n(to), n(ti * r["times"]), n(to * r["times"]), basis]
        lines.append(f"| {i} | `{r['task']}` | {r['what']} | {r['model']} | {r['times']} | " + " | ".join(cells) + " |")
    lines += ["", "| 묶음 | 모델 | 실행 수 | 입력 합 | 출력 합 | 추정 없는 실행 |" + (" 비용(USD) |" if prices else ""),
              "| --- | --- | --- | --- | --- | --- |" + (" --- |" if prices else "")]
    tiers = []
    for r in rows:
        if r["tier"] not in tiers:
            tiers.append(r["tier"])
    spend = {}
    for tier in tiers:
        models = []
        for r in rows:
            if r["tier"] == tier and not r["cpu"] and r["model"] not in models:
                models.append(r["model"])
        for m in models:
            mine = [r for r in rows if r["tier"] == tier and r["model"] == m and not r["cpu"]]
            known = [r for r in mine if r["est"]]
            ti = sum(r["est"][1] * r["times"] for r in known)
            to = sum(r["est"][2] * r["times"] for r in known)
            unknown = sum(r["times"] for r in mine if not r["est"])
            line = f"| {tier} | {m} | {sum(r['times'] for r in mine)} | {n(ti)} | {n(to)} | {unknown} |"
            if prices:
                if m in prices:
                    usd = run_models.cost(ti, to, *prices[m])
                    spend[tier] = spend.get(tier, 0.0) + usd
                    line += f" {usd:,.2f} |"
                else:
                    line += " 값 없음 |"
            lines.append(line)
    if prices and cap:
        lines += ["", f"일일 한도 {cap:,.2f} USD로 나눈 날 수(추정이 있는 실행만, 한 실행은 하루 안에 끝나야 유효하다: ADR-0088):", ""]
        days = 0
        for tier in tiers:
            if tier in spend:
                d = math.ceil(spend[tier] / cap)
                days += d
                lines.append(f"- {tier}: {d}일(누적 {days}일)")
    return "\n".join(lines) + "\n"


BEGIN = "<!-- queue:data begin (bench/waiting.py --write; do not edit by hand) -->"
END = "<!-- queue:data end -->"


def parse_prices(items):
    prices = {}
    for item in items:
        try:
            model, pair = item.split("=")
            a, b = pair.split(",")
            prices[model] = (float(a), float(b))
        except ValueError:
            sys.exit(f"--price takes MODEL=IN,OUT, not {item}")
    return prices


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    ap.add_argument("cmd", nargs="?", choices=["table"])
    ap.add_argument("--write", metavar="FILE")
    ap.add_argument("--queue", default=os.path.join(HERE, "waiting.json"))
    ap.add_argument("--price", action="append", default=[], metavar="MODEL=IN,OUT")
    ap.add_argument("--cap", type=float, help="the gateway's daily cap, USD")
    args = ap.parse_args(argv)
    if not args.cmd and not args.write:
        ap.error("table or --write FILE")
    with open(args.queue, encoding="utf-8") as f:
        spec = json.load(f)
    text = table(rows_of(spec), parse_prices(args.price), args.cap)
    if not args.write:
        sys.stdout.write(text)
        return 0
    with open(args.write, encoding="utf-8") as f:
        doc = f.read()
    a, b = doc.index(BEGIN) + len(BEGIN), doc.index(END)
    with open(args.write, "w", encoding="utf-8") as f:
        f.write(doc[:a] + "\n\n" + text + "\n" + doc[b:])
    print(f"updated {args.write}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
