#!/usr/bin/env python3
"""Per-turn context of an agent session, decomposed (backlog 2D-25), and the
compact agent view against the current one (backlog 2D-23, ADR-0068).

  python3 dev-docs/experiments/context-size/run.py [--bmb PATH] > dev-docs/experiments/context-size/results.md

Offline: no model is called. Inputs are committed files only:
  - the frozen prompts, task files and `set.json` of each task set, and the
    tool list `bmb mcp` serves for it;
  - the CLI pilot rows (`bench/pilot/results/`), which record every tool call
    with its arguments and result bytes, and the CLI's token usage, whose
    `iterations[-1]` is the last request of the session (its whole context);
  - the official qwen3.8-27b rows (`bench/m1*/results/2026-10-30-*.jsonl`),
    which record prompt tokens summed over requests and the request count.
Each pilot session's calls are replayed in order against a fresh copy of the
seed, once through `bmb mcp` and once through `bmb mcp --compact`, so the two
views are compared on the agents' own call sequences.

Tokens are estimated (`tokest.py`, fitted to Claude counts); bytes are exact.
"""
import argparse
import ast
import json
import os
import shutil
import statistics
import subprocess
import sys
import tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "..", "..", ".."))
BENCH = os.path.join(ROOT, "bench")
sys.path.insert(0, HERE)
import tokest  # noqa: E402

PILOT = os.path.join(BENCH, "pilot", "results")
ANSWER_TOOL = {"name": "answer",
               "description": "Submit the answer to a question: the full paths (unit::name) of the definitions asked for.",
               "inputSchema": {"type": "object", "properties": {"defs": {"type": "array", "items": {"type": "string"}}},
                               "required": ["defs"]}}
# Pilot runs with per-call arguments and result bytes (BMB arm), and the
# task set whose seed and server flags they ran on.
REPLAY = [("m1", "2026-10-28-m1-claude-opus-5-5-r2.jsonl"),
          ("m1b", "2026-10-28-m1b-claude-opus-5-5-r2.jsonl"),
          ("m1b-write", "2026-10-28-m1b-write-claude-opus-5-5.jsonl"),
          ("bvb", "2026-10-28-bvb-claude-opus-5-5-r2.jsonl"),
          ("bvb-write", "2026-10-28-bvb-write-claude-opus-5-5.jsonl")]
VERUS = ("bvb-verus", "2026-10-28-bvb-verus-claude-opus-5-5.jsonl")
QWEN = [("m1", "bench/m1/results/2026-10-30-qwen3.8-27b.jsonl"),
        ("m1b", "bench/m1b/results/2026-10-30-qwen3.8-27b.jsonl")]

BMB = None


class Scratch:
    """A scratch directory at a fixed path: tool results name workspace files
    (diagnostics, L1 `file`), so a random path would change the token counts.
    The path has the length of the harness's session directories."""

    def __enter__(self):
        self.path = os.path.join(tempfile.gettempdir(), "bmb-ctx-00000000")
        shutil.rmtree(self.path, ignore_errors=True)
        os.makedirs(self.path)
        return self.path

    def __exit__(self, *_):
        shutil.rmtree(self.path, ignore_errors=True)


def tok(text):
    return tokest.tokens(text)


def js(x):
    return json.dumps(x, ensure_ascii=False)


def set_config(name):
    p = os.path.join(BENCH, name, "set.json")
    return json.load(open(p)) if os.path.exists(p) else {}


def mcp_tools(flags, hide):
    msg = '{"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}\n'
    r = subprocess.run([BMB, "mcp", *flags, os.path.join(BENCH, "m1", "seed")], input=msg, capture_output=True,
                       text=True)
    return [t for t in json.loads(r.stdout)["result"]["tools"] if t["name"] not in hide]


def tools_text(tools):
    """Tool definitions as the model is given them: name, description, schema."""
    return "\n".join(js({"name": t["name"], "description": t["description"], "input_schema": t["inputSchema"]})
                     for t in tools)


def load_rows(path):
    rows = []
    for line in open(path):
        if not line.strip():
            continue
        r = json.loads(line)
        if r.get("summary") or r.get("partial"):
            continue
        for k in ("calls", "model_usage"):
            if isinstance(r.get(k), str):
                r[k] = ast.literal_eval(r[k])
        rows.append(r)
    return rows


