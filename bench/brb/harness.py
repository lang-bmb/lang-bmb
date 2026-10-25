#!/usr/bin/env python3
"""BRB harness: BMB+proof against Rust+tests (ADR-0034 §2, G5; backlog 2H-03).

  harness.py manifest [--write]          check (or rewrite) the frozen-input hashes
  harness.py run --arm bmb|rust --agent AGENT [--tasks ID,..] [--jobs N]
                 [--out FILE] [--expect all-pass|weak-fail] [--proptest on|off]
  harness.py adequacy [--jobs N]          every Rust mutant is killed by the hidden cases
  harness.py summarize RESULTS.jsonl..   metric table (one column per file), the G5, H-ORACLE and G8 lines

Both arms get the same task statement, the same agent adapter, the same
limits (tool calls, model output tokens per session) and a checker (BMB:
`verify` with counterexamples; Rust: compiler errors and `cargo test`). The
hidden judge (`judge.py`) evaluates each session's final implementation by
behaviour, the same way in both arms.

Agents:
  oracle            the task's reference solution (BMB through MCP; Rust by
                    writing src/lib.rs and running cargo test): 0 defects (CI)
  weak              a plausible but wrong solution that passes its own checker
                    (BMB: verifies with a weak contract; Rust: its own tests
                    pass): the hidden judge must find defects (CI)
  anthropic:MODEL   a real model through the Anthropic Messages API (paid; needs
                    ANTHROPIC_API_KEY and human approval, AUTONOMY.md §4)

Standard library only.
"""
import argparse
import concurrent.futures
import hashlib
import importlib.util
import json
import os
import re
import shutil
import subprocess
import sys
import tempfile
import time

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(os.path.dirname(HERE))
M1 = os.path.join(ROOT, "bench", "m1")
sys.path.insert(0, HERE)
import judge  # noqa: E402
from rust_tools import RustTools  # noqa: E402


def _m1():
    spec = importlib.util.spec_from_file_location("m1_harness", os.path.join(M1, "harness.py"))
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    return mod


m1 = _m1()

MAX_TOOL_CALLS = m1.MAX_TOOL_CALLS  # 40, ADR-0021 §3
# Model output tokens per session, the same in both arms (ADR-0034 §2 fairness).
# Checked after every model turn; the session stops once it is reached.
SESSION_OUTPUT_TOKENS = 64000
ARMS = ("bmb", "rust")
TEMPLATE = os.path.join(HERE, "rust-template")


def bmb_bin():
    return m1.bmb_bin()


# ---------- frozen inputs ----------

FROZEN = ["prompt-bmb.md", "prompt-rust.md", "cases.py", "judge.py", "rust_tools.py", "harness.py",
          "rust-template/Cargo.toml", "rust-template/Cargo.lock", "rust-template/src/lib.rs"]


def read_bytes(path):
    with open(path, "rb") as f:
        return f.read()


def frozen_files():
    tdir = os.path.join(HERE, "tasks")
    return FROZEN + sorted("tasks/" + f for f in os.listdir(tdir) if f.endswith(".py"))


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


# ---------- Rust sandbox ----------

def make_crate(path, proptest):
    """The session's sandbox crate: the template, with or without proptest."""
    shutil.copytree(TEMPLATE, path, ignore=shutil.ignore_patterns("target"))
    if not proptest:
        toml = read_bytes(os.path.join(path, "Cargo.toml")).decode()
        head, _, rest = toml.partition("[dev-dependencies]")
        rest = rest.split("\n[", 1)
        toml = head + ("[" + rest[1] if len(rest) > 1 else "")
        with open(os.path.join(path, "Cargo.toml"), "w") as f:
            f.write(toml)
        os.unlink(os.path.join(path, "Cargo.lock"))


