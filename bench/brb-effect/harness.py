#!/usr/bin/env python3
"""BRB-EFFECT harness: tasks with output and state (H-EFFECT, ADR-0078;
backlog T1-43).

  harness.py manifest [--write]          check (or rewrite) the frozen-input hashes
  harness.py run --arm bmb|rust --agent AGENT [--tasks ID,..] [--jobs N]
                 [--out FILE] [--sessions DIR] [--proptest on|off]
                 [--expect all-pass|weak-fail]
  harness.py adequacy [--jobs N]          every Rust mutant is caught by the hidden cases
  harness.py summarize RESULTS.jsonl..   metric table (one column per file) and the H-EFFECT line

A BRB variant: each task prints lines (`print_i64` in BMB, `println!` in
Rust) and keeps state across the elements of its input, and the hidden judge
(`judge.py`) checks the printed lines as well as the result. Sessions run
exactly as in BRB (`bench/brb/harness.py`, frozen, loaded here as a library
with this set's judge): the same prompts, agents, limits, Rust sandbox and
tool servers, and the same final self-check.

The H-EFFECT measure of a session: BMB, the remaining test surface of the
delivered workspace (`bmb query tests`: executable definitions that still
need tests, of all executable definitions, and their lines); Rust, the
agent's test code lines.

Agents:
  oracle            the reference solution: 0 defects in both arms (CI)
  weak              a plausible wrong solution: its own check passes (BMB
                    proof, Rust tests), the hidden judge finds the defect (CI)
  openai:MODEL      a real model, OpenAI-compatible Chat Completions
  anthropic:MODEL   a real model, Anthropic Messages API
                    (models and endpoints: ADR-0066, bench/m1/model_api.py)

Standard library only.
"""
import argparse
import concurrent.futures
import hashlib
import importlib.util
import json
import os
import shutil
import subprocess
import sys
import tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(os.path.dirname(HERE))
BRB = os.path.join(ROOT, "bench", "brb")
TASK_SET = "brb-effect"


def _load(name, path):
    spec = importlib.util.spec_from_file_location(name, path)
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    return mod


# BRB's frozen harness first (it imports its own `judge`), then this set's
# judge, which BRB's session loop uses through the module's `judge` global.
brb = _load("brb_effect_brb_harness", os.path.join(BRB, "harness.py"))
judge = _load("brb_effect_judge", os.path.join(HERE, "judge.py"))
brb.judge = judge
m1 = brb.m1

ARMS = brb.ARMS
MAX_TOOL_CALLS = brb.MAX_TOOL_CALLS
SESSION_OUTPUT_TOKENS = brb.SESSION_OUTPUT_TOKENS
# H-EFFECT (ADR-0078, registered before any model run): the remaining test
# surface is at most this share of the executable definitions, and the test
# code lines at most this share of the Rust arm's.
AREA_MAX = 0.20
LINES_MAX = 1 / 3
# The reference solutions' area (test-needing / executable definitions),
# reported beside the verdict as an auxiliary measure, never part of it
# (decision 2026-11-02: the 0.20 criterion stays). The gate checks the
# references still give it (`check.sh`, bench brb-effect controls).
REFERENCE_AREA = (4, 19)


def bmb_test_surface(ws):
    """The remaining test surface of the delivered workspace (`bmb query
    tests`): its summary row with the executable definitions, or None when
    the query fails."""
    r = subprocess.run([brb.bmb_bin(), "query", "--cache", m1.cache_dir(ws), "tests", "--", ws],
                       capture_output=True, text=True)
    rows = [json.loads(line) for line in r.stdout.splitlines() if line.startswith("{")]
    summary = next((x for x in rows if x.get("summary")), None)
    if r.returncode != 0 or summary is None:
        return None
    return {k: summary[k] for k in ("exec_defs", "test_needed_defs", "io_defs", "unproved_defs",
                                     "test_sites", "test_lines")}


brb.bmb_test_surface = bmb_test_surface


# ---------- frozen inputs ----------

def read_bytes(path):
    with open(path, "rb") as f:
        return f.read()


def frozen_files():
    """This set's judge, harness and tasks, and BRB's MANIFEST (the BRB
    harness, judge, prompts, tool server and Rust template this set runs on)."""
    files = ["judge.py", "harness.py", "../brb/MANIFEST"]
    return files + sorted("tasks/" + f for f in os.listdir(os.path.join(HERE, "tasks")) if f.endswith(".py"))


