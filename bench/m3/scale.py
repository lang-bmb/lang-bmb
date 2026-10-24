#!/usr/bin/env python3
"""Query cost scale measurement (backlog 3A-02, DESIGN.md principle 7).

  scale.py verify  --work DIR [--sizes 100 1000 10000] [--out F.jsonl]
      generate each size with gen.py (same parameters), verify it cold with
      --cache DIR/wN/.bmb-cache, verify again warm, then change one leaf body
      and verify incrementally. One JSON line per run.

  scale.py queries --work DIR [--sizes ...] [--runs 5] [--out F.jsonl]
      on the verified workspaces, through one `bmb mcp` server per size (the
      agent path, warm .bmb-cache): for fixed probe definitions in unit m1
      (same local neighbourhood in every size, see gen.py) run slice, surface,
      edges, edges both, impact and effects, plus map L0 and the whole-program
      baselines; record response bytes, ceil(bytes/4) tokens (ADR-0021) and
      the median wall time of RUNS calls after one warm-up call. Front-end
      baselines: `bmb check` (CLI, whole workspace) and MCP `tools/list`.

  scale.py table F.jsonl...   print the Markdown tables of the results.

Python 3 standard library only. The binary is $BMB or target/release/bmb
(build it with `cargo build --release`).
"""

import argparse
import json
import math
import os
import platform
import re
import resource
import statistics
import subprocess
import sys
import time

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(os.path.dirname(HERE))
BMB = os.environ.get("BMB", os.path.join(ROOT, "target", "release", "bmb"))
sys.path.insert(0, HERE)
import gen  # noqa: E402

SIZES = [100, 1000, 10000]
PROBE_UNIT = 1  # m1: its callers live in m1..m4, present from N=100 on


def ws(work, n):
    # Same path length in every size, so `file` keys do not differ by a byte.
    return os.path.join(work, f"n{n:05d}")


def generate(work, n):
    out = subprocess.run([sys.executable, os.path.join(HERE, "gen.py"), ws(work, n), "--defs", str(n)],
                         check=True, capture_output=True, text=True).stdout
    return json.loads(out)


def timed(cmd):
    before = resource.getrusage(resource.RUSAGE_CHILDREN)
    t = time.perf_counter()
    p = subprocess.run(cmd, capture_output=True, text=True)
    wall = time.perf_counter() - t
    after = resource.getrusage(resource.RUSAGE_CHILDREN)
    return p, wall, after.ru_utime - before.ru_utime + after.ru_stime - before.ru_stime


def verify_run(work, n, phase):
    d = ws(work, n)
    p, wall, cpu = timed([BMB, "verify", "--cache", os.path.join(d, ".bmb-cache"), d])
    rows = [json.loads(x) for x in p.stdout.splitlines() if x.startswith('{"def"')]
    return {
        "kind": "verify", "size": n, "phase": phase, "exit": p.returncode, "wall_s": round(wall, 3),
        "child_cpu_s": round(cpu, 3), "defs_with_status": len(rows),
        "verified": sum(r["status"] == "verified" for r in rows),
        "obligations": sum(r["obligations"] for r in rows),
    }


def touch_leaf(work, n):
    """Changes the branch condition of the first `shift` leaf in m1 (its
    contract still holds, so the definition stays proved)."""
    path = os.path.join(ws(work, n), "src", f"m{PROBE_UNIT}.bmb")
    text = open(path, encoding="utf-8").read()
    m = re.search(r"fn (shift)_\d+\(x: i64\)[^{]*\{\n        if x > (\d+) \{", text)
    if not m:
        sys.exit("scale.py: no shift leaf to touch in m1")
    i = m.start(2)
    c = int(m.group(2))
    text = text[:i] + str(c + 1) + text[i + len(m.group(2)):]
    open(path, "w", encoding="utf-8").write(text)
    return m.group(0).split("(")[0][3:]