def warm_rust(target_dir, proptest):
    """Fetches the template's dependencies (the only network use) and builds
    them once, before any session, so sessions run offline and the first one
    does not pay for compiling proptest."""
    tmp = tempfile.mkdtemp(prefix="brb-warm-")
    try:
        crate = os.path.join(tmp, "solution")
        make_crate(crate, proptest)
        env = {**os.environ, "CARGO_TARGET_DIR": target_dir}
        if proptest and subprocess.run(["cargo", "fetch", "--locked", "--offline"], cwd=crate, env=env,
                                       capture_output=True).returncode != 0:
            subprocess.run(["cargo", "fetch", "--locked"], cwd=crate, env=env, check=True, capture_output=True)
        subprocess.run(["cargo", "test", "--offline", "--no-run", "--quiet"], cwd=crate, env=env,
                       check=True, capture_output=True)
    finally:
        shutil.rmtree(tmp, ignore_errors=True)


# ---------- agents ----------

def make_agent(kind, task, arm):
    if kind in ("oracle", "weak"):
        if arm == "bmb":
            code = task.BMB_REF if kind == "oracle" else task.BMB_WEAK
            return m1.ScriptAgent(m1.ops_to_calls([{"op": "unit", "path": "tasks", "summary": "BRB task"},
                                                   {"op": "put", "unit": "tasks", "code": code}]))
        code = task.RUST_REF if kind == "oracle" else task.RUST_WEAK
        return m1.ScriptAgent([("write_file", {"path": "src/lib.rs", "content": code}), ("cargo_test", {})])
    if kind.startswith("anthropic:"):
        sys.path.insert(0, M1)
        from anthropic_agent import AnthropicAgent  # noqa: E402
        prompt = read_bytes(os.path.join(HERE, f"prompt-{arm}.md")).decode()
        return AnthropicAgent(kind.split(":", 1)[1], prompt, judge.task_prompt(task, arm))
    sys.exit(f"unknown agent {kind}")


# ---------- sessions ----------

def run_session(task, arm, agent_kind, rust_target, proptest):
    tmp = tempfile.mkdtemp(prefix=f"brb-{arm}-{task.ID}-")
    try:
        if arm == "bmb":
            ws = os.path.join(tmp, "ws")
            os.makedirs(os.path.join(ws, "src"))
            server = m1.Mcp(ws)
            check_tool = "verify"
        else:
            ws = os.path.join(tmp, "solution")
            make_crate(ws, proptest)
            server = RustTools(ws, rust_target)
            check_tool = "cargo_test"
        agent = make_agent(agent_kind, task, arm)
        if hasattr(agent, "set_tools"):
            agent.set_tools(server.tools)
        m = {"tool_calls": 0, "tool_seconds": 0.0, "checks": 0, "last_check_ok": None,
             "limit_hit": None, "tool_counts": {}}
        tx = None
        last = []
        t0 = time.monotonic()
        try:
            while True:
                calls = agent.next_calls(last)
                if not calls:
                    break
                last = []
                for name, args in calls:
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
                        m["last_check_ok"] = check_ok(arm, text, err)
                    last.append((name, args, text, err))
                if m["limit_hit"]:
                    break
                if agent.usage.get("output_tokens", 0) >= SESSION_OUTPUT_TOKENS:
                    m["limit_hit"] = "output_tokens"
                    break
            # The arm's own checker once more on the delivered state (backlog
            # 2H-04, H-ORACLE): did the gate or the agent's tests pass at the
            # end? Not counted as an agent call.
            text, err = server.call(check_tool, {})
            final_check_ok = check_ok(arm, text, err)
        finally:
            server.close()
        # The delivered sources, so that later metrics (G8 test code, H-REUSE)
        # are computed from this run instead of a new paid one.
        final_sources = delivered_sources(ws, arm)
        test_code = bmb_test_surface(ws) if arm == "bmb" else {"test_lines": rust_test_lines(final_sources)}
        session_s = time.monotonic() - t0
        t1 = time.monotonic()
        jdir = os.path.join(tmp, "judge")
        os.makedirs(jdir)
        if arm == "bmb":
            verdict = judge.judge_bmb(task, ws, bmb_bin(), jdir, m1.cache_dir(ws))
        else:
            verdict = judge.judge_rust(task, ws, jdir)
        judge_s = time.monotonic() - t1
        success = verdict["delivered"] and verdict["failed"] == 0 and verdict.get("domain_ok", True) is not False
        usage = agent.usage
        return {
            "id": task.ID, "arm": arm, "agent": agent_kind, "success": success, "defect": not success,
            **verdict, **m, "final_check_ok": final_check_ok, "final_sources": final_sources,
            "test_code": test_code,
            "session_seconds": round(session_s, 2), "tool_seconds": round(m["tool_seconds"], 2),
            "judge_seconds": round(judge_s, 2),
            "model_usage": usage,
            "total_tokens": usage.get("input_tokens", 0) + usage.get("output_tokens", 0),
            "agent_settings": getattr(agent, "settings", None),
            "stop": getattr(agent, "stop", None),
        }
    finally:
        shutil.rmtree(tmp, ignore_errors=True)