def manifest_text():
    return "".join(f"{hashlib.sha256(read_bytes(os.path.join(HERE, f))).hexdigest()}  {f}\n"
                   for f in frozen_files())


def manifest_digest():
    return hashlib.sha256(read_bytes(os.path.join(HERE, "MANIFEST"))).hexdigest()


def check_manifest():
    path = os.path.join(HERE, "MANIFEST")
    if not os.path.exists(path) or read_bytes(path).decode() != manifest_text():
        sys.exit("frozen inputs differ from MANIFEST: refusing to run (ADR-0021 §1)")


def tasks():
    return judge.load_tasks(os.path.join(HERE, "tasks"))


def adequacy(task):
    tmp = tempfile.mkdtemp(prefix=f"brb-effect-mut-{task.ID}-")
    try:
        return {"id": task.ID, "mutants": judge.judge_mutants(task, tmp)}
    finally:
        shutil.rmtree(tmp, ignore_errors=True)


# ---------- summary ----------

def surface_of(t):
    """(test-needing executable definitions, executable definitions, test
    lines) over the BMB sessions `t` that passed the hidden judge; None when
    a session's surface is unknown."""
    passed = [r for r in t if r["success"]]
    tc = [r.get("test_code") or {} for r in passed]
    if any("exec_defs" not in c for c in tc):
        return None
    return (sum(c["test_needed_defs"] for c in tc), sum(c["exec_defs"] for c in tc),
            sum(c["test_lines"] for c in tc), len(passed))


def effect_verdict(bmb, rust):
    """The H-EFFECT line for one model's BMB and Rust results (ADR-0078):
    confirmed for this model when the remaining test surface of the BMB
    sessions that passed the hidden judge is at most 20% of their executable
    definitions and their test lines at most 1/3 of the Rust sessions' that
    passed; falsified when either is exceeded. The hypothesis is confirmed
    only when both models' lines say so."""
    head = "H-EFFECT (남은 테스트 면적 ≤ 실행 정의의 20%, 테스트 코드 줄 ≤ Rust의 1/3)"
    s = surface_of(bmb)
    if s is None:
        return f"{head}: 판정 불가(BMB 남은 테스트 면적을 모르는 세션)"
    need, defs, blines, bn = s
    rlines, rn = brb.passed_lines(rust)
    if not bn or not rn:
        return f"{head}: 판정 불가(숨은 판정을 통과한 세션이 없는 비교군)"
    if not defs:
        return f"{head}: 판정 불가(실행 정의 0)"
    area = need / defs
    parts = [f"면적 {need}/{defs} = {area:.3f} (BMB 통과 {bn})"]
    if rlines == 0:
        parts.append(f"줄 BMB {blines} / Rust 0 (Rust 통과 {rn})")
        verdict = "반증(면적)" if area > AREA_MAX else "판정 불가(Rust 테스트 0줄)"
        return f"{head}: " + ", ".join(parts) + f" → {verdict}" + aux_line(area)
    ratio = blines / rlines
    parts.append(f"줄 BMB {blines} / Rust {rlines} = {ratio:.3f} (Rust 통과 {rn})")
    over = [n for n, bad in (("면적", area > AREA_MAX), ("줄", ratio > LINES_MAX)) if bad]
    verdict = f"반증({'·'.join(over)})" if over else "이 모델에서 확정 조건 충족"
    return f"{head}: " + ", ".join(parts) + f" → {verdict}" + aux_line(area)


def aux_line(area):
    """The auxiliary measure: the area against the reference solutions'."""
    rn, rd = REFERENCE_AREA
    return f"\n보조(판정에 쓰지 않음): 기준 해법 면적 {rn}/{rd} = {rn / rd:.3f}, 대비 {area / (rn / rd):.2f}배"


