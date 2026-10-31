#!/usr/bin/env python3
"""Original-Verus side of BVB (backlog 2A-09, ADR-0034 §2).

  harness.py run --agent AGENT [--tasks ID,..] [--out FILE] [--expect all-pass|weak-fail] [--jobs N]
  harness.py compare BMB.jsonl VERUS.jsonl     paired table: one row per task, BMB and Verus side by side
  harness.py strength RESULTS.jsonl            spec strength and total tokens of one run (either side)
  harness.py check-env                         is the pinned Verus installed? (exit 1 if not)

Each BVB task (bench/bvb/tasks, frozen by bench/bvb/MANIFEST) carries, next to
its BMB form, the original Verus form (`verus`): the same natural-language
description with the original Verus signature, the original specification
(helpers renamed `bvb_ref_*`) and a verified Verus solution. A session gets
one Verus file that starts as a skeleton and three tools (`read_file`,
`write_file`, `verify`), under the BMB rules: one session per task, no retry,
40 tool calls (ADR-0021 §3), the same model settings (bench/m1/anthropic_agent.py).

Judging mirrors ADR-0021 §2 for BMB:
  1. the final file verifies (`verus --no-cheating`: no assume/admit/external_body);
  2. the target function has the required signature (result name and
     whitespace ignored, `pub` allowed);
  3. spec strength: a hidden function `requires <original pre> ensures
     <original post> { target(args) }` is appended and the file verifies again,
     so the agent's `requires` is implied by the original one and its
     `ensures` implies the original one.

Agents:
  oracle            writes `verus.reference` (CI check that every task is solvable
                    and the judge accepts the original spec)
  weak              writes `verus.reference` with the target's `ensures` removed;
                    the spec check must reject it
  openai:MODEL      a real model, OpenAI-compatible Chat Completions
  anthropic:MODEL   a real model, Anthropic Messages API
                    (models and endpoints: ADR-0066, bench/m1/model_api.py)

The verifier is `verus` from bench/bvb/verus/install.sh ($BVB_VERUS overrides
the binary). Standard library only.
"""
import argparse
import concurrent.futures
import importlib.util
import json
import os
import re
import subprocess
import sys
import tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
BVB = os.path.dirname(HERE)
ROOT = os.path.dirname(os.path.dirname(BVB))
M1 = os.path.join(ROOT, "bench", "m1")
sys.path.insert(0, M1)  # anthropic_agent
# The BMB harness (frozen-input checks, ScriptAgent), loaded under its own name.
_spec = importlib.util.spec_from_file_location("bmb_harness", os.path.join(M1, "harness.py"))
bmb_harness = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(bmb_harness)
import resultlog  # noqa: E402
import sessionlog  # noqa: E402

MAX_TOOL_CALLS = bmb_harness.MAX_TOOL_CALLS
SKELETON = "use vstd::prelude::*;\n\nfn main() {}\n\nverus! {\n\n} // verus!\n"
TIMEOUT = 600
HEADER_KEYWORDS = ("requires", "ensures", "recommends", "decreases", "returns", "opens_invariants", "no_unwind")

TOOLS = [
    {"name": "read_file", "description": "Return the current contents of the Verus file.",
     "inputSchema": {"type": "object", "properties": {}}},
    {"name": "write_file", "description": "Replace the whole Verus file with `code`.",
     "inputSchema": {"type": "object", "properties": {"code": {"type": "string"}}, "required": ["code"]}},
    {"name": "verify", "description": "Run Verus (--no-cheating) on the file and return its output.",
     "inputSchema": {"type": "object", "properties": {}}},
]


def env():
    vals = {}
    with open(os.path.join(HERE, "verus.env")) as f:
        lines = f.read().splitlines()
    for line in lines:
        m = re.match(r"^(\w+)=(.*)$", line.strip())
        if m:
            vals[m.group(1)] = m.group(2)
    return vals


# ---------- verifier ----------