def delivered_sources(ws, arm):
    """{relative path: text} of the files the agent could write: BMB `src/*.bmb`,
    Rust `src/**.rs` and `tests/**.rs`."""
    out = {}
    for top in ("src",) if arm == "bmb" else ("src", "tests"):
        for d, _, fs in os.walk(os.path.join(ws, top)):
            for f in sorted(fs):
                if f.endswith(".bmb" if arm == "bmb" else ".rs"):
                    path = os.path.join(d, f)
                    out[os.path.relpath(path, ws)] = read_bytes(path).decode("utf-8", "replace")
    return dict(sorted(out.items()))


def bmb_test_surface(ws):
    """The remaining test surface of the delivered workspace (`bmb query tests`,
    backlog 2D-21): its summary row, or None when the query fails."""
    r = subprocess.run([bmb_bin(), "query", "--cache", m1.cache_dir(ws), "tests", "--", ws],
                       capture_output=True, text=True)
    rows = [json.loads(line) for line in r.stdout.splitlines() if line.startswith("{")]
    summary = next((x for x in rows if x.get("summary")), None)
    if r.returncode != 0 or summary is None:
        return None
    return {k: summary[k] for k in ("test_needed_defs", "test_sites", "test_lines")}


def code_lines(text):
    """Lines with code: neither blank nor only a `//` comment (block comments
    count as code; agents rarely write them in tests)."""
    return sum(1 for line in text.splitlines() if line.strip() and not line.strip().startswith("//"))


def cfg_test_items(text):
    """The text of every item under `#[cfg(test)]` (module or function), up
    to its matching brace. Braces inside string, char and comment text are
    skipped."""
    out = []
    for m in re.finditer(r"#\[cfg\(test\)\]", text):
        i, depth = m.end(), 0
        while i < len(text):
            c = text[i]
            if text.startswith("//", i):
                i = text.find("\n", i)
                i = len(text) if i < 0 else i
                continue
            if text.startswith("/*", i):
                j = text.find("*/", i + 2)
                i = len(text) if j < 0 else j + 2
                continue
            if c == '"':
                i += 1
                while i < len(text) and text[i] != '"':
                    i += 2 if text[i] == "\\" else 1
            elif c == "'" and re.match(r"'(\\.|[^\\'])'", text[i:i + 4]):
                i += text[i + 1:].index("'") + 1
            elif c == "{":
                depth += 1
            elif c == "}":
                depth -= 1
                if depth == 0:
                    out.append(text[m.start():i + 1])
                    break
            elif c == ";" and depth == 0:
                break
            i += 1
    return out


def rust_test_lines(sources):
    """Test code lines of a delivered Rust crate (G8, backlog 2H-06): every
    file under `tests/` and every `#[cfg(test)]` item under `src/`."""
    n = 0
    for path, text in sources.items():
        if path.startswith("tests" + os.sep) or path.startswith("tests/"):
            n += code_lines(text)
        else:
            n += sum(code_lines(item) for item in cfg_test_items(text))
    return n


def check_ok(arm, text, err):
    """Did the arm's checker pass? BMB: `verify` reports no failed definition
    and no error diagnostic; Rust: `cargo test` exits 0."""
    if err:
        return False
    if arm == "rust":
        return text.startswith("exit status: 0\n")
    rows = m1.rows(text)
    return bool(rows) and all(r.get("status", "verified") == "verified" and r.get("severity") != "error"
                              for r in rows)