def cmd_verify(a):
    os.makedirs(a.work, exist_ok=True)
    rows = []
    for n in a.sizes:
        g = generate(a.work, n)
        rows.append({"kind": "gen", "size": n, **g,
                     "bytes": sum(os.path.getsize(os.path.join(ws(a.work, n), "src", f))
                                  for f in os.listdir(os.path.join(ws(a.work, n), "src")))})
        rows.append(verify_run(a.work, n, "cold"))
        rows.append(verify_run(a.work, n, "warm"))
        leaf = touch_leaf(a.work, n)
        r = verify_run(a.work, n, "incremental")
        r["changed"] = leaf
        rows.append(r)
        generate(a.work, n)  # restore the generated text and leave the cache warm for it
        rows.append(verify_run(a.work, n, "restore"))
        for r in rows[-5:]:
            print(json.dumps(r, ensure_ascii=False), flush=True)
    write(a.out, rows)


class Mcp:
    def __init__(self, root):
        self.p = subprocess.Popen([BMB, "mcp", root], stdin=subprocess.PIPE, stdout=subprocess.PIPE,
                                  text=True, encoding="utf-8")
        self.id = 0
        self.rpc("initialize", {})

    def rpc(self, method, params):
        self.id += 1
        self.p.stdin.write(json.dumps({"jsonrpc": "2.0", "id": self.id, "method": method, "params": params}) + "\n")
        self.p.stdin.flush()
        return json.loads(self.p.stdout.readline())

    def call(self, tool, args):
        r = self.rpc("tools/call", {"name": tool, "arguments": args})["result"]
        if r["isError"]:
            raise RuntimeError(f"{tool} {args}: {r['content'][0]['text'][:300]}")
        return r["content"][0]["text"]

    def close(self):
        self.p.stdin.close()
        self.p.wait()


def probes():
    """Fixed probe definitions of unit m1, by role. Same names in every size."""
    p = gen.parse(["-", "--defs", "20"])
    unit = gen.Unit(PROBE_UNIT, p)
    first = lambda pred: next(n for n, k, info in unit.defs if pred(k, info))  # noqa: E731
    u = f"m{PROBE_UNIT}::"
    return {
        "chain_top": u + first(lambda k, i: k == "chain" and i["layer"] == p.depth - 1),
        "leaf": u + first(lambda k, i: k == "leaf"),
        "remap": u + first(lambda k, i: k == "remap"),
        "reset": u + first(lambda k, i: k == "reset"),
        "show": u + first(lambda k, i: k == "show"),
        "spec_inb": u + "inb",
    }


def children_cpu():
    r = resource.getrusage(resource.RUSAGE_CHILDREN)
    return r.ru_utime + r.ru_stime


def proc_cpu(pid):
    """CPU seconds (user + system) of a running process, from /proc."""
    f = open(f"/proc/{pid}/stat").read().rsplit(")", 1)[1].split()
    return (int(f[11]) + int(f[12])) / os.sysconf("SC_CLK_TCK")


def measure(fn, runs, cpu=children_cpu):
    """Output, wall times and CPU times of RUNS calls after one warm-up call
    (fills OS caches; the proof cache is already warm). CPU time is less
    sensitive than wall time to other load on the machine (10 ms ticks)."""
    fn()
    times, cpus = [], []
    for _ in range(runs):
        c, t = cpu(), time.perf_counter()
        out = fn()
        times.append(time.perf_counter() - t)
        cpus.append(cpu() - c)
    return out, times, cpus


