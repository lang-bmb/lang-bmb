#!/usr/bin/env python3
"""One command for the real-model measurements (backlog INF-14).

  run_models.py --agent anthropic:MODEL --price-in P --price-out Q --budget USD [--sets ..]
  run_models.py --dry-run [--limit N] [--sets ..] [--out-dir DIR]

Runs the frozen task sets that need a real model, in a fixed order, with the
existing harnesses (each checks its own MANIFEST and keeps the ADR-0021 rules:
one session per task, no retry):

  m1         bench/m1        G2 (write), G4 navigation              1E-10
  m1b        bench/m1b       G2, G3 (BSB kill rate)                 2A-06
  bvb        bench/bvb       BMB side of H-FAMILIAR, H-SPEC column  2A-08
  bvb-verus  bench/bvb/verus original Verus side (needs Verus)      2A-09
  brb-bmb    bench/brb       G5, H-ORACLE, G8 (BMB arm)             2H-03
  brb-rust   bench/brb       G5, H-ORACLE, G8 (Rust arm)            2H-03

then prints one report with every summary (`m1` summarize, BVB `compare` or
`strength`, BRB `summarize`) and a line per metric saying whether it was filled.

Cost guard. Model prices change, so they are arguments (USD per million input
and output tokens, from the provider's price page). After each set the spend
is computed from the recorded token usage; before a set starts, its cost is
projected as (mean cost per session so far) x (its sessions), and the run stops
before a set that would exceed the budget. The first set has no projection, so
it is the smallest (m1, 15 sessions). A set is never cut in the middle.

--dry-run runs every set with the scripted reference agents (no model, no
cost), `--limit N` tasks per set, into a temporary directory, and checks that
each metric line appears. The gate runs it (backlog INF-14). The Verus side is
skipped when Verus is not installed, so H-FAMILIAR then reads "skipped".

Real runs refuse to start on an uncommitted tree (results from `+dirty`
commits are not reported, ADR-0021), without ANTHROPIC_API_KEY, or with
bvb-verus selected and Verus missing. Standard library only.
"""
import argparse
import datetime
import json
import os
import re
import subprocess
import sys
import tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(HERE)
PY = sys.executable


def json_ids(d):
    tdir = os.path.join(d, "tasks")
    return [json.load(open(os.path.join(tdir, f)))["id"] for f in sorted(os.listdir(tdir))]


def brb_ids():
    tdir = os.path.join(HERE, "brb", "tasks")
    return [re.search(r'^ID = "([^"]+)"', open(os.path.join(tdir, f)).read(), re.M).group(1)
            for f in sorted(os.listdir(tdir)) if f.endswith(".py")]


def m1_run(set_dir):
    return lambda agent, out, tasks, real: [PY, "bench/m1/harness.py", "--set", set_dir, "run", "--agent", agent,
                                            "--out", out] + tasks


def brb_run(arm):
    return lambda agent, out, tasks, real: ([PY, "bench/brb/harness.py", "run", "--arm", arm, "--agent", agent,
                                             "--out", out] + tasks
                                            + (["--proptest", "on" if real else "off"] if arm == "rust" else []))


# name, task ids, argv(agent, out, tasks, real), results directory
SETS = [
    ("m1", lambda: json_ids(os.path.join(HERE, "m1")), m1_run("bench/m1"), "bench/m1/results"),
    ("m1b", lambda: json_ids(os.path.join(HERE, "m1b")), m1_run("bench/m1b"), "bench/m1b/results"),
    ("bvb", lambda: json_ids(os.path.join(HERE, "bvb")), m1_run("bench/bvb"), "bench/bvb/results"),
    ("bvb-verus", lambda: json_ids(os.path.join(HERE, "bvb")),
     lambda agent, out, tasks, real: [PY, "bench/bvb/verus/harness.py", "run", "--agent", agent, "--out", out] + tasks,
     "bench/bvb/results"),
    ("brb-bmb", brb_ids, brb_run("bmb"), "bench/brb/results"),
    ("brb-rust", brb_ids, brb_run("rust"), "bench/brb/results"),
]
NAMES = [s[0] for s in SETS]


def verus_installed():
    r = subprocess.run([PY, "bench/bvb/verus/harness.py", "check-env"], cwd=ROOT, capture_output=True, text=True)
    return r.returncode == 0


def usage_of(path):
    """(sessions, input tokens, output tokens) recorded in a results file."""
    n = i = o = 0
    for line in open(path):
        r = json.loads(line)
        if r.get("summary"):
            continue
        u = r.get("model_usage") or {}
        n, i, o = n + 1, i + u.get("input_tokens", 0), o + u.get("output_tokens", 0)
    return n, i, o


