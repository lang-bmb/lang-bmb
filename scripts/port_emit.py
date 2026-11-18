#!/usr/bin/env python3
"""The port's `emit-llvm --all-checks` against its allow-list (backlog 4D-05).

The port's code generator grows construct by construct. On a program with a
construct it does not lower yet, stage1 writes nothing and exits 2 with the
reasons on stderr. `tests/port/emit/ported.txt` lists the tracked inputs it
does lower; the differential test compares those with stage0 byte for byte.

    port_emit.py run STAGE1 BMB ARGS..    one side of the differential test:
        stage1's `emit-llvm ARGS..`; on an input stage1 does not lower yet,
        stage0's (BMB) instead, so the two sides agree there. Exit 3 when
        the input and the allow-list disagree: a listed input that stage1
        no longer lowers, or an unlisted one that it does.
    port_emit.py write STAGE1 [BMB]       regenerate the allow-list from the
        tracked inputs of the `emit` case (stage1 lowers them and stage0
        writes IR for them)

Standard library only.
"""
import json
import os
import subprocess
import sys
import tempfile

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
LIST = os.path.join(ROOT, "tests", "port", "emit", "ported.txt")
CASES = os.path.join(ROOT, "tests", "port", "cases.json")
MARK = b"stage1: emit-llvm: not ported yet"


def listed():
    if not os.path.exists(LIST):
        return set()
    with open(LIST, encoding="utf-8") as f:
        return {line.strip() for line in f if line.strip()}


def sources(args):
    out, skip = [], False
    for a in args:
        if skip:
            skip = False
        elif a == "-o":
            skip = True
        elif not a.startswith("--"):
            out.append(os.path.relpath(os.path.abspath(a), ROOT))
    return out


def run(stage1, bmb, args):
    r = subprocess.run([stage1, "emit-llvm", *args], capture_output=True)
    not_ported = r.returncode == 2 and r.stderr.startswith(MARK)
    wrote = "-o" in args and os.path.exists(args[args.index("-o") + 1])
    names = sources(args)
    allow = listed()
    if len(names) == 1:
        name = names[0]
        if name in allow and not wrote:
            sys.stderr.write(f"port_emit: {name} is in ported.txt and stage1 did not lower it\n")
            sys.stderr.buffer.write(r.stderr)
            return 3
        if name not in allow and wrote:
            sys.stderr.write(f"port_emit: stage1 lowers {name}: run scripts/port_emit.py write\n")
            return 3
    if not_ported:
        r = subprocess.run([bmb, "emit-llvm", *args], capture_output=True)
    sys.stdout.buffer.write(r.stdout)
    sys.stderr.buffer.write(r.stderr)
    return r.returncode


def write(stage1, bmb):
    sys.path.insert(0, os.path.join(ROOT, "scripts"))
    import difftest

    with open(CASES, encoding="utf-8") as f:
        case = next(c for c in json.load(f)["cases"] if c["name"] == "emit")
    names = difftest.tracked(case["inputs"]["files"])
    ok = []
    with tempfile.TemporaryDirectory() as d:
        for n in names:
            out = os.path.join(d, "o.ll")
            if os.path.exists(out):
                os.remove(out)
            src = os.path.join(ROOT, n)
            r = subprocess.run([stage1, "emit-llvm", "--all-checks", src, "-o", out], capture_output=True)
            if r.returncode == 0 and os.path.exists(out):
                ok.append(os.path.relpath(src, ROOT))
    with open(LIST, "w", encoding="utf-8") as f:
        f.write("".join(n + "\n" for n in sorted(ok)))
    print(f"port_emit: {len(ok)} of {len(names)} inputs lowered")
    return 0


def main():
    a = sys.argv[1:]
    if len(a) >= 3 and a[0] == "run":
        return run(a[1], a[2], a[3:])
    if len(a) >= 2 and a[0] == "write":
        return write(a[1], a[2] if len(a) > 2 else os.path.join(ROOT, "target", "debug", "bmb"))
    print(__doc__)
    return 2


if __name__ == "__main__":
    sys.exit(main())
