#!/usr/bin/env python3
"""`#[uses]` default context = r2c, compared at scale (backlog 2R-04a).

Deterministic, offline. 2R-02 (syntax-eval-1.md §2.2) found that the rule
r2c (the closure of the spec fns a definition's own clauses, invariants and
asserts name, together with those its callees' requires and ensures name)
proves three small corpora. This repeats the comparison where the question
of 2R-02 §5 rank 1 lies: a large generated workspace (bench/m3/gen.py) and
solver seeds 0..4. For each corpus and each rule (none, r1c, r2c, actual)
every `#[uses(..)]` is replaced by the rule's set, and

  - `bmb stability --seeds 5`: definitions proved under every seed, those
    whose verdict changes with the seed, the largest rlimit of a definition
    over the seeds, and how many exceed a fifth of the default limit
    (1,000,000: ADR-0092 §3's budget);
  - `bmb verify --emit-smt`: the bytes of the queries the solver receives;
  - the incremental key (ADR-0032, 0033): a definition's key includes the
    definitions its `#[uses]` names, so a rule changes it exactly for the
    definitions whose set differs from the written one (counted).

Usage: bench/syntax/uses_default.py BMB [--write FILE]
Prints one JSON line per corpus and rule; --write appends them.
"""

import glob
import json
import os
import re
import subprocess
import sys
import tempfile

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import measure  # noqa: E402

BUDGET = 1_000_000


def corpora(tmp):
    m3 = os.path.join(tmp, "m3")
    subprocess.run(
        [sys.executable, os.path.join(ROOT, "bench/m3/gen.py"), m3, "--defs", "300", "--seed", "1"],
        check=True,
        capture_output=True,
    )
    return [
        ("m3 300 defs, seed 1", sorted(glob.glob(os.path.join(m3, "src", "*.bmb")))),
        ("m1b seed", sorted(glob.glob(os.path.join(ROOT, "bench/m1b/seed/src/*.bmb")))),
        ("tests/verify/pass", sorted(glob.glob(os.path.join(ROOT, "tests/verify/pass/*.bmb")))),
    ]


def rewrite(read, rows, rule):
    """The files with every #[uses(..)] replaced by `rule`'s set."""
    by_fn = {r["fn"]: r for r in rows}
    out_files = []
    for fname, text in read:
        unit = None
        out = []
        for line in text.split("\n"):
            m = re.match(r"\s*mod (\w+) \{", line)
            if m:
                unit = m.group(1)
            if rule != "actual" and line.strip().startswith("#[uses("):
                continue
            fm = re.match(r"(\s*)(?:proof )?fn (\w+)[<(]", line)
            if fm and rule not in ("none", "actual") and (unit + "::" + fm.group(2)) in by_fn:
                s = by_fn[unit + "::" + fm.group(2)][rule]
                if s:
                    short = [p.split("::", 1)[1] if p.split("::")[0] == unit else p for p in s]
                    out.append(f"{fm.group(1)}#[uses({', '.join(short)})]")
            out.append(line)
        out_files.append((fname, "\n".join(out)))
    return out_files


def measure_rule(bmb, files, tmp):
    d = tempfile.mkdtemp(dir=tmp)
    paths = []
    for fname, text in files:
        p = os.path.join(d, fname)
        with open(p, "w", encoding="utf-8") as w:
            w.write(text)
        paths.append(p)
    st = subprocess.run([bmb, "stability", "--seeds", "5", *paths], capture_output=True, text=True)
    rows = [json.loads(x) for x in st.stdout.splitlines() if x.startswith('{"def"')]
    proved = sum(1 for r in rows if all(v == "verified" for v in r["verdicts"]))
    unstable = sum(1 for r in rows if not r["stable"])
    rmax = max((r["rlimit_max"] for r in rows), default=0)
    smt = os.path.join(d, "smt")
    subprocess.run([bmb, "verify", "--emit-smt", smt, *paths], capture_output=True, text=True)
    qbytes = sum(os.path.getsize(f) for f in glob.glob(os.path.join(smt, "*.smt2")))
    # A definition's rlimit is the sum over its obligations; the budget is
    # per obligation, so take verify's rlimit_max at seed 0 for that count.
    v = subprocess.run([bmb, "verify", *paths], capture_output=True, text=True)
    vrows = [json.loads(x) for x in v.stdout.splitlines() if x.startswith('{"def"')]
    over = sum(1 for r in vrows if r.get("rlimit_max", 0) > BUDGET)
    total = sum(r.get("rlimit", 0) for r in vrows)
    return {
        "defs": len(rows),
        "proved_all_seeds": proved,
        "unstable": unstable,
        "rlimit_total_seed0": total,
        "def_rlimit_max_over_seeds": rmax,
        "obligations_over_budget": over,
        "query_bytes": qbytes,
    }


def run_corpus(bmb, label, programs, tmp, write):
    """`programs`: lists of paths, each one program. A program the
    approximate M1 parser of measure.py cannot read is skipped and counted."""
    parsed = []
    skipped = 0
    for paths in programs:
        read = [(os.path.basename(p), open(p, encoding="utf-8").read()) for p in paths]
        try:
            _, items, _ = measure.tally([(None, t) for _, t in read])
        except (ValueError, KeyError, IndexError):
            skipped += 1
            continue
        parsed.append((read, measure.uses_rules(items)))
    for rule in ("actual", "none", "r1c", "r2c"):
        tot = {}
        fns = 0
        for read, rows in parsed:
            r = measure_rule(bmb, rewrite(read, rows, rule), tmp)
            if rule == "actual":
                r["keys_changed"] = 0
            elif rule == "none":
                r["keys_changed"] = sum(1 for x in rows if x["actual"])
            else:
                r["keys_changed"] = sum(1 for x in rows if set(x[rule]) != set(x["actual"]))
            fns += len(rows)
            for k, v in r.items():
                tot[k] = max(tot.get(k, 0), v) if k == "def_rlimit_max_over_seeds" else tot.get(k, 0) + v
        line = {"corpus": label, "rule": rule, "programs": len(parsed), "skipped": skipped, "fns": fns, **tot}
        print(json.dumps(line, ensure_ascii=False), flush=True)
        if write:
            with open(write, "a", encoding="utf-8") as w:
                w.write(json.dumps(line, ensure_ascii=False) + "\n")


def main():
    bmb = sys.argv[1]
    write = sys.argv[sys.argv.index("--write") + 1] if "--write" in sys.argv else None
    with tempfile.TemporaryDirectory() as tmp:
        for label, paths in corpora(tmp):
            if label == "tests/verify/pass":
                run_corpus(bmb, label + " (each file alone)", [[p] for p in paths], tmp, write)
            else:
                run_corpus(bmb, label, [paths], tmp, write)


if __name__ == "__main__":
    main()
