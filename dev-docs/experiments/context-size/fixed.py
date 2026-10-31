#!/usr/bin/env python3
"""The fixed part of every request, before and after the lean sets (backlog 2D-28).

  python3 dev-docs/experiments/context-size/fixed.py [--bmb PATH] > dev-docs/experiments/context-size/fixed.md

Offline: no model is called. The fixed part is what `run.py` §2 counts: the
system prompt (the set's `prompt.md`), the tool definitions as the model is
given them (name, description, input schema of the tools `bmb mcp` serves
with the set's flags, minus its hidden tools) and the mean write-task prompt.
Tokens are the same estimate (`tokest.py`, fitted to Claude counts).
"""
import argparse
import os
import re
import statistics
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import run  # noqa: E402
import tokest  # noqa: E402

PAIRS = [("bvb-write", "bvb-write-lean"), ("m1b-write", "m1b-write-lean")]


def fmt(n):
    return f"{n:,}"


def load(name, extra=()):
    cfg = run.set_config(name)
    tools = run.mcp_tools(cfg["mcp_flags"] + list(extra), set(cfg["hide_tools"]))
    prompt = open(os.path.join(run.BENCH, name, "prompt.md")).read()
    return prompt, tools, run.task_prompts(name)


def sections(prompt):
    """(heading, text) of the prompt: the intro, then each `## ` section."""
    parts = re.split(r"(?m)^(?=## )", prompt)
    return [("(머리)" if not p.startswith("## ") else p.splitlines()[0][3:], p) for p in parts if p]


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--bmb", default=os.path.join(run.ROOT, "target", "debug", "bmb"))
    run.BMB = os.path.abspath(ap.parse_args().bmb)
    tokest.calibration(run.BMB)
    tok = tokest.tokens
    print("# 고정 부분: 원판과 2D-28 판\n")
    print("생성: `python3 dev-docs/experiments/context-size/fixed.py`(모델 호출 없음). 해석은 `README.md` 2D-28 절.\n")

    print("## 1. 판별 합\n")
    print("작성 과제 기준(`run.py` 2절과 같은 셈). 탐색 과제는 하네스의 `answer` 도구 정의가 더해진다(어느 판이든 같은 값).\n")
    print("| 판 | 도구 수 | 시스템 프롬프트 B / 토큰 | 도구 정의 B / 토큰 | 과제 지시문 평균 토큰 | 합 토큰 | 원판 대비 |")
    print("| --- | ---: | ---: | ---: | ---: | ---: | ---: |")
    rows = []
    for src, lean in PAIRS:
        rows += [(src, src, ()), (lean, lean, ()), (f"{lean} + `--compact`", lean, ("--compact",))]
    base = {}
    for label, name, extra in rows:
        prompt, tools, tasks = load(name, extra)
        tt = run.tools_text(tools)
        task = round(statistics.mean(tok(p) for p in tasks))
        total = tok(prompt) + tok(tt) + task
        src = name.replace("-lean", "")
        base.setdefault(src, total)
        print(f"| {label} | {len(tools)} | {fmt(len(prompt.encode()))} / {fmt(tok(prompt))} | "
              f"{fmt(len(tt.encode()))} / {fmt(tok(tt))} | {fmt(task)} | {fmt(total)} | "
              f"{(total - base[src]) / base[src]:+.0%} |")
    print()

    print("## 2. 시스템 프롬프트 절\n")
    print("| 절 | `bvb-write` B / 토큰 | `bvb-write-lean` B / 토큰 |")
    print("| --- | ---: | ---: |")
    a = dict(sections(load("bvb-write")[0]))
    b = dict(sections(load("bvb-write-lean")[0]))
    for head in list(a) + [h for h in b if h not in a]:
        cell = (lambda d: f"{fmt(len(d[head].encode()))} / {fmt(tok(d[head]))}" if head in d else "—")
        print(f"| {head} | {cell(a)} | {cell(b)} |")
    print()

    print("## 3. 도구 정의\n")
    print("도구마다 모델에 주는 꼴(이름, 설명, 입력 스키마)의 JSON 한 줄.\n")
    print("| 도구 | `bvb-write` B / 토큰 | `bvb-write-lean` B / 토큰 |")
    print("| --- | ---: | ---: |")
    a = {t["name"]: run.tools_text([t]) for t in load("bvb-write")[1]}
    b = {t["name"]: run.tools_text([t]) for t in load("bvb-write-lean")[1]}
    for name in a:
        cell = (lambda d: f"{fmt(len(d[name].encode()))} / {fmt(tok(d[name]))}" if name in d else "숨김")
        print(f"| {name} | {cell(a)} | {cell(b)} |")
    print()


if __name__ == "__main__":
    main()
