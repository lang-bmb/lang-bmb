#!/usr/bin/env python3
"""BRB-ENCAP harness: type invariants against private fields + tests (ADR-0060
result 6, H-ENCAP; backlog T0-09).

  harness.py manifest [--write]          check (or rewrite) the frozen-input hashes
  harness.py run --arm bmb|rust --agent AGENT [--tasks ID,..] [--jobs N]
                 [--out FILE] [--sessions DIR] [--proptest on|off]
                 [--expect all-pass|weak-caught|gamed]
  harness.py adequacy [--jobs N]          every Rust mutant is caught by the hidden scripts
  harness.py summarize RESULTS.jsonl..   metric table (one column per file) and the H-ENCAP line

A BRB variant: each session starts from a data structure (a type with an
invariant and its operations) and adds one operation. BMB states the
invariant as a type invariant (ADR-0060) the proof gate enforces; Rust keeps
private fields, a documented invariant and tests. The hidden judge
(`judge.py`) runs operation sequences and checks the results and, after
every operation, the invariant on the raw fields.

Sessions run as in BRB (`bench/brb/harness.py`, frozen; loaded here as a
library): the same agent adapter, the same limits (40 tool calls, 64,000
model output tokens per session), the same Rust sandbox and tool server, the
final self-check (BMB `verify`, Rust `cargo test`) after the session.

Agents:
  oracle            the reference operation: 0 defects in both arms (CI)
  weak              an operation that breaks the invariant. Rust: its own
                    tests pass, the hidden judge finds the violation. BMB:
                    the gate rejects it at a type invariant boundary, and the
                    judge's run of the unproved build shows the violation (CI)
  game              BMB only: the weak operation with the invariant predicate
                    weakened; the gate passes, the judge reports the changed
                    invariant and the violation (CI)
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
import sys
import tempfile
import time

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(os.path.dirname(HERE))
BRB = os.path.join(ROOT, "bench", "brb")
TASK_SET = "brb-encap"


def _load(name, path):
    spec = importlib.util.spec_from_file_location(name, path)
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    return mod


# BRB's frozen harness first (it imports its own `judge`), then this set's
# judge under another name.
brb = _load("brb_harness", os.path.join(BRB, "harness.py"))
judge = _load("brb_encap_judge", os.path.join(HERE, "judge.py"))
m1 = brb.m1

ARMS = brb.ARMS
MAX_TOOL_CALLS = brb.MAX_TOOL_CALLS
SESSION_OUTPUT_TOKENS = brb.SESSION_OUTPUT_TOKENS


# ---------- frozen inputs ----------

def read_bytes(path):
    with open(path, "rb") as f:
        return f.read()


def frozen_files():
    """This set's prompts, judge, harness, tasks and seeds, and BRB's MANIFEST
    (the BRB harness, judge, tool server and Rust template this set runs on)."""
    files = ["prompt-bmb.md", "prompt-rust.md", "judge.py", "harness.py", "../brb/MANIFEST"]
    files += sorted("tasks/" + f for f in os.listdir(os.path.join(HERE, "tasks")) if f.endswith(".py"))
    for d, _, fs in sorted(os.walk(os.path.join(HERE, "seed"))):
        files += sorted(os.path.relpath(os.path.join(d, f), HERE) for f in fs)
    return files


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


# ---------- agents ----------

def make_agent(kind, task, arm):
    if kind in ("oracle", "weak", "game"):
        if arm == "bmb":
            code = {"oracle": task.BMB_REF, "weak": task.BMB_WEAK, "game": task.BMB_GAME}[kind]
            if code is None:
                sys.exit(f"{task.ID}: no {kind} solution")
            return m1.ScriptAgent(m1.ops_to_calls([{"op": "put", "unit": task.UNIT, "code": code}]))
        if kind == "game":
            sys.exit("the gaming control is BMB only")
        method = task.RUST_REF if kind == "oracle" else task.RUST_WEAK
        calls = [("write_file", {"path": "src/lib.rs", "content": judge.rust_ref_source(task, method)})]
        if kind == "weak":
            calls.append(("write_file", {"path": f"tests/{task.NAME}.rs", "content": task.RUST_WEAK_TESTS}))
        return m1.ScriptAgent(calls + [("cargo_test", {})])
    sys.path.insert(0, m1.HERE)
    from model_api import make_model_agent  # noqa: E402
    prompt = read_bytes(os.path.join(HERE, f"prompt-{arm}.md")).decode()
    if agent := make_model_agent(kind, prompt, judge.task_prompt(task, arm)):
        return agent
    sys.exit(f"unknown agent {kind}")


# ---------- sessions ----------

def start_workspace(task, arm, tmp, proptest):
    """The session's starting point: (workspace or crate, its seed copy)."""
    files = judge.seed_files(task, arm)
    seed = os.path.join(tmp, "seed")
    judge.write_files(seed, files)
    if arm == "bmb":
        ws = os.path.join(tmp, "ws")
        judge.write_files(ws, files)
    else:
        ws = os.path.join(tmp, "solution")
        brb.make_crate(ws, proptest)
        os.unlink(os.path.join(ws, "src", "lib.rs"))
        judge.write_files(ws, files)
    return ws, seed


