#!/usr/bin/env python3
"""`bmb weaken` between a program and its body mutants.

    mutants.py BMB ROOT

As the Rust compiler's own test does (`compiler/tests/weaken.rs`): a program
of the corpus is imported as workspace `a`, `bmb mutate --def DEF` gives the
mutants of one of its functions, and for each of the first three the
workspace `b` is `a` with the mutant's unit in place of the original. A
body mutant changes no contract, so every definition compares as `same`,
also the callers whose identity changed with the callee; the comparison
goes through the solver for them. Prints each comparison's output and exit
code; the workspaces are left as made (the last mutant's `b`).
"""
import json
import os
import shutil
import subprocess
import sys

bmb, root = sys.argv[1], sys.argv[2]
CASES = [
    ("tests/verify/pass/structs.bmb", "geo::twice"),
    ("tests/run/trait_contracts.bmb", "sc::best"),
    ("tests/verify/pass/invariants.bmb", "inv::make"),
    ("tests/verify/pass/invariants.bmb", "inv::two"),
]


def run(args):
    p = subprocess.run([bmb] + args, capture_output=True)
    sys.stdout.write(p.stdout.decode("utf-8", "replace"))
    sys.stdout.write(p.stderr.decode("utf-8", "replace"))
    print("exit", p.returncode)
    sys.stdout.flush()
    return p.stdout.decode("utf-8", "replace")


for k, (file, name) in enumerate(CASES):
    a, b = "a%d" % k, "b%d" % k
    run(["import", a, root + file])
    out = subprocess.run([bmb, "mutate", "--def", name, a], capture_output=True).stdout.decode("utf-8", "replace")
    rows = [json.loads(line) for line in out.splitlines() if '"text":' in line]
    print(file, name, len(rows), "mutants")
    for row in rows[:3]:
        shutil.rmtree(b, ignore_errors=True)
        shutil.copytree(a, b)
        with open(os.path.join(b, "src", row["unit"] + ".bmb"), "w", encoding="utf-8") as f:
            f.write(row["text"])
        run(["weaken", a, b])
        run(["weaken", b, a])
