#!/usr/bin/env python3
"""Data tables of the utility report (backlog T0-26, `dev-docs/reports/utility.md`).

  utility.py tables              the tables, from every official results file
  utility.py --write REPORT      replace the generated block of REPORT
  utility.py --check REPORT      exit 1 if the block is stale (gate job)

Official results are the files the measurement path writes (`run_models.py`,
ADR-0066): `bench/{m1,m1b,bvb,brb}/results/<date>-<model>[-arm].jsonl` with a
summary row. Pilot files (`bench/pilot/results/`) are not read: the report
cites them by hand. Every number here comes from the result rows; nothing is
estimated. Standard library only.
"""
import glob
import json
import os
import re
import statistics
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(HERE)
SETS = ["m1", "m1b", "bvb", "brb"]
BEGIN = "<!-- utility:data begin (bench/utility.py --write; do not edit by hand) -->"
END = "<!-- utility:data end -->"
# Tools that only read the workspace (BMB MCP, BVB Verus, BRB Rust).
READS = {"query", "map", "impact", "uses", "read", "list_files", "read_file", "show", "search"}
CHECKS = {"verify", "cargo_test", "lint"}


def runs():
    """[(set, model, arm, summary, rows)] of the official results files, in a fixed order."""
    out = []
    for s in SETS:
        for path in sorted(glob.glob(os.path.join(ROOT, "bench", s, "results", "*.jsonl"))):
            with open(path) as f:
                recs = [json.loads(line) for line in f if line.strip()]
            summary = next((r for r in recs if r.get("summary")), None)
            if not summary:
                continue
            agent = summary.get("agent", "")
            model = agent.split(":", 1)[-1]
            name = os.path.basename(path)[11:-6]  # strip the date and .jsonl
            arm = name[len(model) + 1:] if name.startswith(model + "-") else ""
            if s == "bvb":
                arm = arm or "bmb"
            out.append((s, model, arm, summary, [r for r in recs if "id" in r]))
    return sorted(out, key=lambda x: (SETS.index(x[0]), x[2] != "bmb", x[2], x[1]))


def ratio(a, b):
    return f"{a}/{b}" if b else "—"


def med(xs):
    return f"{statistics.median(xs):g}" if xs else "—"


def tools(r):
    counts = r.get("tool_counts") or {}
    reads = sum(n for t, n in counts.items() if t in READS)
    checks = sum(n for t, n in counts.items() if t in CHECKS)
    return counts, reads, checks


def total_tokens(r):
    if "total_tokens" in r:
        return r["total_tokens"]
    u = r.get("model_usage") or {}
    return u.get("input_tokens", 0) + u.get("output_tokens", 0)


def outcome_table(rs):
    lines = ["| 집합 | 팔 | 모델 | 작성 성공 | 총 토큰 | 성공당 토큰 | 세션당 도구 호출(중앙값) |",
             "| --- | --- | --- | --- | --- | --- | --- |"]
    for s, model, arm, summary, rows in rs:
        write = [r for r in rows if r.get("kind", "write") != "navigate"]
        ok = sum(1 for r in write if r.get("success"))
        tok = sum(total_tokens(r) for r in write)
        per = f"{tok // ok:,}" if ok else "—"
        calls = [r.get("tool_calls", 0) for r in write]
        lines.append(f"| {s} | {arm or 'bmb'} | {model} | {ratio(ok, len(write))} | {tok:,} | {per} | {med(calls)} |")
    return lines