class VerusVerifier:
    """Runs the pinned Verus binary on a file's text."""

    def __init__(self, binary=None):
        e = env()
        self.version = e["VERUS_VERSION"]
        self.toolchain = e["VERUS_RUST_TOOLCHAIN"]
        home = os.environ.get("BVB_VERUS_HOME", os.path.expanduser(f"~/.cache/bmb/verus/{self.version}"))
        self.binary = binary or os.environ.get("BVB_VERUS", os.path.join(home, "verus-x86-linux", "verus"))

    def available(self):
        if not os.access(self.binary, os.X_OK):
            return False
        r = subprocess.run([self.binary, "--version"], capture_output=True, text=True,
                           env={**os.environ, "RUSTUP_TOOLCHAIN": self.toolchain})
        return r.returncode == 0 and f"Version: {self.version}" in r.stdout

    def verify(self, code, extra=()):
        """(verified, output). Output is the verifier's stderr+stdout, trimmed."""
        with tempfile.TemporaryDirectory(prefix="bvb-verus-") as d:
            path = os.path.join(d, "main.rs")
            with open(path, "w") as f:
                f.write(code)
            try:
                r = subprocess.run([self.binary, "--no-cheating", *extra, path], capture_output=True, text=True, cwd=d,
                                   timeout=TIMEOUT, env={**os.environ, "RUSTUP_TOOLCHAIN": self.toolchain})
            except subprocess.TimeoutExpired:
                return False, f"verus: timed out after {TIMEOUT}s"
            out = (r.stderr + r.stdout).replace(path, "main.rs").replace(d, ".")
            return r.returncode == 0, out[-20000:]


# ---------- signatures and spec check ----------

def _skip_balanced(code, i, open_ch, close_ch):
    depth = 0
    while i < len(code):
        if code[i] == open_ch:
            depth += 1
        elif code[i] == close_ch:
            depth -= 1
            if depth == 0:
                return i + 1
        i += 1
    return -1


def find_header(code, target):
    """(start, end) of `fn target(..) -> ..` plus its spec clauses, ending just
    before the body's `{`; None if the function is missing. Braces inside
    parentheses are skipped; a clause may not contain a top-level `{`."""
    m = re.search(r"(?m)^[ \t]*(?:pub(?:\([^)]*\))?\s+)?fn\s+" + re.escape(target) + r"\s*\(", code)
    if not m:
        return None
    i = _skip_balanced(code, m.end() - 1, "(", ")")
    if i < 0:
        return None
    depth = 0
    while i < len(code):
        c = code[i]
        if c in "([":
            depth += 1
        elif c in ")]":
            depth -= 1
        elif c == "{" and depth == 0:
            return m.start(), i
        elif c == ";" and depth == 0:
            return None
        i += 1
    return None


def signature_of(code, target):
    """The target's signature (`fn ... -> ...` without spec clauses), or None."""
    h = find_header(code, target)
    if not h:
        return None
    text = code[h[0]:h[1]]
    m = re.search(r"\b(" + "|".join(HEADER_KEYWORDS) + r")\b", text)
    return (text[:m.start()] if m else text).strip()


def normalize_signature(sig):
    s = re.sub(r"^\s*pub(\([^)]*\))?\s+", "", sig.strip())
    s = re.sub(r"->\s*\(\s*\w+\s*:\s*(.*)\)\s*$", r"-> \1", s)
    return re.sub(r"\s+", "", s)


def strip_ensures(code, target):
    """The code with the target's `ensures` clauses removed (the weak control)."""
    h = find_header(code, target)
    if not h:
        return code
    head = code[h[0]:h[1]]
    m = re.search(r"\bensures\b", head)
    if not m:
        return code
    rest = head[m.end():]
    n = re.search(r"\b(" + "|".join(k for k in HEADER_KEYWORDS if k != "ensures") + r")\b", rest)
    cut_end = m.end() + (n.start() if n else len(rest))
    return code[:h[0]] + head[:m.start()] + head[cut_end:] + code[h[1]:]


def hidden_prefix(code):
    prefix, n = "bvb_", 0
    while re.search(r"\b" + prefix, code):
        n += 1
        prefix = f"bvb{n}_"
    return prefix


