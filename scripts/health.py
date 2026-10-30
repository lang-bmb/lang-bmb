#!/usr/bin/env python3
"""Project health metrics and their ratchet (ADR-0039, dev-docs/HEALTH.md).

Computes the machine-checkable metrics on four axes (V: proof trust,
S: self-containment, P: practicality, O: process) and compares them with the
committed baseline `dev-docs/health/baseline.json`:

    python3 scripts/health.py            # check: exit 1 on a violated rule
    python3 scripts/health.py table      # current values as a Markdown table
    python3 scripts/health.py accept --reason "why"   # record current values

A rule either holds always (`fixed`: e.g. no external crates; `fixed_max`,
`fixed_below`: a bound) or compares with
the baseline (`no_worse`: may only improve; `budget`: may grow by at most the
given fraction). A metric that got worse can only be moved by `accept`, which
writes the reason into the baseline's history (reviewed like any commit).
Needs `target/debug/bmb` (built by the gate) and the pinned toolchain.
Standard library only.
"""
import datetime
import json
import os
import re
import subprocess
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
BMB = ROOT / "target" / "debug" / "bmb"
BASELINE = ROOT / "dev-docs" / "health" / "baseline.json"

# Proof corpora whose cost is tracked: the language corpus and the bench seeds.
PROOF_SETS = ["tests/verify/pass", "tests/run", "bench/m1/seed/src", "bench/m1b/seed/src"]
# Floating-point run programs whose proof, IR and output V6 also repeats
# (ADR-0061 §6: no fast-math, no contraction, the same bits every time).
FLOAT_PROGRAMS = ["tests/run/floats.bmb", "tests/run/mandel.bmb"]
# The libraries and the programs built on them (V10, backlog T2-08).
LIBRARIES = "lib"
LIBRARY_PROGRAMS = "bench/game"
ALLOWED_RUNTIME = {"linux-vdso.so.1", "libc.so.6", "ld-linux-x86-64.so.2"}


def run(args, **kw):
    return subprocess.run(args, cwd=ROOT, capture_output=True, text=True, **kw)


def toolchain_rlimit():
    for line in (ROOT / "scripts" / "toolchain.env").read_text().splitlines():
        if line.startswith("Z3_RLIMIT="):
            return int(line.split("=", 1)[1])
    raise SystemExit("toolchain.env has no Z3_RLIMIT")


def verify_rows(files):
    out = run([str(BMB), "verify", *files]).stdout
    return out, [json.loads(l) for l in out.splitlines() if l.startswith('{"def"')]


def proof_metrics():
    rows, outs = [], []
    for d in PROOF_SETS:
        files = sorted(str(p.relative_to(ROOT)) for p in (ROOT / d).glob("*.bmb"))
        # tests/verify/pass and tests/run hold independent programs; seeds are one workspace each.
        groups = [files] if d.startswith("bench/") else [[f] for f in files]
        for g in groups:
            out, r = verify_rows(g)
            outs.append(out)
            rows += r
    again = []
    for d in PROOF_SETS[2:]:
        files = sorted(str(p.relative_to(ROOT)) for p in (ROOT / d).glob("*.bmb"))
        again.append(verify_rows(files)[0])
    deterministic = again == outs[-len(again):] and floats_deterministic()
    worst = max(rows, key=lambda r: r["rlimit_max"])
    return {
        "V3_unproved_defs": sum(r["status"] != "verified" for r in rows),
        "V4_rlimit_total": sum(r["rlimit"] for r in rows),
        "V5_rlimit_headroom": round(worst["rlimit_max"] / toolchain_rlimit(), 4),
        "V6_deterministic_verify": deterministic,
        "_proved_defs": len(rows),
        "_worst_obligation": worst["def"],
    }


def floats_deterministic():
    """V6 for floating point (ADR-0061 §6): each `FLOAT_PROGRAMS` entry
    proves with the same output, lowers to the same IR and, built twice,
    prints the same bytes, which are its expected output."""
    with tempfile.TemporaryDirectory() as t:
        for f in FLOAT_PROGRAMS:
            seen = []
            for k in range(2):
                ll, exe = os.path.join(t, f"{k}.ll"), os.path.join(t, f"{k}")
                proof = run([str(BMB), "verify", f]).stdout
                r = run([str(BMB), "emit-llvm", f, "-o", ll])
                b = run(["clang", "-O2", "-Wno-override-module", "-o", exe, ll])
                if r.returncode != 0 or b.returncode != 0:
                    return False
                seen.append((proof, open(ll).read(), run([exe]).stdout))
            want = (ROOT / f).with_suffix(".out").read_text()
            if seen[0] != seen[1] or seen[0][2] != want:
                return False
    return True


