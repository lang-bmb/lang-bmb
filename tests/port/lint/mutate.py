#!/usr/bin/env python3
"""`bmb mutate` over one program: every body mutation of its functions.

    mutate.py BMB FILE

The executable functions are those the compiler under test lists itself
(`map L0`: the definitions of kind `fn`), in path order. They are mutated ten at a
time, then one alone, then the first two in reverse order; a path that is a
`spec fn` or no definition ends a run with its message. Prints each command's
output and exit code.
"""
import json
import subprocess
import sys

bmb, file = sys.argv[1], sys.argv[2]


def run(args):
    p = subprocess.run([bmb] + args, capture_output=True)
    sys.stdout.write(p.stdout.decode("utf-8", "replace"))
    sys.stdout.write(p.stderr.decode("utf-8", "replace"))
    print("exit", p.returncode)


rows = subprocess.run([bmb, "map", "L0", file], capture_output=True).stdout.decode("utf-8", "replace")
fns, others = [], []
for line in rows.splitlines():
    try:
        row = json.loads(line)
    except ValueError:
        continue
    if row.get("kind") != "def":
        continue
    if row.get("def_kind") == "fn":
        fns.append(row["path"])
    else:
        others.append(row["path"])
print("fns", len(fns), "others", len(others))


def defs(paths):
    out = []
    for p in paths:
        out += ["--def", p]
    return out


for i in range(0, len(fns), 10):
    run(["mutate"] + defs(fns[i:i + 10]) + [file])
if fns:
    run(["mutate", "--def", fns[-1], file])
if len(fns) > 1:
    run(["mutate", "--def", fns[1], "--def", fns[0], file])
if fns and others:
    run(["mutate", "--def", fns[0], "--def", others[0], file])
run(["mutate", "--def", "no::such", file])