def check_block(task, code):
    """The hidden spec-strength check appended to the agent's file."""
    v = task["verus"]
    prefix = hidden_prefix(code)
    sig = v["signature"].strip()
    check_sig = re.sub(r"^(?:pub\s+)?fn\s+\w+\s*\(", "fn bvb_check(", sig, count=1)
    clauses = ""
    if v["pre"]:
        clauses += "    requires\n        " + v["pre"].strip().rstrip(",") + ",\n"
    clauses += "    ensures\n        " + v["post"].strip().rstrip(",") + ",\n"
    block = (
        "\n// ---- hidden BVB spec check ----\nverus! {\n\n" + v["specs"].strip() + "\n\n"
        + check_sig + "\n" + clauses + "{\n    " + v["target"] + "(" + v["args"] + ")\n}\n\n} // verus!\n"
    )
    return block.replace("bvb_", prefix)


def squash(text):
    return re.sub(r"\s+", " ", text).strip()


def skeleton(task):
    """The file a session starts from: the crate skeleton plus the task's given
    specification functions (tasks whose original spec is a recursive function)."""
    given = task["verus"].get("given")
    if not given:
        return SKELETON
    return SKELETON.replace("verus! {\n\n", "verus! {\n\n" + given.strip() + "\n\n", 1)


def judge(task, code, verifier):
    verified, _ = verifier.verify(code)
    sig = signature_of(code, task["verus"]["target"])
    sig_ok = sig is not None and normalize_signature(sig) == normalize_signature(task["verus"]["signature"])
    accepted = False
    if verified and sig_ok:
        accepted, _ = verifier.verify(code + check_block(task, code))
    verdict = {"verified": verified, "signature_ok": sig_ok, "spec_accepted": accepted,
               "success": verified and sig_ok and accepted}
    if task["verus"].get("given"):
        # The hidden spec names the given functions, so they must be unchanged.
        intact = squash(task["verus"]["given"]) in squash(code)
        verdict["given_intact"] = intact
        verdict["success"] = verdict["success"] and intact
    return verdict


# ---------- sessions ----------

def make_agent(kind, task):
    v = task["verus"]
    if kind == "oracle":
        return bmb_harness.ScriptAgent([("write_file", {"code": v["reference"]}), ("verify", {})])
    if kind == "weak":
        return bmb_harness.ScriptAgent([("write_file", {"code": strip_ensures(v["reference"], v["target"])}),
                                        ("verify", {})])
    from model_api import make_model_agent  # noqa: E402  (bench/m1, same settings as BMB)
    with open(os.path.join(HERE, "prompt.md")) as f:
        prompt = f.read()
    if agent := make_model_agent(kind, prompt, v["prompt"]):
        return agent
    sys.exit(f"unknown agent {kind}")


def run_task(task, agent_kind, verifier, agent=None, sessions=None):
    agent = agent or make_agent(agent_kind, task)
    if hasattr(agent, "set_tools"):
        agent.set_tools(TOOLS)
    code = skeleton(task)
    m = {"tool_calls": 0, "writes": 0, "verify_calls": 0, "limit_hit": False}
    # Every call's tool and result size, in order, with BMB_CONTEXT_LOG=1 (backlog 2D-25).
    calls_log = []
    calls = []  # every call in full, for the session record (INF-17)
    last = []
    while True:
        step = agent.next_calls(last)
        if not step:
            break
        last = []
        for name, args in step:
            if m["tool_calls"] >= MAX_TOOL_CALLS:
                m["limit_hit"] = True
                break
            m["tool_calls"] += 1
            if name == "read_file":
                text, err = code, False
            elif name == "write_file" and isinstance(args.get("code"), str):
                code = args["code"]
                m["writes"] += 1
                text, err = '{"written":true}', False
            elif name == "verify":
                m["verify_calls"] += 1
                ok, out = verifier.verify(code)
                text, err = ("verified\n" if ok else "verification failed\n") + out, not ok
            else:
                text, err = f"unknown tool or bad arguments: {name}", True
            calls_log.append({"name": name, "bytes": len(text.encode()), "error": bool(err)})
            calls.append({"name": name, "args": args, "error": bool(err), "text": text})
            last.append((name, args, text, err))
        if m["limit_hit"]:
            break
    verdict = judge(task, code, verifier)
    if sessions:
        sessionlog.write(sessions, sessionlog.record(task["id"], agent_kind, agent, calls,
                                                     {"changed": {"main.rs": code}, "removed": []}))
    return {"id": task["id"], "kind": "write", "agent": agent_kind, **verdict, **m,
            "model_usage": agent.usage, "agent_settings": getattr(agent, "settings", None),
            "stop": getattr(agent, "stop", None),
            **({"calls": calls_log} if os.environ.get("BMB_CONTEXT_LOG") == "1" else {})}