def bench_load():
    """V8, V9 (backlog INF-11): the proof cost of the bench load, which V4 and
    V5 do not see. The m1b and BVB reference solutions run through their
    harness (`--agent oracle`), whose results carry the `rlimit` of the judged
    workspace and of the BSB kill checks. V8 sums it; V9 counts definitions
    whose costliest obligation reached the solver limit (they are proved only
    by the ground-instance retry, or fail; 1B-37's first version made two BSB
    kill checks do that)."""
    limit = toolchain_rlimit()
    total = at_limit = 0
    for s in ("bench/m1b", "bench/bvb"):
        with tempfile.NamedTemporaryFile(suffix=".jsonl") as f:
            r = run([sys.executable, "bench/m1/harness.py", "--set", s, "run", "--agent", "oracle", "--jobs", "2",
                     "--out", f.name], env=dict(os.environ, BMB=str(BMB)))
            if r.returncode != 0:
                raise SystemExit(f"health: {s} oracle run failed\n{r.stderr[-1000:]}")
            for line in open(f.name):
                row = json.loads(line)
                if row.get("kind") != "write":
                    continue
                total += row.get("rlimit_total", 0) + row.get("bsb_rlimit_total", 0)
                at_limit += (row.get("rlimit_max", 0) >= limit) + (row.get("bsb_rlimit_max", 0) >= limit)
    return {"V8_bench_rlimit_total": total, "V9_bench_defs_at_limit": at_limit}


def library_cost():
    """V10 (backlog T2-08): the proof cost of the libraries and of the
    Benchmarks Game programs that use them, which V4 does not see (T2-05's
    probe moved `lib/*.bmb` by +20% and no metric moved). Each library is
    proved alone and each game program with every library, as the tests
    `libraries_are_proved` and `game_programs_are_proved` do; a game
    program's rows count only its own definitions, the libraries' are
    counted once, alone. One proof cache serves every run: a library's
    obligations are the same queries in a program, and a cached obligation
    reports the `rlimit` it was proved with."""
    libs = sorted(str(p.relative_to(ROOT)) for p in (ROOT / LIBRARIES).glob("*.bmb"))
    games = sorted(str(p.relative_to(ROOT)) for p in (ROOT / LIBRARY_PROGRAMS).glob("*.bmb"))
    rows, lib_defs = [], set()
    with tempfile.TemporaryDirectory() as cache:
        for lib in libs:
            r = verify_rows(["--cache", cache, lib])[1]
            lib_defs |= {x["def"] for x in r}
            rows += r
        for game in games:
            rows += [x for x in verify_rows(["--cache", cache, game, *libs])[1] if x["def"] not in lib_defs]
    worst = max(rows, key=lambda r: r["rlimit_max"])
    return {
        "V10_library_rlimit_total": sum(r["rlimit"] for r in rows),
        "_library_unproved_defs": sum(r["status"] != "verified" for r in rows),
        "_library_worst_obligation": f"{worst['def']} {round(worst['rlimit_max'] / toolchain_rlimit(), 4)}",
    }


def solver_stability():
    """V7: the share of obligations whose verdict changes under meaning-
    preserving query changes (backlog 1B-40, scripts/solver_stability.py)."""
    sys.path.insert(0, str(ROOT / "scripts"))
    import solver_stability
    r = solver_stability.measure(solver_stability.default_paths(), 1, str(BMB))
    return {"V7_unstable_queries": r["ratio"], "_stability_obligations": r["obligations"]}


def diag_coverage():
    codes = [json.loads(l)["code"] for l in run([str(BMB), "codes"]).stdout.splitlines()]
    expected = "".join(p.read_text() for p in (ROOT / "tests").rglob("*.jsonl"))
    rust = ""
    for p in (ROOT / "compiler" / "tests").glob("*.rs"):
        rust += p.read_text()
    for p in (ROOT / "compiler" / "src").rglob("*.rs"):
        s = p.read_text()
        if "#[cfg(test)]" in s:
            rust += s[s.index("#[cfg(test)]"):]
    missing = [c for c in codes if c not in expected and c not in rust]
    return {"V1_untested_diag_codes": len(missing), "_diag_codes": len(codes), "_untested": missing}


def helpful_errors():
    total = helped = 0
    for p in (ROOT / "tests").rglob("*.jsonl"):
        for line in p.read_text().splitlines():
            if not line.startswith("{"):
                continue
            r = json.loads(line)
            if r.get("severity") != "error":
                continue
            total += 1
            m = r.get("message", "")
            helped += "counterexample" in m or "fix" in r or "hint" in m
    return {"P3_errors_with_help": round(helped / total, 4) if total else 0.0}