# ---------- adequacy ----------

def adequacy(task):
    """The hidden cases kill every Rust mutant of the task."""
    tmp = tempfile.mkdtemp(prefix=f"brb-mut-{task.ID}-")
    try:
        return {"id": task.ID, "mutants": judge.judge_mutants(task, tmp)}
    finally:
        shutil.rmtree(tmp, ignore_errors=True)


# ---------- summary ----------

def g5_verdict(bmb, rust):
    """The G5 line (ADR-0034 §1, provisional M2 thresholds): defect-rate ratio
    BMB/Rust <= 1/2 and total-token ratio BMB/Rust <= 2. `bmb` and `rust` are
    (defects, tasks, total tokens)."""
    (bd, bn, bt), (rd, rn, rt) = bmb, rust
    if not bn or not rn:
        return "G5: 판정 불가(과제 없음)"
    br, rr = bd / bn, rd / rn
    if rr == 0:
        dr, dpass = ("0/0" if br == 0 else "∞"), None
    else:
        dr, dpass = f"{br / rr:.3f}", br / rr <= 0.5
    if rt == 0:
        tr, tpass = ("0/0" if bt == 0 else "∞"), None
    else:
        tr, tpass = f"{bt / rt:.3f}", bt / rt <= 2
    if dpass is None or tpass is None:
        why = "Rust 결함 0" if dpass is None else "Rust 토큰 0"
        verdict = f"판정 불가({why})"
    else:
        verdict = "통과" if dpass and tpass else "미달"
    return f"G5 (잠정 기준: 결함률 비 ≤ 1/2, 총 토큰 비 ≤ 2): 결함률 비 {dr}, 총 토큰 비 {tr} → {verdict}"


def oracle_verdict(bmb, rust):
    """The H-ORACLE line (ADR-0040 §4, pre-registered before any model run):
    among tasks whose own checker passes on the delivered state (BMB: the proof
    gate; Rust: the agent's tests), the share with a hidden defect. Falsified
    when the BMB share is at least the Rust share. `bmb` and `rust` are
    (passed and defective, passed); undecided when no task passed its checker
    in one arm, or when no Rust task passed with a defect (nothing to beat)."""
    (bd, bp), (rd, rp) = bmb, rust
    if not bp or not rp:
        return "H-ORACLE: 판정 불가(자기 검사를 통과한 과제가 없는 비교군)"
    shares = f"BMB {bd}/{bp} = {bd / bp:.3f}, Rust {rd}/{rp} = {rd / rp:.3f}"
    if rd == 0:
        return f"H-ORACLE (통과 & 숨은 실패 비율): {shares} → 판정 불가(Rust 통과 & 숨은 실패 0)"
    verdict = "반증" if bd / bp >= rd / rp else "지지"
    return f"H-ORACLE (통과 & 숨은 실패 비율, 반증: BMB ≥ Rust): {shares} → {verdict}"


def g8_verdict(bmb, rust):
    """The G8 line (ADR-0041, backlog 2H-06): test code over the tasks that pass
    the hidden judge. BMB counts the remaining test surface the compiler
    reports (`query tests` lines), Rust the agent's test code lines. The M3
    target is provisional (1/10); M2 only reports. `bmb` and `rust` are
    (lines, passed tasks)."""
    (bl, bn), (rl, rn) = bmb, rust
    head = f"G8 (테스트 코드 줄, 숨은 판정 통과 과제): BMB {bl}줄/{bn}과제, Rust {rl}줄/{rn}과제"
    if not bn or not rn:
        return head + " → 판정 불가(통과 과제가 없는 비교군)"
    if rl == 0:
        return head + " → 판정 불가(Rust 테스트 0줄)"
    return head + f" → 비 {bl / rl:.3f}(잠정 M3 기준 ≤ 1/10: {'충족' if bl / rl <= 0.1 else '미달'})"


