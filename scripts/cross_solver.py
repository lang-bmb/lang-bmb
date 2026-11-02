#!/usr/bin/env python3
"""Second-solver cross-check (backlog 1B-39, HEALTH.md §3).

  scripts/cross_solver.py [--jobs N] [--groups a,b,..] [--write FILE]

Every query the proof gate sends to Z3 is sent to cvc5 as well, and the two
answers are compared. The queries are the ones `bmb verify --emit-smt` writes:
for each obligation its counterexample probe, the bit-abstracted query, the
exact query and the ground-instance retry, each a `(reset)` segment of the
definition's script (canonical symbols, ADR-0063). The programs are the ones
the health metrics prove (V4, V8, V10), plus the failing corpus:

  corpus   tests/verify/pass, tests/verify/fail, tests/run, bench/baseline,
           each file alone
  seeds    bench/{m1,m1b,bvb}/seed/src as one workspace each, and each
           BRB-encap task's seed files alone
  lib      lib/*.bmb alone, tests/lib_run and bench/game with every library
  bench    the m1b and BVB reference and weak solutions as the frozen harness
           judges them (`--agent oracle` and `weak`), through a `bmb` wrapper
           that adds `--emit-smt` to each `verify` it runs

Identical segments are asked once. Z3 is the pinned binary (toolchain.env)
on the segment as written, so it reproduces the gate's answer; cvc5 gets the
same text without Z3's options (`rlimit`, seeds), under a deterministic
resource limit per query (`rlimit-per`). A disagreement is one solver saying
`sat` and the other `unsat`. `unknown`, a limit, or a parse error is not a
disagreement and is counted by kind. The check passes when there is no
disagreement. Needs the `cvc5` Python package at CVC5_VERSION (toolchain.env).
"""
import argparse
import collections
import concurrent.futures
import hashlib
import json
import os
import re
import shutil
import subprocess
import sys
import tempfile
import time
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
BMB = ROOT / "target" / "debug" / "bmb"
GROUPS = ["corpus", "seeds", "lib", "bench"]
# Resource units per cvc5 query. Deterministic, unlike a time limit. Measured
# on lib/base64.bmb's queries: 12,000 to 20,000 units a second on the slow
# ones, 95% of the queries under 210,000; so a query stops within about 20 s.
CVC5_RLIMIT = 250_000


def pins():
    env = {}
    for line in (ROOT / "scripts" / "toolchain.env").read_text().splitlines():
        if "=" in line and not line.startswith("#"):
            k, v = line.split("=", 1)
            env[k] = v
    return env


def files(d):
    return sorted(str(p.relative_to(ROOT)) for p in (ROOT / d).glob("*.bmb"))


def programs(group):
    """(name, [files]) for each program of a group, as health.py groups them."""
    if group == "corpus":
        return [(f, [f]) for d in ("tests/verify/pass", "tests/verify/fail", "tests/run", "bench/baseline")
                for f in files(d)]
    if group == "seeds":
        out = [(f"bench/{s}/seed", files(f"bench/{s}/seed/src")) for s in ("m1", "m1b", "bvb")]
        for task in sorted(p for p in (ROOT / "bench/brb-encap/seed").iterdir() if p.is_dir()):
            out += [(f, [f]) for f in files(task.relative_to(ROOT))]
        return out
    if group == "lib":
        libs = files("lib")
        return [(f, [f]) for f in libs] + [(f, [f, *libs]) for d in ("tests/lib_run", "bench/game")
                                           for f in files(d)]
    raise ValueError(group)


WRAPPER = """#!/bin/sh
# bmb, with every `verify` also writing its queries (scripts/cross_solver.py).
if [ "$1" = verify ]; then
  shift
  out="$CROSS_EMIT/$(date +%s%N)-$$"
  args=""
  while [ $# -gt 0 ]; do
    if [ "$1" = --cache ]; then shift 2; continue; fi
    args="$args '$(printf %s "$1" | sed "s/'/'\\\\''/g")'"
    shift
  done
  eval exec "$CROSS_BMB" verify --emit-smt "$out" $args
fi
exec "$CROSS_BMB" "$@"
"""