def self_containment():
    cargo = (ROOT / "compiler" / "Cargo.toml").read_text()
    deps = cargo.split("[dependencies]", 1)[1].split("\n[", 1)[0] if "[dependencies]" in cargo else ""
    crates = [l for l in deps.splitlines() if l.strip() and not l.strip().startswith("#")]
    procs, rust_lines = set(), 0
    # Every module, also those in subdirectories (`src/verify/`).
    for p in sorted((ROOT / "compiler" / "src").rglob("*.rs")):
        s = p.read_text()
        body = s[: s.index("#[cfg(test)]")] if "#[cfg(test)]" in s else s
        rust_lines += sum(1 for l in body.splitlines() if l.strip() and not l.strip().startswith("//"))
        procs |= set(re.findall(r'Command::new\("([^"]+)"\)', body))
    # toolchain.rs runs the pinned tools through a variable (version checks).
    api_bound = sum("bmb::" in p.read_text() for p in (ROOT / "compiler" / "tests").glob("*.rs"))
    selfhost = ROOT / "selfhost"
    bmb_compiler_lines = sum(len(p.read_text().splitlines()) for p in selfhost.rglob("*.bmb")) if selfhost.exists() else 0
    with tempfile.TemporaryDirectory() as t:
        exe = os.path.join(t, "basics")
        r = run([str(BMB), "build", "tests/run/basics.bmb", "-o", exe])
        libs = set()
        if r.returncode == 0:
            for line in run(["ldd", exe]).stdout.splitlines():
                name = line.strip().split(" ")[0]
                libs.add(os.path.basename(name))
    return {
        "S1_external_crates": len(crates),
        "S2_external_processes": sorted(procs),
        "S3_runtime_libs_outside_libc": sorted(libs - ALLOWED_RUNTIME) if libs else ["<build failed>"],
        "S4_stage0_rust_lines": rust_lines,
        "S5_tests_bound_to_rust_api": api_bound,
        "S6_selfhost_bmb_lines": bmb_compiler_lines,
    }


def practicality():
    # `tests/lib_run` programs run with every `lib/` file (ADR-0044 §4, T4-02).
    runnable = [
        p
        for d in ("run", "lib_run")
        for p in (ROOT / "tests" / d).glob("*.bmb")
        if p.with_suffix(".out").exists()
    ]
    bmb_lines = 0
    for p in ROOT.rglob("*.bmb"):
        rel = p.relative_to(ROOT).parts
        if rel[0] in ("target", ".git", ".claude"):
            continue
        # Generated copies of m1b and BVB (bench/hname/gen.py T1-04, bench/edit/gen.py T1-06) are not code anyone wrote.
        if rel[:2] in (("bench", "m1b-alpha"), ("bench", "m1b-mislead"), ("bench", "m1b-write"),
                       ("bench", "bvb-write")):
            continue
        bmb_lines += len(p.read_text().splitlines())
    return {"P1_runnable_programs": len(runnable), "P2_bmb_lines": bmb_lines}


def binary_size():
    """Stripped size of the baseline benchmark programs (ADR-0041: binary size lowest)."""
    total = 0
    with tempfile.TemporaryDirectory() as t:
        for src in sorted((ROOT / "bench" / "baseline").glob("*.bmb")):
            exe = os.path.join(t, src.stem)
            if run([str(BMB), "build", str(src.relative_to(ROOT)), "-o", exe]).returncode != 0:
                return {"B1_baseline_binary_bytes": -1}
            run(["strip", exe])
            total += os.path.getsize(exe)
    return {"B1_baseline_binary_bytes": total}


def process():
    text = (ROOT / "dev-docs" / "BACKLOG.md").read_text()
    rows = [l for l in text.splitlines() if re.match(r"^\| ~?(?:\d|INF)[A-Z]?-\d{2}[a-z]? ", l)]
    done = sum("| ✅" in l for l in rows)
    blocked = sum("| 🚫" in l for l in rows)
    adr_dir = ROOT / "dev-docs" / "adr"
    files = {p.name[:4] for p in adr_dir.glob("[0-9][0-9][0-9][0-9]-*.md")}
    index = set(re.findall(r"^\| \[(\d{4})\]\(", (adr_dir / "README.md").read_text(), re.M))
    return {
        "O1_adr_index_mismatch": len(files ^ index),
        "O2_soundness_fuzzers": len(list((ROOT / "compiler" / "tests").glob("soundness*.rs"))),
        "_backlog": f"{done}/{len(rows)} done, {blocked} blocked",
    }