def fmt(n):
    return f"{n:,}"


# ---------- 1. calibration ----------

def calibration_table():
    rows, scale = tokest.calibration(BMB)
    print("## 1. 토큰 추정의 보정\n")
    print(f"추정 = 사전 분할 조각 수 × {scale:.3f}. 보정점은 T1-05가 잰 Claude 토큰 중 지금 `bmb`가 같은 바이트를 내는 것이다.\n")
    print("| 작업 공간 | 투영 | 바이트 | Claude 토큰 | 추정 | 추정/실측 |")
    print("| --- | --- | ---: | ---: | ---: | ---: |")
    for label, view, b, t, p in rows:
        e = round(p * scale)
        print(f"| {label} | {view} | {fmt(b)} | {fmt(t)} | {fmt(e)} | {e / t:.2f} |")
    print()


# ---------- 2. fixed part ----------

def task_prompts(name):
    d = os.path.join(BENCH, name, "tasks")
    tasks = [json.load(open(os.path.join(d, f))) for f in sorted(os.listdir(d))]
    return [t["prompt"] for t in tasks if t["kind"] == "write"]


def fixed_rows():
    """(arm, system prompt, tools text, mean write-task prompt tokens, n tools)."""
    out = []
    for name in ("m1", "m1b", "m1b-write", "bvb", "bvb-write"):
        cfg = set_config(name)
        tools = mcp_tools(cfg.get("mcp_flags", []), set(cfg.get("hide_tools", [])))
        out.append((name, open(os.path.join(BENCH, name, "prompt.md")).read(), tools, task_prompts(name)))
    cfg = set_config("m1b-write")
    out.append(("m1b-write --compact", open(os.path.join(BENCH, "m1b-write", "prompt.md")).read(),
                mcp_tools(cfg["mcp_flags"] + ["--compact"], set(cfg["hide_tools"])), task_prompts("m1b-write")))
    sys.path.insert(0, os.path.join(BENCH, "bvb", "verus"))
    import harness as verus  # noqa: E402
    d = os.path.join(BENCH, "bvb", "tasks")
    vprompts = [json.load(open(os.path.join(d, f)))["verus"]["prompt"] for f in sorted(os.listdir(d))]
    out.append(("bvb-verus", open(os.path.join(BENCH, "bvb", "verus", "prompt.md")).read(), verus.TOOLS, vprompts))
    sys.path.insert(0, os.path.join(BENCH, "brb"))
    import judge  # noqa: E402
    import rust_tools  # noqa: E402
    brb_tasks = judge.load_tasks(os.path.join(BENCH, "brb", "tasks"))
    bmb_tools = mcp_tools([], set())
    out.append(("brb-bmb", open(os.path.join(BENCH, "brb", "prompt-bmb.md")).read(), bmb_tools,
                [judge.task_prompt(t, "bmb") for t in brb_tasks]))
    cfg = set_config("brb-write")
    out.append(("brb-write", open(os.path.join(BENCH, "brb-write", "prompt-bmb.md")).read(),
                mcp_tools(cfg["mcp_flags"], set(cfg["hide_tools"])), [judge.task_prompt(t, "bmb") for t in brb_tasks]))
    out.append(("brb-rust", open(os.path.join(BENCH, "brb", "prompt-rust.md")).read(), rust_tools.TOOLS,
                [judge.task_prompt(t, "rust") for t in brb_tasks]))
    return out


def fixed_table():
    print("## 2. 고정 부분: 매 요청에 들어가는 것\n")
    print("작성 과제 기준(탐색 과제는 `answer` 도구 정의가 더해진다). 도구 정의는 모델에 주는 꼴(이름, 설명, 입력 스키마)의 JSON.\n")
    print("| 판 | 도구 수 | 시스템 프롬프트 B / 토큰 | 도구 정의 B / 토큰 | 과제 지시문 평균 토큰 | 합 토큰 |")
    print("| --- | ---: | ---: | ---: | ---: | ---: |")
    fixed = {}
    for arm, sysp, tools, prompts in fixed_rows():
        tt = tools_text(tools)
        task = round(statistics.mean(tok(p) for p in prompts))
        total = tok(sysp) + tok(tt) + task
        fixed[arm] = total
        print(f"| {arm} | {len(tools)} | {fmt(len(sysp.encode()))} / {fmt(tok(sysp))} | "
              f"{fmt(len(tt.encode()))} / {fmt(tok(tt))} | {fmt(task)} | {fmt(total)} |")
    print()
    return fixed