def emit(group, out_dir, jobs):
    """Write the group's scripts under out_dir; return the number of programs."""
    if group == "bench":
        wrapper = out_dir / "bmb"
        wrapper.write_text(WRAPPER)
        wrapper.chmod(0o755)
        env = dict(os.environ, BMB=str(wrapper), CROSS_BMB=str(BMB), CROSS_EMIT=str(out_dir))
        n = 0
        for s in ("bench/m1b", "bench/bvb"):
            for agent in ("oracle", "weak"):
                with tempfile.NamedTemporaryFile(suffix=".jsonl") as f:
                    r = subprocess.run([sys.executable, "bench/m1/harness.py", "--set", s, "run", "--agent", agent,
                                        "--jobs", str(jobs), "--out", f.name], cwd=ROOT, env=env,
                                       capture_output=True, text=True)
                    if r.returncode != 0:
                        sys.exit(f"cross_solver: {s} {agent} run failed\n{r.stderr[-2000:]}")
                    n += sum(1 for _ in open(f.name))
        return n
    progs = programs(group)

    def one(k):
        name, fs = progs[k]
        d = out_dir / f"{k:04d}"
        subprocess.run([str(BMB), "verify", "--emit-smt", str(d), *fs], cwd=ROOT, capture_output=True, text=True)

    with concurrent.futures.ThreadPoolExecutor(jobs) as ex:
        list(ex.map(one, range(len(progs))))
    return len(progs)


SEGMENT = re.compile(r"^; (.*)\n\(reset\)\n", re.M)


def kind(label):
    if label.startswith("probe of"):
        return "probe"
    if label.startswith("exact query of"):
        return "exact"
    if label.startswith("retry of"):
        return "retry"
    if label.endswith("(bit operations abstracted)"):
        return "abstracted"
    if label.startswith("obligation"):
        return "obligation"
    return "other"


def segments(text):
    """(label, segment text) for each `(reset)` segment of a script."""
    marks = list(SEGMENT.finditer(text))
    for i, m in enumerate(marks):
        end = marks[i + 1].start() if i + 1 < len(marks) else len(text)
        yield m.group(1), text[m.end():end]


def answer(lines):
    words = [l.strip() for l in lines if l.strip()]
    for w in words:
        # cvc5 adds the reason to `unknown`: `unknown (RESOURCEOUT)`.
        if w.split(" ")[0] in ("sat", "unsat", "unknown"):
            return w.split(" ")[0]
    return "error: " + " ".join(words)[:200]


def ask_z3(text):
    r = subprocess.run(["z3", "-smt2", "-in"], input=text, capture_output=True, text=True)
    return answer(r.stdout.splitlines())


Z3_ONLY = re.compile(r"^\(set-option :(rlimit|smt\.|sat\.)[^\n]*\n", re.M)


def ask_cvc5(text, limit=CVC5_RLIMIT):
    import cvc5
    tm = cvc5.TermManager()
    s = cvc5.Solver(tm)
    s.setOption("rlimit-per", str(limit))
    # Constant arrays (`(as const (Array ..))`, the encoding of array
    # literals) and nested recursive datatypes (`Box`, `Vec` payloads) are
    # off in cvc5's default configuration.
    s.setOption("arrays-exp", "true")
    s.setOption("dt-nested-rec", "true")
    if "(set-logic " not in text:
        s.setLogic("ALL")
    sm = cvc5.SymbolManager(tm)
    p = cvc5.InputParser(s, sm)
    p.setStringInput(cvc5.InputLanguage.SMT_LIB_2_6, Z3_ONLY.sub("", text), "query")
    out = []
    try:
        while True:
            c = p.nextCommand()
            if c.isNull():
                break
            out.append(c.invoke(s, sm))
    except Exception as e:  # cvc5 raises on parse and solver errors
        return "error: " + str(e).splitlines()[0][:200]
    return answer("\n".join(out).splitlines())