def summarize(paths):
    runs = []
    for p in paths:
        recs = [json.loads(line) for line in read_bytes(p).decode().splitlines() if line.strip()]
        head = next((r for r in recs if r.get("summary")), {})
        runs.append((os.path.basename(p), head, [r for r in recs if not r.get("summary")]))
    if len({h.get("manifest", "") for _, h, _ in runs}) > 1:
        sys.exit("results from different frozen inputs (MANIFEST) are not compared (ADR-0021 §1)")

    def metrics(t):
        n = len(t)
        usage = [r.get("model_usage") or {} for r in t]
        bmb = bool(t) and t[0].get("arm") == "bmb"
        s = surface_of(t) if bmb else None
        return [
            ("과제", str(n)),
            ("성공(숨은 판정 전부 통과)", f"{sum(r['success'] for r in t)}/{n}"),
            ("미제출(빌드·검증·시그니처 실패)", str(sum(1 for r in t if not r["delivered"]))),
            ("정의역 위반(BMB: requires가 정의역보다 강함)", str(sum(1 for r in t if r.get("domain_ok") is False))),
            ("실패 케이스 / 케이스", f"{sum(r['failed'] for r in t)}/{sum(r['cases'] for r in t)}"),
            ("그중 출력 줄만 틀림(반환값은 맞음)", str(sum(r.get("wrong_output", 0) for r in t))),
            ("자기 검사 통과(최종 상태, 증명 게이트 / 에이전트 테스트)",
             f"{sum(1 for r in t if r.get('final_check_ok') is True)}/{n}"),
            ("자기 검사 통과 & 숨은 실패 / 자기 검사 통과", "{}/{}".format(*brb.passed_defective(t))),
            ("남은 테스트 면적(BMB 통과 세션: 테스트 필요 / 실행 정의)",
             (f"{s[0]}/{s[1]}" if s else "—") if bmb else "—"),
            ("테스트 코드 줄(통과 세션; BMB 남은 면적 / Rust 에이전트 테스트)", str(brb.passed_lines(t)[0])),
            ("테스트 코드 줄(전체)", str(sum(brb.test_lines_of(r) or 0 for r in t))),
            ("총 모델 토큰(입력+출력)", str(sum(r["total_tokens"] for r in t))),
            ("모델 출력 토큰", str(sum(u.get("output_tokens", 0) for u in usage))),
            ("도구 호출(합)", str(sum(r["tool_calls"] for r in t))),
            ("검사 도구 호출(verify / cargo_test)", str(sum(r["checks"] for r in t))),
            ("한도 도달 과제", str(sum(1 for r in t if r["limit_hit"]))),
            ("세션 벽시계(초, 합)", f"{sum(r['session_seconds'] for r in t):.1f}"),
        ]

    out = ["| 지표 | " + " | ".join(n for n, _, _ in runs) + " |", "| --- |" + " --- |" * len(runs)]
    for key, label in (("arm", "비교군"), ("agent", "에이전트"), ("proptest", "proptest")):
        out.append(f"| {label} | " + " | ".join("—" if h.get(key) is None else str(h[key]) for _, h, _ in runs) + " |")
    out.append("| 컴파일러 커밋 | " + " | ".join(str(h.get("commit", "—"))[:12] for _, h, _ in runs) + " |")
    cols = [metrics(t) for _, _, t in runs]
    for i, (label, _) in enumerate(cols[0]):
        out.append(f"| {label} | " + " | ".join(c[i][1] for c in cols) + " |")
    by_arm = {}
    for _, h, t in runs:
        by_arm.setdefault(h.get("arm"), []).append(t)
    out.append("")
    if sorted(by_arm) == ["bmb", "rust"] and all(len(v) == 1 for v in by_arm.values()):
        b, r = by_arm["bmb"][0], by_arm["rust"][0]
        if sorted(x["id"] for x in b) != sorted(x["id"] for x in r):
            out.append("H-EFFECT: 판정 불가(두 비교군의 과제 집합이 다름)")
        else:
            out.append(effect_verdict(b, r))
    else:
        out.append("H-EFFECT: 판정 불가(BMB 결과 하나와 Rust 결과 하나를 함께 주어야 함)")
    ids = sorted({r["id"] for _, _, t in runs for r in t})
    out += ["", "| 과제 | " + " | ".join(n for n, _, _ in runs) + " |", "| --- |" + " --- |" * len(runs)]
    for tid in ids:
        cells = []
        for _, _, t in runs:
            r = next((r for r in t if r["id"] == tid), None)
            if r is None:
                cells.append("—")
                continue
            tc = r.get("test_code") or {}
            area = f" (면적 {tc['test_needed_defs']}/{tc['exec_defs']})" if "exec_defs" in tc else ""
            if r["success"]:
                cells.append("성공" + area)
            elif not r["delivered"]:
                cells.append(f"미제출: {r.get('why', '')[:40]}")
            else:
                kinds = ", ".join(f"{k} {v}" for k, v in r["fail_kinds"].items())
                dom = "정의역 위반; " if r.get("domain_ok") is False else ""
                cells.append(f"결함: {dom}{r['failed']}/{r['cases']} ({kinds})" + area)
        out.append(f"| {tid} | " + " | ".join(cells) + " |")
    return "\n".join(out) + "\n"