def cmd_queries(a):
    rows = []
    pr = probes()
    for n in a.sizes:
        d = ws(a.work, n)
        if not os.path.isdir(os.path.join(d, ".bmb-cache")):
            sys.exit(f"scale.py: {d} has no .bmb-cache; run `scale.py verify` first")

        def rec(**kw):
            out, times, cpus = kw.pop("m")
            b = len(out.encode("utf-8"))
            r = {"kind": "query", "size": n, **kw, "bytes": b, "tokens": math.ceil(b / 4),
                 "lines": out.count("\n"), "median_s": round(statistics.median(times), 4),
                 "min_s": round(min(times), 4), "cpu_median_s": round(statistics.median(cpus), 3),
                 "runs_s": [round(t, 4) for t in times]}
            rows.append(r)
            print(json.dumps(r, ensure_ascii=False), flush=True)

        # Front-end baseline: whole-workspace parse + check, one process per call.
        rec(query="cli check", probe=None,
            m=measure(lambda: subprocess.run([BMB, "check", d], capture_output=True, text=True, check=True).stdout, a.runs))
        s = Mcp(d)
        sc = lambda: proc_cpu(s.p.pid)  # noqa: E731
        rec(query="mcp tools/list", probe=None, m=measure(lambda: json.dumps(s.rpc("tools/list", {})), a.runs, sc))
        rec(query="map L0", probe=None, m=measure(lambda: s.call("map", {"level": "L0"}), a.runs, sc))
        # A bare name repeated in every unit: the answer grows with N because
        # the matches do (scope, not codebase size, drives it).
        rec(query="find inb", probe=None, m=measure(lambda: s.call("query", {"name": "find", "args": ["inb"]}), a.runs, sc))
        for role, p in pr.items():
            plan = [("find", [p]), ("slice", [p]), ("surface", [p]), ("edges", [p]),
                    ("edges", [p, "both"]), ("impact", [p]), ("effects", [p])]
            if role == "spec_inb":
                plan = [x for x in plan if x[0] not in ("slice", "effects")]
            for q, args in plan:
                rec(query=" ".join([q] + args[1:]), probe=role, path=p,
                    m=measure(lambda q=q, args=args: s.call("query", {"name": q, "args": args}), a.runs, sc))
        after_commit(s, sc, pr, rec, a.runs, d)
        s.close()
    write(a.out, rows)


def after_commit(s, sc, pr, rec, runs, d):
    """Cost of a write and of the first read after it (backlog 3A-03): RUNS
    times, commit a body change of the first `shift` leaf of m1 (its
    branch constant, alternately +1 and back, through tx_put), then run
    `slice` and `surface` of the chain_top probe once. The server's
    front-end cache sees one changed file each time. Ends on the original
    text."""
    text = open(os.path.join(d, "src", f"m{PROBE_UNIT}.bmb"), encoding="utf-8").read()
    m = re.search(r"fn (shift_\d+)\(x: i64\)[^{]*\{\n        if x > \d+ \{", text)
    if not m:
        sys.exit("scale.py: no shift leaf in m1")
    leaf = f"m{PROBE_UNIT}::{m.group(1)}"
    code = json.loads(s.call("query", {"name": "slice", "args": [leaf]}).splitlines()[1])["code"]
    c = re.search(r"if x > (\d+) \{", code)
    if not c:
        sys.exit(f"scale.py: no branch constant in {leaf}")
    variants = [code[:c.start(1)] + str(int(c.group(1)) + 1) + code[c.end(1):], code]

    def commit(text):
        tx = json.loads(s.call("tx_begin", {}))["tx"]
        s.call("tx_put", {"tx": tx, "unit": f"m{PROBE_UNIT}", "code": text})
        return s.call("tx_commit", {"tx": tx})

    samples = {"tx_commit": [], "slice": [], "surface": []}
    outs = {}
    for k in range(runs + (runs % 2)):
        for name, fn in [("tx_commit", lambda k=k: commit(variants[k % 2])),
                         ("slice", lambda: s.call("query", {"name": "slice", "args": [pr["chain_top"]]})),
                         ("surface", lambda: s.call("query", {"name": "surface", "args": [pr["chain_top"]]}))]:
            c0, t0 = sc(), time.perf_counter()
            outs[name] = fn()
            samples[name].append((time.perf_counter() - t0, sc() - c0))
    for name, xs in samples.items():
        q = name if name == "tx_commit" else f"{name} after commit"
        rec(query=q, probe="chain_top", path=leaf if name == "tx_commit" else pr["chain_top"],
            m=(outs[name], [t for t, _ in xs], [c for _, c in xs]))