def ask_both(text):
    t = time.monotonic()
    z = ask_z3(text)
    tz = time.monotonic() - t
    c = ask_cvc5(text)
    return z, c, tz, time.monotonic() - t - tz


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--jobs", type=int, default=os.cpu_count())
    ap.add_argument("--groups", default=",".join(GROUPS))
    ap.add_argument("--write", help="also write the report (markdown) to this file")
    ap.add_argument("--keep", help="keep the emitted scripts in this directory")
    a = ap.parse_args()

    want = pins()["CVC5_VERSION"]
    try:
        import cvc5
    except ImportError:
        sys.exit(f"cross_solver: needs the cvc5 Python package ({want}): pip install cvc5=={want}")
    if cvc5.__version__ != want:
        sys.exit(f"cross_solver: cvc5 {cvc5.__version__}, toolchain.env pins {want}")
    z3v = subprocess.run(["z3", "--version"], capture_output=True, text=True).stdout.split()
    if pins()["Z3_VERSION"] not in z3v:
        sys.exit(f"cross_solver: z3 {' '.join(z3v)}, toolchain.env pins {pins()['Z3_VERSION']}")
    if not BMB.exists():
        sys.exit("cross_solver: build bmb first (cargo build)")

    # The commit the queries come from: recorded before anything runs.
    at = commit()
    work = Path(a.keep or tempfile.mkdtemp(prefix="bmb-cross-"))
    groups = a.groups.split(",")
    unique = {}  # segment hash -> (text, kind, first place)
    places = collections.Counter()
    progs = {}
    for g in groups:
        d = work / g
        if d.exists():
            shutil.rmtree(d)
        d.mkdir(parents=True)
        progs[g] = emit(g, d, a.jobs)
        for f in sorted(d.rglob("*.smt2")):
            for label, seg in segments(f.read_text()):
                h = hashlib.sha256(seg.encode()).hexdigest()
                places[g] += 1
                if h not in unique:
                    where = f"{f.relative_to(d).parent.name}/{f.stem} {label}"
                    unique[h] = (seg, kind(label), g, where)
    keys = sorted(unique)
    print(f"cross_solver: {sum(places.values())} queries, {len(keys)} distinct", file=sys.stderr)

    with concurrent.futures.ProcessPoolExecutor(a.jobs) as ex:
        answers = list(ex.map(ask_both, [unique[h][0] for h in keys], chunksize=4))

    table = collections.defaultdict(collections.Counter)
    disagree, errors = [], []
    tz = tc = 0.0
    for h, (z, c, dz, dc) in zip(keys, answers):
        _, k, g, where = unique[h]
        tz, tc = tz + dz, tc + dc
        table[k][(z if not z.startswith("error") else "error", c if not c.startswith("error") else "error")] += 1
        if {z, c} == {"sat", "unsat"}:
            disagree.append((g, where, z, c, h[:12]))
        if z.startswith("error") or c.startswith("error"):
            errors.append((g, where, z, c, h[:12]))

    lines = cell_table(table)
    report = [
        f"# 두 번째 해법기 교차 확인 (1B-39)",
        "",
        f"- 커밋 `{at}`, Z3 {pins()['Z3_VERSION']}, cvc5 {want} (`rlimit-per` {CVC5_RLIMIT:,})",
        f"- 프로그램: " + ", ".join(f"{g} {progs[g]}" for g in groups),
        f"- 질의 {sum(places.values()):,}개(" + ", ".join(f"{g} {places[g]:,}" for g in groups)
        + f"), 서로 다른 것 {len(keys):,}개",
        f"- 시간 합: Z3 {tz:,.0f}초, cvc5 {tc:,.0f}초",
        f"- **불일치(한쪽 `sat`, 다른 쪽 `unsat`): {len(disagree)}**",
        "",
        "질의 종류별 답(Z3 / cvc5):",
        "",
        *lines,
    ]
    if disagree:
        report += ["", "불일치:", "", "| 집합 | 질의 | Z3 | cvc5 | 해시 |", "| --- | --- | --- | --- | --- |"]
        report += [f"| {g} | {w} | {z} | {c} | `{h}` |" for g, w, z, c, h in disagree]
    if errors:
        # By message, symbols erased (`$5`), with one query each.
        groups = collections.defaultdict(list)
        for g, w, z, c, h in errors:
            msg = z if z.startswith("error") else c
            who = "Z3" if z.startswith("error") else "cvc5"
            groups[(who, re.sub(r"\$\d+", "$k", msg))].append((g, w, h))
        report += ["", "오류(해법기가 질의를 받지 못함, 메시지별):", "", "| 해법기 | 메시지 | 질의 수 | 예 |",
                   "| --- | --- | ---: | --- |"]
        for (who, msg), xs in sorted(groups.items(), key=lambda kv: -len(kv[1])):
            g, w, h = xs[0]
            cell = msg.replace("|", "\\|")
            report.append(f"| {who} | {cell} | {len(xs):,} | {g} `{h}` " + w.replace("|", "\\|") + " |")
    text = "\n".join(report) + "\n"
    print(text)
    if a.write:
        Path(a.write).parent.mkdir(parents=True, exist_ok=True)
        Path(a.write).write_text(text)
    if not a.keep:
        shutil.rmtree(work, ignore_errors=True)
    sys.exit(1 if disagree else 0)


def cell_table(table):
    cols = [(z, c) for z in ("unsat", "sat", "unknown", "error") for c in ("unsat", "sat", "unknown", "error")]
    cols = [col for col in cols if any(table[k][col] for k in table)]
    head = "| 종류 | " + " | ".join(f"{z} / {c}" for z, c in cols) + " | 합 |"
    out = [head, "| --- |" + " ---: |" * (len(cols) + 1)]
    for k in ("obligation", "abstracted", "exact", "retry", "probe", "other"):
        if k in table:
            out.append(f"| {k} | " + " | ".join(f"{table[k][col]:,}" for col in cols)
                       + f" | {sum(table[k].values()):,} |")
    return out


def commit():
    c = subprocess.run(["git", "rev-parse", "--short=8", "HEAD"], cwd=ROOT, capture_output=True, text=True)
    d = subprocess.run(["git", "status", "--porcelain"], cwd=ROOT, capture_output=True, text=True)
    return c.stdout.strip() + ("+dirty" if d.stdout.strip() else "")


if __name__ == "__main__":
    main()