# ---------- 3. replay ----------

def group(name, args):
    if name == "query":
        q = args.get("name", "?")
        return "query units" if q == "units" else ("query slice" if q == "slice" else "query other")
    if name == "map":
        return "map " + str(args.get("level", "all"))
    if name.startswith("tx_"):
        return "tx_*"
    return name


class Server:
    def __init__(self, root, flags):
        self.p = subprocess.Popen([BMB, "mcp", *flags, root], stdin=subprocess.PIPE, stdout=subprocess.PIPE,
                                  text=True)
        self.n = 0

    def call(self, name, args):
        self.n += 1
        msg = {"jsonrpc": "2.0", "id": self.n, "method": "tools/call", "params": {"name": name, "arguments": args}}
        self.p.stdin.write(json.dumps(msg) + "\n")
        self.p.stdin.flush()
        return json.loads(self.p.stdout.readline())["result"]["content"][0]["text"]

    def close(self):
        self.p.stdin.close()
        self.p.wait()


def warm_cache(seed, tmp):
    ws = os.path.join(tmp, "warm")
    shutil.copytree(os.path.join(seed, "src"), os.path.join(ws, "src"))
    subprocess.run([BMB, "verify", "--cache", os.path.join(ws, ".bmb-cache"), ws], capture_output=True)
    return os.path.join(ws, ".bmb-cache")


def replay_set(name, path):
    """Per session: [(group, recorded bytes, default text, compact text)] and the row."""
    cfg = set_config(name)
    seed = os.path.join(BENCH, name, "seed")
    out = []
    with Scratch() as tmp:
        cache = warm_cache(seed, tmp)
        for i, r in enumerate(load_rows(os.path.join(PILOT, path))):
            servers = []
            for view, extra in (("default", []), ("compact", ["--compact"])):
                ws = os.path.join(tmp, view, "ws")
                shutil.rmtree(ws, ignore_errors=True)
                shutil.copytree(os.path.join(seed, "src"), os.path.join(ws, "src"))
                shutil.copytree(cache, os.path.join(ws, ".bmb-cache"))
                servers.append(Server(ws, cfg.get("mcp_flags", []) + extra))
            calls = []
            for c in r["calls"]:
                if c["name"] == "answer":
                    calls.append(("answer", c["bytes"], '{"answered":true}', '{"answered":true}', c["args"]))
                    continue
                a, b = (s.call(c["name"], c["args"]) for s in servers)
                calls.append((group(c["name"], c["args"]), c["bytes"], a, b, c["args"]))
            for s in servers:
                s.close()
            out.append((r, calls))
    return out