def cost(tokens_in, tokens_out, price_in, price_out):
    return tokens_in / 1e6 * price_in + tokens_out / 1e6 * price_out


def over_budget(spent, sessions, next_sessions, budget):
    """The reason not to start the next set, or None: its projected cost is
    the mean cost per finished session times its sessions. Before any session
    there is no projection."""
    if not sessions:
        return None
    projected = spent / sessions * next_sessions
    if spent + projected > budget:
        return f"예산: 지출 ${spent:.2f} + 예상 ${projected:.2f} > ${budget:.2f}"
    return None


def report(paths, skipped):
    """The summaries of every finished set, then one line per metric."""
    def sh(*argv):
        r = subprocess.run([PY, *argv], cwd=ROOT, capture_output=True, text=True)
        return r.stdout if r.returncode == 0 else f"(실패: {r.stderr.strip()[-300:]})\n"

    out = []
    for n in ("m1", "m1b", "bvb"):
        if n in paths:
            out += [f"## {n}" + ("(BMB)" if n == "bvb" else ""), "", sh("bench/m1/harness.py", "summarize", paths[n])]
    if "bvb" in paths and "bvb-verus" in paths:
        out += ["## BVB: BMB 대 Verus", "", sh("bench/bvb/verus/harness.py", "compare", paths["bvb"], paths["bvb-verus"])]
    elif "bvb" in paths:
        out += ["## BVB 명세 강도(BMB만)", "", sh("bench/bvb/verus/harness.py", "strength", paths["bvb"])]
    if "brb-bmb" in paths and "brb-rust" in paths:
        out += ["## BRB", "", sh("bench/brb/harness.py", "summarize", paths["brb-bmb"], paths["brb-rust"])]
    text = "\n".join(out)
    sections = {n: text.split(f"## {n}", 1)[1].split("\n## ", 1)[0] for n in ("m1", "m1b") if f"## {n}" in text}
    checks = [
        ("G2 작성 1회 성공률(m1·m1b)", "| 작성 1회 성공률 |", ("m1", "m1b")),
        ("G3 명세 변이체 판별률(m1b, BSB)", "| 명세 변이체 판별률 |", ("m1b",)),
        ("H-SPEC 보조: BVB 명세 강도", "spec_accepted" if "bvb-verus" not in paths else "| 명세 강도(", ("bvb",)),
        ("H-FAMILIAR: BVB 1회 성공률 BMB 대 Verus", "| 1회 성공률 |", ("bvb", "bvb-verus")),
        ("G5 결함률·총 토큰(BRB)", "G5 (", ("brb-bmb", "brb-rust")),
        ("H-ORACLE(BRB)", "H-ORACLE (", ("brb-bmb", "brb-rust")),
        ("G8 테스트 코드(BRB)", "G8 (", ("brb-bmb", "brb-rust")),
    ]
    status, lines = {}, ["## 지표", ""]
    for label, marker, need in checks:
        missing = [n for n in need if n not in paths]
        if missing:
            why = "; ".join(f"{n}: {skipped.get(n, '선택 안 됨')}" for n in missing)
            status[label] = "skipped"
            lines.append(f"- {label}: 건너뜀({why})")
        elif all(marker in sections.get(n, text) for n in need if n in ("m1", "m1b")) and marker in text:
            status[label] = "filled"
            lines.append(f"- {label}: 채움")
        else:
            status[label] = "missing"
            lines.append(f"- {label}: **빠짐**")
    return text + "\n" + "\n".join(lines) + "\n", status