# ---------- main ----------

def main():
    ap = argparse.ArgumentParser()
    sub = ap.add_subparsers(dest="cmd", required=True)
    mf = sub.add_parser("manifest")
    mf.add_argument("--write", action="store_true")
    rn = sub.add_parser("run")
    rn.add_argument("--arm", choices=ARMS, required=True)
    rn.add_argument("--agent", required=True)
    rn.add_argument("--tasks")
    rn.add_argument("--out")
    rn.add_argument("--expect", choices=["all-pass", "weak-fail"])
    rn.add_argument("--jobs", type=int, default=1)
    rn.add_argument("--sessions", help="directory for session records (backlog INF-17)")
    rn.add_argument("--proptest", choices=["on", "off"], default="on")
    ad = sub.add_parser("adequacy")
    ad.add_argument("--jobs", type=int, default=1)
    ad.add_argument("--tasks")
    sm = sub.add_parser("summarize")
    sm.add_argument("results", nargs="+")
    args = ap.parse_args()

    if args.cmd == "summarize":
        sys.stdout.write(summarize(args.results))
        return
    if args.cmd == "manifest":
        if args.write:
            with open(os.path.join(HERE, "MANIFEST"), "w") as f:
                f.write(manifest_text())
        else:
            check_manifest()
            print("MANIFEST ok")
        return

    check_manifest()
    ts = tasks()
    if args.tasks:
        wanted = set(args.tasks.split(","))
        ts = [t for t in ts if t.ID in wanted]
    pool = concurrent.futures.ThreadPoolExecutor(max_workers=max(1, args.jobs))

    if args.cmd == "adequacy":
        res = list(pool.map(adequacy, ts))
        bad = 0
        for r in res:
            print(json.dumps(r, ensure_ascii=False))
            bad += sum(1 for m in r["mutants"] if not m["killed"])
        total = sum(len(r["mutants"]) for r in res)
        print(json.dumps({"summary": True, "tasks": len(res), "mutants": total, "survived": bad}))
        if bad or not total:
            sys.exit("a mutant survived the hidden cases (or did not build)")
        return

    proptest = args.proptest == "on"
    commit = m1.run_commit()
    log = m1.resultlog.ResultLog(args.out, {"agent": args.agent, "task_set": TASK_SET, "arm": args.arm,
                                            "manifest": manifest_digest(), "commit": commit,
                                            "tasks": [t.ID for t in ts], **m1.condition_fields()})

    def session(t):
        r = brb.run_session(t, args.arm, args.agent, tmp_target, proptest, args.sessions)
        log.add(r)
        return r

    tmp_target = tempfile.mkdtemp(prefix="brb-effect-target-")
    try:
        if args.arm == "rust":
            brb.warm_rust(tmp_target, proptest)
        list(pool.map(session, [t for t in ts if not log.done(t.ID)]))
    finally:
        shutil.rmtree(tmp_target, ignore_errors=True)
    results = [log.rows[t.ID] for t in ts]
    for r in results:
        print(json.dumps(r, ensure_ascii=False))
    n = len(results)
    summary = {
        "summary": True, "task_set": TASK_SET, "arm": args.arm, "agent": args.agent, "tasks": n,
        "proptest": proptest if args.arm == "rust" else None,
        "manifest": manifest_digest(), "commit": commit, **m1.condition_fields(),
        "limits": {"tool_calls": MAX_TOOL_CALLS, "session_output_tokens": SESSION_OUTPUT_TOKENS,
                   "session_output_counts": "visible (ADR-0076)"},
        "success": f"{sum(r['success'] for r in results)}/{n}",
        "total_tokens": sum(r["total_tokens"] for r in results),
    }
    log.finish(results, summary)
    print(json.dumps(summary, ensure_ascii=False))
    if args.expect == "all-pass" and not all(r["success"] and r["final_check_ok"] for r in results):
        sys.exit("expected every task to pass the hidden judge and its own checker")
    if args.expect == "weak-fail":
        if not results or any(r["success"] for r in results):
            sys.exit("expected the hidden judge to find a defect in every weak solution")
        if any(r["last_check_ok"] is not True or r["final_check_ok"] is not True for r in results):
            sys.exit("expected every weak solution to pass its own checker (verify / cargo test)")


if __name__ == "__main__":
    main()