def replay_tables(fixed):
    print("## 3. 도구 결과: 파일럿 세션의 호출을 그대로 다시 돌림\n")
    print("파일럿(Claude Code CLI, `claude-opus-5-5`, ADR-0058 T0)의 BMB 쪽 세션마다 기록된 호출을 순서대로, 시드 사본 둘에 "
          "`bmb mcp`(지금 판)와 `bmb mcp --compact`로 다시 돌렸다. \"재현\"은 기본 판의 결과 바이트가 기록과 같은 호출의 비율이다"
          "(다르면 기록 뒤의 컴파일러 변경, 예: L1의 `id` 필드).\n")
    data = {name: replay_set(name, path) for name, path in REPLAY}
    print("| 집합 | 세션 | 호출 | 재현 | 기본 B | `--compact` B | 기본 토큰 | `--compact` 토큰 | 줄어듦 |")
    print("| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |")
    for name, sessions in data.items():
        calls = [c for _, cs in sessions for c in cs]
        same = sum(1 for c in calls if len(c[2].encode()) == c[1])
        da = sum(len(c[2].encode()) for c in calls)
        db = sum(len(c[3].encode()) for c in calls)
        ta = sum(tok(c[2]) for c in calls)
        tb = sum(tok(c[3]) for c in calls)
        print(f"| {name} | {len(sessions)} | {len(calls)} | {same / len(calls):.0%} | {fmt(da)} | {fmt(db)} | "
              f"{fmt(ta)} | {fmt(tb)} | {1 - tb / ta:.0%} |")
    print()
    print("도구별(모든 집합 합, 추정 토큰):\n")
    print("| 도구 | 호출 | 호출당 기본 토큰 | 호출당 `--compact` 토큰 | 기본 합 | 줄어듦 |")
    print("| --- | ---: | ---: | ---: | ---: | ---: |")
    by = {}
    for sessions in data.values():
        for _, cs in sessions:
            for g, _, a, b, _ in cs:
                x = by.setdefault(g, [0, 0, 0])
                x[0] += 1
                x[1] += tok(a)
                x[2] += tok(b)
    for g, (n, a, b) in sorted(by.items(), key=lambda kv: -kv[1][1]):
        print(f"| {g} | {n} | {fmt(round(a / n))} | {fmt(round(b / n))} | {fmt(a)} | "
              f"{(1 - b / a) if a else 0:.0%} |")
    print()
    return data


# ---------- 4. the last request's context ----------

def usage_total(u):
    return u.get("input_tokens", 0) + u.get("cache_creation_input_tokens", 0) + u.get("cache_read_input_tokens", 0)


def last_request(u):
    it = (u.get("iterations") or [None])[-1]
    return usage_total(it) if it else None


def final_tables(fixed, data):
    print("## 4. 마지막 요청의 문맥 분해(파일럿, 세션 평균)\n")
    print("마지막 요청의 입력은 고정 부분과 그때까지의 모든 도구 결과·호출 인자·모델 출력이다(문맥의 최댓값, 64K 한도와 견줄 값). "
          "\"실측\"은 CLI가 보고한 마지막 요청의 입력 토큰(캐시 포함), 나머지는 추정 토큰이다. \"나머지\" = 실측 − 고정 − 도구 결과 − "
          "호출 인자: 모델의 글·사고와 CLI가 덧붙이는 틀. `--compact` 열은 같은 호출의 결과를 `--compact`로 바꾸고 "
          "도구 설명이 늘어난 만큼을 더한 값이다.\n")
    print("| 집합 | 세션 | 실측 | 고정(추정) | 읽기 결과 | 쓰기·증명 결과 | 호출 인자 | 나머지 | `--compact`이면 | 줄어듦 |")
    print("| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |")
    reads = ("query", "map")
    for name, sessions in data.items():
        rows = []
        for r, cs in sessions:
            last = last_request(r["model_usage"])
            if last is None:
                continue
            rd = sum(tok(a) for g, _, a, _, _ in cs if g.split()[0] in reads)
            wr = sum(tok(a) for g, _, a, _, _ in cs if g.split()[0] not in reads)
            args = sum(tok(js(x)) for *_, x in cs)
            saved = sum(tok(a) - tok(b) for _, _, a, b, _ in cs)
            rows.append((last, rd, wr, args, saved))
        n = len(rows)
        m = [sum(x[i] for x in rows) / n for i in range(5)]
        f = fixed[name]
        cfg = set_config(name)
        flags, hide = cfg.get("mcp_flags", []), set(cfg.get("hide_tools", []))
        m[4] -= tok(tools_text(mcp_tools(flags + ["--compact"], hide))) - tok(tools_text(mcp_tools(flags, hide)))
        print(f"| {name} | {n} | {fmt(round(m[0]))} | {fmt(f)} | {fmt(round(m[1]))} | {fmt(round(m[2]))} | "
              f"{fmt(round(m[3]))} | {fmt(round(m[0] - f - m[1] - m[2] - m[3]))} | {fmt(round(m[0] - m[4]))} | "
              f"{m[4] / m[0]:.0%} |")
    rows = [last_request(r["model_usage"]) for r in load_rows(os.path.join(PILOT, VERUS[1]))]
    rows = [x for x in rows if x is not None]
    mean = sum(rows) / len(rows)
    f = fixed["bvb-verus"]
    print(f"| bvb-verus | {len(rows)} | {fmt(round(mean))} | {fmt(f)} | — | — | — | {fmt(round(mean - f))} "
          f"(도구 결과·인자 포함) | — | — |")
    print()