def load_tasks(ids=None):
    bmb_harness.SET = BVB
    bmb_harness.check_manifest()
    tdir = os.path.join(BVB, "tasks")
    tasks = []
    for name in sorted(os.listdir(tdir)):
        with open(os.path.join(tdir, name)) as f:
            tasks.append(json.load(f))
    if ids:
        wanted = set(ids.split(","))
        tasks = [t for t in tasks if t["id"] in wanted]
    return tasks


def run(args, verifier):
    tasks = load_tasks(args.tasks)
    commit = bmb_harness.run_commit()
    log = resultlog.ResultLog(args.out, {"agent": args.agent, "task_set": "bvb-verus",
                                         "manifest": bmb_harness.manifest_digest(), "commit": commit,
                                         "tasks": [t["id"] for t in tasks], **bmb_harness.condition_fields()})

    def session(t):
        r = run_task(t, args.agent, verifier, sessions=args.sessions)
        log.add(r)
        return r

    with concurrent.futures.ThreadPoolExecutor(max_workers=max(1, args.jobs)) as pool:
        list(pool.map(session, [t for t in tasks if not log.done(t["id"])]))
    results = [log.rows[t["id"]] for t in tasks]
    for r in results:
        print(json.dumps(r, ensure_ascii=False))
    summary = {"summary": True, "agent": args.agent, "tasks": len(results), "task_set": "bvb-verus",
               "manifest": bmb_harness.manifest_digest(), "verus_version": getattr(verifier, "version", None),
               "commit": commit, **bmb_harness.condition_fields(), "jobs": args.jobs, "write_success": f"{sum(r['success'] for r in results)}/{len(results)}"}
    log.finish(results, summary)
    print(json.dumps(summary, ensure_ascii=False))
    if args.expect == "all-pass" and not all(r["success"] for r in results):
        sys.exit("expected every task to pass: " + ", ".join(r["id"] for r in results if not r["success"]))
    if args.expect == "weak-fail" and (not results or any(r["success"] or not r["verified"] for r in results)):
        sys.exit("expected every weak solution to verify and be rejected by the spec check: "
                 + ", ".join(r["id"] for r in results if r["success"] or not r["verified"]))
    return results


def spec_strength(recs):
    """(accepted, eligible): among tasks whose final code verifies with the
    required signature, those whose spec implies the hidden original contract
    (backlog 2A-12, H-SPEC). Implementation failures do not count against the
    spec, so this reads the spec apart from the 1-shot success rate."""
    eligible = [r for r in recs if r.get("verified") and r.get("signature_ok")]
    return sum(1 for r in eligible if r.get("spec_accepted")), len(eligible)


def total_tokens(recs):
    return sum((r.get("model_usage") or {}).get("input_tokens", 0) + (r.get("model_usage") or {}).get("output_tokens", 0)
               for r in recs)


def load_results(p):
    with open(p) as f:
        recs = [json.loads(line) for line in f if line.strip()]
    return next((r for r in recs if r.get("summary")), {}), [r for r in recs if not r.get("summary")]