def preflight_real(sets):
    problems = []
    if not os.environ.get("ANTHROPIC_API_KEY"):
        problems.append("ANTHROPIC_API_KEY가 없다")
    if subprocess.run(["git", "status", "--porcelain"], cwd=ROOT, capture_output=True, text=True).stdout.strip():
        problems.append("커밋되지 않은 변경이 있다(+dirty 결과는 보고에 쓰지 않는다)")
    if "bvb-verus" in sets and not verus_installed():
        problems.append("Verus가 없다: bench/bvb/verus/install.sh, 또는 --sets에서 bvb-verus를 뺀다")
    for argv in ([PY, "bench/m1/harness.py", "manifest"], [PY, "bench/m1/harness.py", "--set", "bench/m1b", "manifest"],
                 [PY, "bench/m1/harness.py", "--set", "bench/bvb", "manifest"], [PY, "bench/brb/harness.py", "manifest"]):
        r = subprocess.run(argv, cwd=ROOT, capture_output=True, text=True)
        if r.returncode != 0:
            problems.append(f"{' '.join(argv[1:])}: {r.stderr.strip()[-200:]}")
    return problems


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--agent", help="anthropic:MODEL (real run)")
    ap.add_argument("--dry-run", action="store_true", help="scripted reference agents, no model")
    ap.add_argument("--sets", default=",".join(NAMES), help="comma list, in this order: " + ",".join(NAMES))
    ap.add_argument("--limit", type=int, help="first N tasks of each set (dry run only)")
    ap.add_argument("--price-in", type=float, help="USD per million input tokens")
    ap.add_argument("--price-out", type=float, help="USD per million output tokens")
    ap.add_argument("--budget", type=float, help="USD; no set starts that the projection puts over it")
    ap.add_argument("--out-dir", help="dry run: where results go (default: a temporary directory)")
    ap.add_argument("--report", help="also write the report to this file")
    ap.add_argument("--jobs", type=int, default=2, help="dry run: sessions at a time (real runs: 1)")
    args = ap.parse_args(argv)

    sets = [s.strip() for s in args.sets.split(",") if s.strip()]
    if unknown := [s for s in sets if s not in NAMES]:
        sys.exit(f"unknown sets: {unknown}")
    sets = [n for n in NAMES if n in sets]
    real = not args.dry_run
    if real:
        if not (args.agent and args.agent.startswith("anthropic:")):
            sys.exit("a real run needs --agent anthropic:MODEL (or --dry-run)")
        if None in (args.price_in, args.price_out, args.budget):
            sys.exit("a real run needs --price-in, --price-out and --budget")
        if args.limit:
            sys.exit("--limit is for dry runs: a real run measures whole frozen sets")
        if problems := preflight_real(sets):
            sys.exit("not starting:\n- " + "\n- ".join(problems))
        model = args.agent.split(":", 1)[1]
        stamp = datetime.date.today().isoformat()
    out_dir = args.out_dir or (None if real else tempfile.mkdtemp(prefix="bmb-models-"))

    paths, skipped = {}, {}
    spent = sessions = 0.0
    for name, ids, run_argv, results_dir in SETS:
        if name not in sets:
            continue
        if name == "bvb-verus" and not verus_installed():
            skipped[name] = "Verus 없음"
            print(f"run_models: {name}: skipped (Verus not installed)", file=sys.stderr)
            continue
        task_ids = ids()
        if args.limit:
            task_ids = task_ids[:args.limit]
        if real:
            if why := over_budget(spent, sessions, len(task_ids), args.budget):
                skipped[name] = why
                print(f"run_models: stop before {name}: {why}", file=sys.stderr)
                for rest in sets[sets.index(name) + 1:]:
                    skipped[rest] = "예산으로 멈춤"
                break
            os.makedirs(os.path.join(ROOT, results_dir), exist_ok=True)
            suffix = {"m1": "", "m1b": "", "bvb": "", "bvb-verus": "-verus", "brb-bmb": "-bmb", "brb-rust": "-rust"}[name]
            out = os.path.join(ROOT, results_dir, f"{stamp}-{model}{suffix}.jsonl")
            if os.path.exists(out):
                sys.exit(f"{out} exists: a frozen set is run once (ADR-0021)")
            cmd = run_argv(args.agent, out, [], True)
        else:
            out = os.path.join(out_dir, f"{name}.jsonl")
            cmd = run_argv("oracle", out, ["--tasks", ",".join(task_ids), "--jobs", str(args.jobs)], False)
        print(f"run_models: {name}: {len(task_ids)} sessions", file=sys.stderr)
        r = subprocess.run(cmd, cwd=ROOT, stdout=subprocess.DEVNULL)
        if r.returncode != 0 or not os.path.exists(out):
            sys.exit(f"run_models: {name} failed (exit {r.returncode})")
        paths[name] = out
        n, ti, to = usage_of(out)
        sessions += n
        if real:
            spent += cost(ti, to, args.price_in, args.price_out)
            print(f"run_models: {name}: {n} sessions, {ti} in / {to} out tokens, spent ${spent:.2f}", file=sys.stderr)

    text, status = report(paths, skipped)
    sys.stdout.write(text)
    if args.report:
        with open(args.report, "w") as f:
            f.write(text)
    if any(v == "missing" for v in status.values()):
        sys.exit("run_models: a metric line is missing from the summaries")


if __name__ == "__main__":
    main()