# ---------- 5. official qwen rows ----------

def qwen_tables(fixed, data):
    print("## 5. 정식 측정(qwen3.8-27b, 커밋 `8d569693`)의 요청당 문맥\n")
    print("하네스는 세션의 요청 수와 프롬프트 토큰 합을 남긴다(`model_usage`). 요청별 값과 호출별 결과 크기는 남기지 않는다. "
          "그래서 요청당 평균은 정확하고, 마지막 요청은 하한(고정 + 읽기 결과)만 낸다. 고정 부분의 qwen 토큰은 `w08-digits`"
          "(m1, 첫 요청에서 출력 한도로 끝나 요청이 하나)의 2,291토큰으로 맞췄다. 읽기 결과의 qwen 토큰은 파일럿 재실행의 "
          "바이트당 추정 토큰을 같은 비로 줄인 값이다.\n")
    rows_m1 = load_rows(os.path.join(ROOT, QWEN[0][1]))
    w08 = next(r for r in rows_m1 if r["id"] == "w08-digits")
    task = json.load(open(os.path.join(BENCH, "m1", "tasks", "w08-digits.json")))["prompt"]
    m1_sys = open(os.path.join(BENCH, "m1", "prompt.md")).read()
    est = tok(m1_sys) + tok(tools_text(mcp_tools([], set()))) + tok(task)
    q = w08["model_usage"]["input_tokens"] / est
    reads = [(len(a.encode()), tok(a), tok(b)) for sessions in data.values() for _, cs in sessions
             for g, _, a, b, _ in cs if g.split()[0] in ("query", "map")]
    per_byte = sum(r[1] for r in reads) / sum(r[0] for r in reads)
    shrink = sum(r[2] for r in reads) / sum(r[1] for r in reads)
    print(f"qwen/추정 비 {q:.2f}(고정 부분), 읽기 결과 바이트당 추정 토큰 {per_byte:.3f}, 재실행에서 읽기 결과의 "
          f"`--compact`/기본 토큰 비 {shrink:.2f}.\n")
    print("| 집합 | 세션 | 요청 | 요청당 프롬프트 토큰(평균) | 세션 평균의 최댓값 | 고정(qwen) | 고정의 몫 | 읽기 바이트(세션 평균) | "
          "마지막 요청 하한(최대) | 같은 하한, `--compact` | 끝남: 출력 한도 / 호출 한도 / `api_error` |")
    print("| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |")
    for name, path in QWEN:
        rows = load_rows(os.path.join(ROOT, path))
        f = round(fixed[name] * q)
        reqs = sum(r["model_usage"]["requests"] for r in rows)
        inp = sum(r["model_usage"]["input_tokens"] for r in rows)
        top = max(r["model_usage"]["input_tokens"] / r["model_usage"]["requests"] for r in rows)
        lower = max(f + r["read_bytes"] * per_byte * q for r in rows)
        lower_c = max(f + r["read_bytes"] * per_byte * q * shrink for r in rows)
        stops = [sum(1 for r in rows if r.get("stop") == s) for s in ("length",)]
        limit = sum(1 for r in rows if r["limit_hit"])
        api = sum(1 for r in rows if r.get("stop") == "api_error")
        print(f"| {name} | {len(rows)} | {reqs} | {fmt(round(inp / reqs))} | {fmt(round(top))} | {fmt(f)} | "
              f"{f * reqs / inp:.0%} | {fmt(round(statistics.mean(r['read_bytes'] for r in rows)))} | "
              f"{fmt(round(lower))} | {fmt(round(lower_c))} | {stops[0]} / {limit} / {api} |")
    print()
    print("요청당 평균이 30,000토큰 이상인 세션:\n")
    print("| 집합 | 과제 | 성공 | 끝난 이유 | 요청 | 요청당 프롬프트 토큰 | 읽기 호출 / 바이트 |")
    print("| --- | --- | --- | --- | ---: | ---: | --- |")
    for name, path in QWEN:
        for r in load_rows(os.path.join(ROOT, path)):
            u = r["model_usage"]
            per = u["input_tokens"] / u["requests"]
            if per >= 30000:
                why = r.get("stop") or ("호출 한도" if r["limit_hit"] else "—")
                print(f"| {name} | {r['id']} | {'예' if r['success'] else '아니오'} | {why} | {u['requests']} | "
                      f"{fmt(round(per))} | {r['read_calls']} / {fmt(r['read_bytes'])} |")
    print()