def machine():
    cpu = ""
    try:
        cpu = next(x.split(":", 1)[1].strip() for x in open("/proc/cpuinfo") if x.startswith("model name"))
    except (OSError, StopIteration):
        pass
    z3 = subprocess.run(["z3", "--version"], capture_output=True, text=True).stdout.strip()
    rev = subprocess.run(["git", "-C", ROOT, "rev-parse", "--short", "HEAD"], capture_output=True, text=True).stdout.strip()
    return {"kind": "machine", "cpu": cpu, "cpus": os.cpu_count(), "os": platform.platform(),
            "python": platform.python_version(), "z3": z3, "commit": rev,
            "loadavg": open("/proc/loadavg").read().split()[:3] if os.path.exists("/proc/loadavg") else None}


def write(path, rows):
    if not path:
        return
    with open(path, "a", encoding="utf-8") as f:
        f.write(json.dumps(machine(), ensure_ascii=False) + "\n")
        for r in rows:
            f.write(json.dumps(r, ensure_ascii=False) + "\n")


def cmd_table(a):
    rows = [json.loads(x) for f in a.files for x in open(f, encoding="utf-8") if x.strip()]
    sizes = sorted({r["size"] for r in rows if "size" in r})
    gens = {r["size"]: r for r in rows if r["kind"] == "gen"}
    if gens:
        print("| N | 단위 | spec fn | 실행 fn | 원문 바이트 | 의무 | 콜드 verify(s) | 웜 verify(s) | 증분 verify(s) |")
        print("| --- | --- | --- | --- | --- | --- | --- | --- | --- |")
        v = {(r["size"], r["phase"]): r for r in rows if r["kind"] == "verify"}
        for n in sizes:
            if n in gens:
                g = gens[n]
                print(f"| {n} | {g['units']} | {g['spec_fns']} | {g['exec_fns']} | {g['bytes']} | "
                      f"{v[(n, 'cold')]['obligations']} | {v[(n, 'cold')]['wall_s']:.1f} | "
                      f"{v[(n, 'warm')]['wall_s']:.2f} | {v[(n, 'incremental')]['wall_s']:.2f} |")
        print()
    q = [r for r in rows if r["kind"] == "query"]
    if q:
        keys = []
        for r in q:
            k = (r["probe"], r["query"])
            if k not in keys:
                keys.append(k)
        by = {(r["probe"], r["query"], r["size"]): r for r in q}
        head = " | ".join(f"{n} 바이트 | {n} 토큰 | {n} ms | {n} CPU ms" for n in sizes)
        print(f"| 프로브 | 질의 | {head} |")
        print("| --- | --- | " + " | ".join("---" for _ in range(4 * len(sizes))) + " |")
        for p, name in keys:
            cells = []
            for n in sizes:
                r = by.get((p, name, n))
                cells += ([str(r["bytes"]), str(r["tokens"]), f"{r['median_s'] * 1000:.0f}",
                           f"{r['cpu_median_s'] * 1000:.0f}"] if r else ["", "", "", ""])
            print(f"| {p or '-'} | {name} | " + " | ".join(cells) + " |")


def main():
    p = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    sub = p.add_subparsers(dest="cmd", required=True)
    for name in ("verify", "queries"):
        s = sub.add_parser(name)
        s.add_argument("--work", required=True)
        s.add_argument("--sizes", type=int, nargs="+", default=SIZES)
        s.add_argument("--out")
        if name == "queries":
            s.add_argument("--runs", type=int, default=5)
    t = sub.add_parser("table")
    t.add_argument("files", nargs="+")
    a = p.parse_args()
    {"verify": cmd_verify, "queries": cmd_queries, "table": cmd_table}[a.cmd](a)


if __name__ == "__main__":
    main()