def test_code_table(rs):
    lines = ["| 모델 | 팔 | 성공 | 성공 세션의 테스트 코드 줄 | 성공 세션당 | 자기 검사 호출(중앙값) | 통과 & 숨은 실패 |",
             "| --- | --- | --- | --- | --- | --- | --- |"]
    for s, model, arm, summary, rows in rs:
        if s != "brb":
            continue
        good = [r for r in rows if r.get("success")]
        lines_ = sum((r.get("test_code") or {}).get("test_lines", 0) for r in good)
        per = f"{lines_ / len(good):.1f}" if good else "—"
        passed_self = [r for r in rows if r.get("delivered") and r.get("final_check_ok")]
        hidden = sum(1 for r in passed_self if r.get("failed"))
        checks = [r.get("checks", 0) for r in rows]
        lines.append(f"| {model} | {arm} | {ratio(len(good), len(rows))} | {lines_} | {per} | {med(checks)} | "
                     f"{ratio(hidden, len(passed_self))} |")
    return lines


def navigation_table(rs):
    lines = ["| 집합 | 모델 | 탐색 성공(P·R=1) | 세션당 읽기 호출(중앙값) | 세션당 읽은 바이트(중앙값) | 세션당 총 토큰(중앙값) |",
             "| --- | --- | --- | --- | --- | --- |"]
    for s, model, arm, summary, rows in rs:
        nav = [r for r in rows if r.get("kind") == "navigate"]
        if not nav:
            continue
        ok = sum(1 for r in nav if r.get("success"))
        lines.append(f"| {s} | {model} | {ratio(ok, len(nav))} | {med([r.get('read_calls', 0) for r in nav])} | "
                     f"{med([r.get('read_bytes', 0) for r in nav])} | {med([total_tokens(r) for r in nav])} |")
    return lines


def tool_mix_table(rs):
    lines = ["| 집합 | 팔 | 모델 | 읽기 도구 | 쓰기 도구 | 검사 도구 | 읽기 비율 |",
             "| --- | --- | --- | --- | --- | --- | --- |"]
    for s, model, arm, summary, rows in rs:
        if s != "brb":
            continue
        reads = writes = checks = 0
        for r in rows:
            counts, rd, ck = tools(r)
            reads += rd
            checks += ck
            writes += sum(counts.values()) - rd - ck
        total = reads + writes + checks
        share = f"{reads / total:.0%}" if total else "—"
        lines.append(f"| {s} | {arm} | {model} | {reads} | {writes} | {checks} | {share} |")
    return lines


def tables():
    rs = runs()
    out = ["### A. 결과와 비용(정식 측정 전부)", ""] + outcome_table(rs)
    out += ["", "### B. 테스트 코드와 숨은 결함(BRB, G5·G8·H-ORACLE)", ""] + test_code_table(rs)
    out += ["", "### C. 탐색(m1·m1b 탐색 과제, G4)", ""] + navigation_table(rs)
    out += ["", "### D. 도구 사용의 구성(BRB 세션 합계)", ""] + tool_mix_table(rs)
    files = sorted({f"`bench/{s}/results/`" for s, *_ in rs})
    out += ["", f"원자료: {', '.join(files)}의 결과 파일 {len(rs)}개(요약 행이 있는 것)."]
    return "\n".join(out) + "\n"


def block():
    return BEGIN + "\n\n" + tables() + "\n" + END


def replace(text, new):
    pat = re.compile(re.escape(BEGIN) + r".*?" + re.escape(END), re.S)
    if not pat.search(text):
        raise SystemExit(f"no generated block ({BEGIN!r} ... {END!r})")
    return pat.sub(lambda _: new, text)


def main(argv=None):
    argv = sys.argv[1:] if argv is None else argv
    if argv[:1] == ["tables"]:
        sys.stdout.write(tables())
        return 0
    if len(argv) == 2 and argv[0] in ("--write", "--check"):
        with open(argv[1]) as f:
            text = f.read()
        new = replace(text, block())
        if argv[0] == "--write":
            with open(argv[1], "w") as f:
                f.write(new)
            return 0
        if new != text:
            print(f"{argv[1]}: the generated tables are stale; run `python3 bench/utility.py --write {argv[1]}` "
                  "and review the assessment against them", file=sys.stderr)
            return 1
        return 0
    sys.exit(__doc__)


if __name__ == "__main__":
    sys.exit(main())