# ---------- 6. projections of the same map ----------

def candidates(l1_rows):
    def jl(rows):
        return "".join(json.dumps(r, ensure_ascii=False, separators=(",", ":")) + "\n" for r in rows)
    hashes = ("body", "iface", "id")
    noise = hashes + ("file", "line", "rlimit_max", "level")
    out = [("해시 12자", jl([{k: (v[:19] if k in hashes else v) for k, v in r.items()} for r in l1_rows])),
           ("해시 뺌", jl([{k: v for k, v in r.items() if k not in hashes} for r in l1_rows])),
           ("해시·위치·비용 뺌", jl([{k: v for k, v in r.items() if k not in noise} for r in l1_rows])),
           ("+ 빈 필드 생략", jl([{k: v for k, v in r.items() if k not in noise and v not in ([], None, "")}
                              for r in l1_rows]))]
    return out


def projection_table():
    print("## 6. 같은 질문에 답하는 투영의 크기(2D-23)\n")
    print("작업 공간 전체의 표면(L1)을 정규형 코드(본문 포함, `bmb fmt`)와 견준다. `--compact`는 MCP 서버가 실제로 내는 "
          "`map L1`이다. L0(`query units`)와 시드의 `verify`도 함께.\n")
    print("| 작업 공간 | 투영 | 바이트 | 추정 토큰 | 코드 대비 |")
    print("| --- | --- | ---: | ---: | ---: |")
    for label, rel in (("m1 seed", "bench/m1/seed"), ("m1b seed", "bench/m1b/seed"), ("bvb seed", "bench/bvb/seed")):
        ws = os.path.join(ROOT, rel)
        src = os.path.join(ws, "src")
        code = subprocess.run([BMB, "fmt", *sorted(os.path.join(src, f) for f in os.listdir(src))],
                              capture_output=True, text=True).stdout
        base = tok(code)
        with Scratch() as tmp:
            w = os.path.join(tmp, "ws")
            shutil.copytree(src, os.path.join(w, "src"))
            views = {}
            for view, extra in (("default", []), ("compact", ["--compact"])):
                s = Server(w, extra)
                views[view] = {k: s.call(name, args) for k, name, args in (("L1", "map", {"level": "L1"}),
                                                                          ("L0", "query", {"name": "units"}),
                                                                          ("verify", "verify", {}))}
                s.close()
        l1_rows = [json.loads(x) for x in views["default"]["L1"].splitlines()[1:]]
        rows = [("정규형 코드", code), ("L1 지금", views["default"]["L1"])] + candidates(l1_rows) + [
            ("L1 `--compact`(Rust 꼴)", views["compact"]["L1"]),
            ("L0 지금(`query units`)", views["default"]["L0"]), ("L0 `--compact`", views["compact"]["L0"]),
            ("`verify` 지금", views["default"]["verify"]), ("`verify` `--compact`", views["compact"]["verify"])]
        for name, text in rows:
            print(f"| {label} | {name} | {fmt(len(text.encode()))} | {fmt(tok(text))} | {tok(text) / base:.2f} |")
    print()


def main():
    global BMB
    ap = argparse.ArgumentParser()
    ap.add_argument("--bmb", default=os.path.join(ROOT, "target", "debug", "bmb"))
    BMB = os.path.abspath(ap.parse_args().bmb)
    print("# 차례당 문맥 크기 분해와 압축 에이전트 투영\n")
    print("생성: `python3 dev-docs/experiments/context-size/run.py`(모델 호출 없음). 해석은 `README.md`.\n")
    calibration_table()
    fixed = fixed_table()
    data = replay_tables(fixed)
    final_tables(fixed, data)
    qwen_tables(fixed, data)
    projection_table()


if __name__ == "__main__":
    main()