def run_session(task, arm, agent_kind, rust_target, proptest, sessions=None):
    tmp = tempfile.mkdtemp(prefix=f"brb-encap-{arm}-{task.ID}-")
    try:
        ws, seed = start_workspace(task, arm, tmp, proptest)
        if arm == "bmb":
            server = m1.Mcp(ws)
            check_tool = "verify"
        else:
            # A private copy of the warmed target directory: every session's
            # crate is named `solution`, and sessions that share one target
            # directory link each other's library into their integration
            # tests (`tests/*.rs`, which the seed has) when they run at once.
            target = os.path.join(tmp, "target")
            shutil.copytree(rust_target, target, symlinks=True)
            server = brb.RustTools(ws, target)
            check_tool = "cargo_test"
        agent = make_agent(agent_kind, task, arm)
        if hasattr(agent, "set_tools"):
            agent.set_tools(server.tools)
        m = {"tool_calls": 0, "tool_seconds": 0.0, "checks": 0, "last_check_ok": None,
             "limit_hit": None, "tool_counts": {}}
        tx = None
        last = []
        calls = []
        t0 = time.monotonic()
        try:
            while True:
                step = agent.next_calls(last)
                if not step:
                    break
                last = []
                for name, args in step:
                    if m["tool_calls"] >= MAX_TOOL_CALLS:
                        m["limit_hit"] = "tool_calls"
                        break
                    m["tool_calls"] += 1
                    m["tool_counts"][name] = m["tool_counts"].get(name, 0) + 1
                    args = {k: (tx if v == "$tx" else v) for k, v in args.items()}
                    t1 = time.monotonic()
                    text, err = server.call(name, args)
                    m["tool_seconds"] += time.monotonic() - t1
                    if name == "tx_begin" and not err:
                        tx = json.loads(text)["tx"]
                    if name == check_tool:
                        m["checks"] += 1
                        m["last_check_ok"] = brb.check_ok(arm, text, err)
                    calls.append({"name": name, "args": args, "error": bool(err), "text": text})
                    last.append((name, args, text, err))
                if m["limit_hit"]:
                    break
                if agent.usage.get("output_tokens", 0) >= SESSION_OUTPUT_TOKENS:
                    m["limit_hit"] = "output_tokens"
                    break
            # The arm's own checker once more on the delivered state (as in
            # BRB): did the gate or the agent's tests pass at the end?
            text, err = server.call(check_tool, {})
            final_check_ok = brb.check_ok(arm, text, err)
        finally:
            server.close()
        final_sources = brb.delivered_sources(ws, arm)
        if arm == "bmb":
            test_code = brb.bmb_test_surface(ws)
        else:
            # The agent's test code: the seed's tests, unchanged, are not its.
            seeded = judge.seed_files(task, arm)
            test_code = {"test_lines": brb.rust_test_lines(
                {p: t for p, t in final_sources.items() if seeded.get(p) != t})}
        session_s = time.monotonic() - t0
        t1 = time.monotonic()
        jdir = os.path.join(tmp, "judge")
        os.makedirs(jdir)
        if arm == "bmb":
            verdict = judge.judge_bmb(task, ws, brb.bmb_bin(), jdir, m1.cache_dir(ws))
        else:
            verdict = judge.judge_rust(task, ws, jdir)
        judge_s = time.monotonic() - t1
        if sessions:
            m1.sessionlog.write(sessions, m1.sessionlog.record(task.ID, agent_kind, agent, calls,
                                                            m1.sessionlog.workspace_changes(seed, ws)))
        success = verdict["delivered"] and verdict["failed"] == 0 and verdict.get("domain_ok", True) is not False
        violated = bool(verdict.get("violation_cases"))
        usage = agent.usage
        return {
            "id": task.ID, "arm": arm, "agent": agent_kind, "success": success, "defect": not success,
            **verdict, **m, "final_check_ok": final_check_ok,
            # H-ENCAP: the arm's own check passed, and a hidden state broke the invariant.
            "passed_and_violated": final_check_ok and violated,
            "final_sources": final_sources, "test_code": test_code,
            "session_seconds": round(session_s, 2), "tool_seconds": round(m["tool_seconds"], 2),
            "judge_seconds": round(judge_s, 2),
            "model_usage": usage,
            "total_tokens": usage.get("input_tokens", 0) + usage.get("output_tokens", 0),
            "agent_settings": getattr(agent, "settings", None),
            "stop": getattr(agent, "stop", None),
        }
    finally:
        shutil.rmtree(tmp, ignore_errors=True)