def test_lines_of(r):
    """A result's test code lines, None when unknown (older results, a failed
    query)."""
    tc = r.get("test_code") or {}
    return tc.get("test_lines")


def passed_lines(t):
    passed = [r for r in t if r["success"]]
    return sum(test_lines_of(r) or 0 for r in passed), len(passed)


def passed_defective(t):
    """(tasks whose final self-check passed with a hidden defect, tasks whose
    final self-check passed); results without the field count as not passed."""
    passed = [r for r in t if r.get("final_check_ok") is True]
    return sum(r["defect"] for r in passed), len(passed)


def summarize(paths):
    runs = []
    for p in paths:
        recs = [json.loads(line) for line in read_bytes(p).decode().splitlines() if line.strip()]
        head = next((r for r in recs if r.get("summary")), {})
        runs.append((os.path.basename(p), head, [r for r in recs if not r.get("summary")]))
    digests = sorted({h.get("manifest", "") for _, h, _ in runs})
    if len(digests) > 1:
        sys.exit("results from different frozen inputs (MANIFEST) are not compared (ADR-0021 §1)")

    def metrics(t):
        n = len(t)
        usage = [r.get("model_usage") or {} for r in t]
        cases = sum(r["cases"] for r in t)
        return [
            ("과제", str(n)),
            ("성공(숨은 테스트 전부 통과)", f"{sum(r['success'] for r in t)}/{n}"),
            ("결함률(결함 과제 / 과제)", f"{sum(r['defect'] for r in t) / n:.3f}" if n else "—"),
            ("미제출(빌드·검증·시그니처 실패)", str(sum(1 for r in t if not r["delivered"]))),
            ("정의역 위반(BMB: requires가 정의역보다 강함)", str(sum(1 for r in t if r.get("domain_ok") is False))),
            ("실패 케이스 / 케이스", f"{sum(r['failed'] for r in t)}/{cases}"),
            ("자기 검사 통과(최종 상태, 증명 게이트 / 에이전트 테스트)",
             f"{sum(1 for r in t if r.get('final_check_ok') is True)}/{n}"),
            ("자기 검사 통과 & 숨은 실패 / 자기 검사 통과", "{}/{}".format(*passed_defective(t))),
            ("테스트 코드 줄(BMB 남은 테스트 면적 / Rust 에이전트 테스트, 전체)",
             str(sum(test_lines_of(r) or 0 for r in t))),
            ("테스트 코드 줄(숨은 판정 통과 과제)", "{}".format(passed_lines(t)[0])),
            ("총 모델 토큰(입력+출력)", str(sum(r["total_tokens"] for r in t))),
            ("모델 입력 토큰", str(sum(u.get("input_tokens", 0) for u in usage))),
            ("모델 출력 토큰", str(sum(u.get("output_tokens", 0) for u in usage))),
            ("도구 호출(합)", str(sum(r["tool_calls"] for r in t))),
            ("검사 도구 호출(verify / cargo_test)", str(sum(r["checks"] for r in t))),
            ("한도 도달 과제", str(sum(1 for r in t if r["limit_hit"]))),
            ("세션 벽시계(초, 합)", f"{sum(r['session_seconds'] for r in t):.1f}"),
            ("도구 시간(초, 합)", f"{sum(r['tool_seconds'] for r in t):.1f}"),
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
            out.append("G5: 판정 불가(두 비교군의 과제 집합이 다름)")
        else:
            out.append(g5_verdict(*[(sum(x["defect"] for x in t), len(t), sum(x["total_tokens"] for x in t))
                                    for t in (b, r)]))
    else:
        out.append("G5: 판정 불가(BMB 결과 하나와 Rust 결과 하나를 함께 주어야 함)")
    if sorted(by_arm) == ["bmb", "rust"] and all(len(v) == 1 for v in by_arm.values()):
        b, r = by_arm["bmb"][0], by_arm["rust"][0]
        if sorted(x["id"] for x in b) == sorted(x["id"] for x in r):
            out.append(oracle_verdict(passed_defective(b), passed_defective(r)))
        else:
            out.append("H-ORACLE: 판정 불가(두 비교군의 과제 집합이 다름)")
    else:
        out.append("H-ORACLE: 판정 불가(BMB 결과 하나와 Rust 결과 하나를 함께 주어야 함)")
    if sorted(by_arm) == ["bmb", "rust"] and all(len(v) == 1 for v in by_arm.values()):
        b, r = by_arm["bmb"][0], by_arm["rust"][0]
        if sorted(x["id"] for x in b) == sorted(x["id"] for x in r):
            out.append(g8_verdict(passed_lines(b), passed_lines(r)))
        else:
            out.append("G8: 판정 불가(두 비교군의 과제 집합이 다름)")
    else:
        out.append("G8: 판정 불가(BMB 결과 하나와 Rust 결과 하나를 함께 주어야 함)")
    ids = sorted({r["id"] for _, _, t in runs for r in t})
    out += ["", "| 과제 | " + " | ".join(n for n, _, _ in runs) + " |", "| --- |" + " --- |" * len(runs)]
    for tid in ids:
        cells = []
        for _, _, t in runs:
            r = next((r for r in t if r["id"] == tid), None)
            if r is None:
                cells.append("—")
            elif r["success"]:
                cells.append("성공")
            elif not r["delivered"]:
                cells.append(f"미제출: {r.get('why', '')[:40]}")
            else:
                kinds = ", ".join(f"{k} {v}" for k, v in r["fail_kinds"].items())
                dom = "정의역 위반; " if r.get("domain_ok") is False else ""
                cells.append(f"결함: {dom}{r['failed']}/{r['cases']} ({kinds})" if r["failed"] else f"결함: {dom}".rstrip("; "))
        out.append(f"| {tid} | " + " | ".join(cells) + " |")
    return "\n".join(out) + "\n"


# ---------- main ----------

def git(*args):
    return subprocess.run(["git", *args], cwd=ROOT, capture_output=True, text=True).stdout.strip()


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
    # proptest needs the template's crates (fetched once, then offline). On by
    # default for measurements; CI runs the scripted agents with it off.
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
    tmp_target = tempfile.mkdtemp(prefix="brb-target-")
    try:
        if args.arm == "rust":
            warm_rust(tmp_target, proptest)
        results = list(pool.map(lambda t: run_session(t, args.arm, args.agent, tmp_target, proptest), ts))
    finally:
        shutil.rmtree(tmp_target, ignore_errors=True)
    for r in results:
        print(json.dumps(r, ensure_ascii=False))
    commit = git("rev-parse", "HEAD")
    dirty = bool(git("status", "--porcelain"))
    n = len(results)
    summary = {
        "summary": True, "task_set": "brb", "arm": args.arm, "agent": args.agent, "tasks": n,
        "proptest": proptest if args.arm == "rust" else None,
        "manifest": manifest_digest(), "commit": commit + ("+dirty" if dirty else ""),
        "limits": {"tool_calls": MAX_TOOL_CALLS, "session_output_tokens": SESSION_OUTPUT_TOKENS},
        "success": f"{sum(r['success'] for r in results)}/{n}",
        "defect_rate": round(sum(r["defect"] for r in results) / n, 3) if n else None,
        "total_tokens": sum(r["total_tokens"] for r in results),
    }
    print(json.dumps(summary, ensure_ascii=False))
    if args.out:
        with open(args.out, "w") as f:
            for r in results + [summary]:
                f.write(json.dumps(r, ensure_ascii=False) + "\n")
    if args.expect == "all-pass" and not all(r["success"] and r["final_check_ok"] for r in results):
        sys.exit("expected every task to pass the hidden judge and its own checker")
    if args.expect == "weak-fail":
        if not results or any(r["success"] for r in results):
            sys.exit("expected the hidden judge to find a defect in every weak solution")
        if any(r["last_check_ok"] is not True or r["final_check_ok"] is not True for r in results):
            sys.exit("expected every weak solution to pass its own checker (verify / cargo test)")


if __name__ == "__main__":
    main()