def compare(bmb_path, verus_path):
    """Markdown table pairing a BMB run (`--set bench/bvb`) with a Verus run."""
    def load(p, want_set):
        head, recs = load_results(p)
        if head.get("task_set") != want_set:
            sys.exit(f"{p}: task_set {head.get('task_set')!r}, expected {want_set!r}")
        return head, {r["id"]: r for r in recs}

    bh, b = load(bmb_path, "bvb")
    vh, v = load(verus_path, "bvb-verus")
    if bh.get("manifest") != vh.get("manifest"):
        sys.exit("the two runs used different BVB manifests")

    def cell(r):
        if r is None:
            return "—"
        if r["success"]:
            return "성공"
        return "실패: " + ", ".join(k for k in ("verified", "signature_ok", "spec_accepted", "given_intact")
                                   if not r.get(k, True))

    ids = sorted(set(b) | set(v))
    both = [i for i in ids if i in b and i in v]
    rows = ["| 지표 | BMB | Verus |", "| --- | --- | --- |",
            f"| 에이전트 | {bh.get('agent')} | {vh.get('agent')} |",
            f"| 커밋 | {str(bh.get('commit'))[:12]} | {str(vh.get('commit'))[:12]} |",
            f"| Verus 버전 | — | {vh.get('verus_version')} |",
            f"| 1회 성공률 | {sum(b[i]['success'] for i in both)}/{len(both)} | {sum(v[i]['success'] for i in both)}/{len(both)} |",
            f"| 도구 호출(합) | {sum(b[i]['tool_calls'] for i in both)} | {sum(v[i]['tool_calls'] for i in both)} |",
            "| 모델 입력 토큰(합) | "
            + " | ".join(str(sum((x[i].get('model_usage') or {}).get('input_tokens', 0) for i in both)) for x in (b, v)) + " |",
            "| 모델 출력 토큰(합) | "
            + " | ".join(str(sum((x[i].get('model_usage') or {}).get('output_tokens', 0) for i in both)) for x in (b, v)) + " |",
            "| 총 모델 토큰(입력+출력) | " + " | ".join(str(total_tokens([x[i] for i in both])) for x in (b, v)) + " |",
            "| 명세 강도(검증·시그니처 통과 중 원본 계약 함의) | "
            + " | ".join("{}/{}".format(*spec_strength([x[i] for i in both])) for x in (b, v)) + " |",
            "", "| 과제 | BMB | Verus |", "| --- | --- | --- |"]
    rows += [f"| {i} | {cell(b.get(i))} | {cell(v.get(i))} |" for i in ids]
    return "\n".join(rows) + "\n"


def main():
    ap = argparse.ArgumentParser()
    sub = ap.add_subparsers(dest="cmd", required=True)
    rn = sub.add_parser("run")
    rn.add_argument("--agent", required=True)
    rn.add_argument("--tasks")
    rn.add_argument("--out")
    rn.add_argument("--expect", choices=["all-pass", "weak-fail"])
    rn.add_argument("--jobs", type=int, default=1)
    rn.add_argument("--sessions", help="directory for session records (backlog INF-17)")
    cp = sub.add_parser("compare")
    cp.add_argument("bmb")
    cp.add_argument("verus")
    st = sub.add_parser("strength")
    st.add_argument("results")
    sub.add_parser("check-env")
    args = ap.parse_args()
    if args.cmd == "compare":
        sys.stdout.write(compare(args.bmb, args.verus))
        return
    if args.cmd == "strength":
        head, recs = load_results(args.results)
        a, n = spec_strength(recs)
        print(json.dumps({"task_set": head.get("task_set"), "tasks": len(recs), "spec_accepted": a, "eligible": n,
                          "total_tokens": total_tokens(recs)}))
        return
    verifier = VerusVerifier()
    if args.cmd == "check-env":
        ok = verifier.available()
        print(f"verus {verifier.version}: {'installed' if ok else 'not installed'} ({verifier.binary})")
        sys.exit(0 if ok else 1)
    if not verifier.available():
        sys.exit(f"verus {verifier.version} not found at {verifier.binary}: run bench/bvb/verus/install.sh")
    run(args, verifier)


if __name__ == "__main__":
    main()