# metric -> (rule, parameter). Metrics starting with "_" are context only.
RULES = {
    "V1_untested_diag_codes": ("fixed", 0),
    "V3_unproved_defs": ("fixed", 0),
    "V4_rlimit_total": ("budget", 0.10),
    "V5_rlimit_headroom": ("fixed_max", 0.20),
    "V6_deterministic_verify": ("fixed", True),
    # Mariposa's measured rate for Dafny and F* projects (H-STABLE, ADR-0040 §4).
    "V7_unstable_queries": ("fixed_below", 0.026),
    "V8_bench_rlimit_total": ("budget", 0.10),
    "V9_bench_defs_at_limit": ("no_worse_down", None),
    "V10_library_rlimit_total": ("budget", 0.10),
    "S1_external_crates": ("fixed", 0),
    "S2_external_processes": ("same", None),
    "S3_runtime_libs_outside_libc": ("fixed", []),
    "S4_stage0_rust_lines": ("report", None),
    "S5_tests_bound_to_rust_api": ("no_worse_down", None),
    "S6_selfhost_bmb_lines": ("no_worse_up", None),
    "P1_runnable_programs": ("no_worse_up", None),
    "P2_bmb_lines": ("no_worse_up", None),
    "P3_errors_with_help": ("no_worse_up", None),
    "B1_baseline_binary_bytes": ("budget", 0.05),
    "O1_adr_index_mismatch": ("fixed", 0),
    "O2_soundness_fuzzers": ("no_worse_up", None),
}


def compute():
    m = {}
    for f in (diag_coverage, proof_metrics, library_cost, solver_stability, bench_load, helpful_errors,
              self_containment, practicality, binary_size, process):
        m.update(f())
    return m


def judge(m, base):
    bad, better = [], []
    for k, (rule, p) in RULES.items():
        v, b = m[k], base.get(k)
        if rule == "fixed" and v != p:
            bad.append(f"{k} = {v!r}, must be {p!r}")
        elif rule == "fixed_max" and v > p:
            bad.append(f"{k} = {v}, must be <= {p}")
        elif rule == "fixed_below" and v >= p:
            bad.append(f"{k} = {v}, must be < {p}")
        elif b is None:
            continue
        elif rule == "same" and v != b:
            bad.append(f"{k} = {v!r}, baseline {b!r} (a new external dependency needs an ADR)")
        elif rule == "budget" and v > b * (1 + p):
            bad.append(f"{k} = {v}, more than {int(p * 100)}% above baseline {b}")
        elif rule == "no_worse_up" and v < b:
            bad.append(f"{k} = {v}, below baseline {b}")
        elif rule == "no_worse_down" and v > b:
            bad.append(f"{k} = {v}, above baseline {b}")
        if b is not None and ((rule == "no_worse_up" and v > b) or (rule in ("no_worse_down", "budget") and v < b)):
            better.append(f"{k}: {b} -> {v}")
    return bad, better


def load_base():
    return json.loads(BASELINE.read_text()) if BASELINE.exists() else {"metrics": {}, "history": []}


def table(m, base):
    lines = ["| 지표 | 현재 | 기준선 | 규칙 |", "| --- | --- | --- | --- |"]
    for k, (rule, p) in RULES.items():
        b = base["metrics"].get(k, "—")
        r = rule if p is None else f"{rule} {p}"
        lines.append(f"| {k} | {m[k]} | {b} | {r} |")
    ctx = ", ".join(f"{k[1:]}={m[k]}" for k in m if k.startswith("_") and k != "_untested")
    return "\n".join(lines) + f"\n\n맥락: {ctx}"


def main():
    if not BMB.exists():
        raise SystemExit("target/debug/bmb is missing: run cargo build first")
    cmd = sys.argv[1] if len(sys.argv) > 1 else "check"
    m = compute()
    base = load_base()
    if cmd == "table":
        print(table(m, base))
        return 0
    if cmd == "accept":
        if "--reason" not in sys.argv:
            raise SystemExit("accept needs --reason \"...\"")
        reason = sys.argv[sys.argv.index("--reason") + 1]
        changed = {k: [base["metrics"].get(k), m[k]] for k in RULES if base["metrics"].get(k) != m[k]}
        bad, _ = judge(m, {})  # fixed rules can never be accepted away
        if bad:
            raise SystemExit("cannot accept, fixed rules fail:\n  " + "\n  ".join(bad))
        base["metrics"] = {k: m[k] for k in RULES}
        base["history"].append({"date": datetime.date.today().isoformat(), "reason": reason, "changed": changed})
        BASELINE.parent.mkdir(parents=True, exist_ok=True)
        BASELINE.write_text(json.dumps(base, ensure_ascii=False, indent=2) + "\n")
        print(f"baseline updated ({len(changed)} metrics changed)")
        return 0
    bad, better = judge(m, base["metrics"])
    missing = [k for k in RULES if k not in base["metrics"]]
    if missing:
        bad.append(f"baseline has no value for {', '.join(missing)}: run `health.py accept --reason ...`")
    for b in better:
        print(f"improved: {b} (record it with `health.py accept --reason ...`)")
    for b in bad:
        print(f"health: {b}", file=sys.stderr)
    if m["_untested"]:
        print(f"untested diagnostic codes: {m['_untested']}", file=sys.stderr)
    return 1 if bad else 0


if __name__ == "__main__":
    sys.exit(main())