# ---------- adequacy ----------

def adequacy(task):
    """The hidden scripts catch every Rust mutant of the task."""
    tmp = tempfile.mkdtemp(prefix=f"brb-encap-mut-{task.ID}-")
    try:
        return {"id": task.ID, "mutants": judge.judge_mutants(task, tmp)}
    finally:
        shutil.rmtree(tmp, ignore_errors=True)


# ---------- summary ----------

def passed_violated(t):
    """(sessions whose own check passed and whose run broke the invariant,
    sessions whose own check passed, of the former those whose type or
    invariant differs from the seed's)."""
    passed = [r for r in t if r.get("final_check_ok") is True]
    bad = [r for r in passed if r.get("violation_cases")]
    return len(bad), len(passed), sum(1 for r in bad if r.get("invariant_kept") is False)


def encap_verdict(bmb, rust):
    """The H-ENCAP line (ADR-0060 result 6, pre-registered before any model
    run). Falsified when, in the BMB arm, a violation passes the gate: any
    session whose proof gate passed while a hidden state broke the seed's
    invariant (sessions that changed the invariant count; the line says how
    many). Otherwise supported when the BMB share of such sessions is below
    the Rust share. `bmb` and `rust` are `passed_violated` triples."""
    (bd, bp, bc), (rd, rp, _) = bmb, rust
    if not bp or not rp:
        return "H-ENCAP: 판정 불가(자기 검사를 통과한 과제가 없는 비교군)"
    shares = f"BMB {bd}/{bp} = {bd / bp:.3f}, Rust {rd}/{rp} = {rd / rp:.3f}"
    head = "H-ENCAP (통과 & 숨은 불변식 위반 비율, 반증: BMB 팔에서 위반이 게이트를 통과)"
    if bd:
        return f"{head}: {shares} → 반증(BMB 위반 {bd}건 중 불변식 변경 {bc}건)"
    if rd == 0:
        return f"{head}: {shares} → 판정 불가(Rust 통과 & 위반 0)"
    return f"{head}: {shares} → 지지"


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
        pv = passed_violated(t)
        return [
            ("과제", str(n)),
            ("성공(숨은 판정 전부 통과)", f"{sum(r['success'] for r in t)}/{n}"),
            ("자기 검사 통과(최종 상태, 증명 게이트 / 에이전트 테스트)",
             f"{sum(1 for r in t if r.get('final_check_ok') is True)}/{n}"),
            ("숨은 불변식 위반(실행이 있었던 과제)", str(sum(1 for r in t if r.get("violation_cases")))),
            ("자기 검사 통과 & 숨은 불변식 위반 / 자기 검사 통과", f"{pv[0]}/{pv[1]}"),
            ("그중 불변식 변경(BMB: 타입·술어가 시드와 다름)", str(pv[2]) if bmb else "—"),
            ("게이트 거부: 불변식 경계(BMB, E0823·E0811)",
             str(sum(1 for r in t if r.get("invariant_rejected"))) if bmb else "—"),
            ("미제출(빌드·검증·서명·필드)", str(sum(1 for r in t if not r["delivered"]))),
            ("정의역 위반(BMB: 어떤 상태·인자에서 부를 수 없음)", str(sum(1 for r in t if r.get("domain_ok") is False))),
            ("실패 스크립트 / 스크립트", f"{sum(r['failed'] for r in t)}/{sum(r['cases'] for r in t)}"),
            ("테스트 코드 줄(BMB 남은 테스트 면적 / Rust 에이전트 테스트, 시드 제외)",
             str(sum(brb.test_lines_of(r) or 0 for r in t))),
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
    if sorted(by_arm) != ["bmb", "rust"] or any(len(v) != 1 for v in by_arm.values()):
        out.append("H-ENCAP: 판정 불가(BMB 결과 하나와 Rust 결과 하나를 함께 주어야 함)")
    elif sorted(r["id"] for r in by_arm["bmb"][0]) != sorted(r["id"] for r in by_arm["rust"][0]):
        out.append("H-ENCAP: 판정 불가(두 비교군의 과제 집합이 다름)")
    else:
        out.append(encap_verdict(passed_violated(by_arm["bmb"][0]), passed_violated(by_arm["rust"][0])))
    ids = sorted({r["id"] for _, _, t in runs for r in t})
    out += ["", "| 과제 | " + " | ".join(n for n, _, _ in runs) + " |", "| --- |" + " --- |" * len(runs)]
    for tid in ids:
        cells = []
        for _, _, t in runs:
            r = next((r for r in t if r["id"] == tid), None)
            if r is None:
                cells.append("—")
                continue
            check = "통과" if r.get("final_check_ok") else "실패"
            viol = f", 위반 {r['violation_cases']}" if r.get("violation_cases") else ""
            kept = ", 불변식 변경" if r.get("invariant_kept") is False else ""
            if r["success"]:
                cells.append(f"성공 (검사 {check})")
            elif not r["delivered"]:
                cells.append(f"미제출: {r.get('why', '')[:30]} (검사 {check}{viol}{kept})")
            else:
                kinds = ", ".join(f"{k} {v}" for k, v in r["fail_kinds"].items())
                dom = "정의역 위반; " if r.get("domain_ok") is False else ""
                cells.append(f"결함: {dom}{r['failed']}/{r['cases']} ({kinds}) (검사 {check}{kept})")
        out.append(f"| {tid} | " + " | ".join(cells) + " |")
    return "\n".join(out) + "\n"


# ---------- main ----------

def expect_failures(expect, arm, results):
    """Why `results` miss the `--expect` condition (empty when they meet it)."""
    bad = []
    for r in results:
        if expect == "all-pass" and not (r["success"] and r["final_check_ok"]):
            bad.append(f"{r['id']}: expected the hidden judge and the own check to pass")
        elif expect == "weak-caught" and arm == "rust" and not (
                r["last_check_ok"] is True and r["final_check_ok"] is True and not r["success"]
                and r.get("violation_cases")):
            bad.append(f"{r['id']}: expected own tests to pass and the hidden judge to find a violation")
        elif expect == "weak-caught" and arm == "bmb" and not (
                r["last_check_ok"] is False and r["final_check_ok"] is False and r.get("invariant_rejected")
                and r.get("violation_cases") and r.get("invariant_kept") is True):
            bad.append(f"{r['id']}: expected the gate to reject at a type invariant boundary, "
                       "and the unproved build to show a violation")
        elif expect == "gamed" and not (
                r["final_check_ok"] is True and r.get("invariant_kept") is False and not r["success"]
                and r.get("violation_cases")):
            bad.append(f"{r['id']}: expected the gate to pass with a changed invariant and a hidden violation")
    return bad if results else ["no results"]


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
    rn.add_argument("--expect", choices=["all-pass", "weak-caught", "gamed"])
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
    elif args.cmd == "run" and args.agent == "game":
        ts = [t for t in ts if t.BMB_GAME]
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
            sys.exit("a mutant survived the hidden scripts (or did not build)")
        return

    proptest = args.proptest == "on"
    commit = m1.run_commit()
    log = m1.resultlog.ResultLog(args.out, {"agent": args.agent, "task_set": TASK_SET, "arm": args.arm,
                                            "manifest": manifest_digest(), "commit": commit,
                                            "tasks": [t.ID for t in ts], **m1.condition_fields()})

    def session(t):
        r = run_session(t, args.arm, args.agent, tmp_target, proptest, args.sessions)
        log.add(r)
        return r

    tmp_target = tempfile.mkdtemp(prefix="brb-encap-target-")
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
    pv = passed_violated(results)
    summary = {
        "summary": True, "task_set": TASK_SET, "arm": args.arm, "agent": args.agent, "tasks": n,
        "proptest": proptest if args.arm == "rust" else None,
        "manifest": manifest_digest(), "commit": commit, **m1.condition_fields(),
        "limits": {"tool_calls": MAX_TOOL_CALLS, "session_output_tokens": SESSION_OUTPUT_TOKENS},
        "success": f"{sum(r['success'] for r in results)}/{n}",
        "passed_and_violated": f"{pv[0]}/{pv[1]}",
        "total_tokens": sum(r["total_tokens"] for r in results),
    }
    log.finish(results, summary)
    print(json.dumps(summary, ensure_ascii=False))
    if args.expect and (bad := expect_failures(args.expect, args.arm, results)):
        sys.exit("\n".join(bad))


if __name__ == "__main__":
    main()
