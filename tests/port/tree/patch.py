#!/usr/bin/env python3
"""patch.py BMB EDIT FILE: JSON Patches and JSON Pointers over a real tree.
The JSON tree of FILE as BMB writes it (`bmb ast`) is the document. The
pointers are those of every N-th node of it (N so that about NODES are
taken) and the forms in `pointers.txt`; EDIT resolves each (`pointer`). The
patches are, at each of those nodes, every operation in the forms that
apply and the forms that are refused (a member or index that is not there, a
value moved into itself, a test that differs), then the patches of
`patches.jsonl` (the forms of an operation the reader refuses, the whole
document as a target, several operations of which one fails); EDIT applies
each to the document (`patch`) and reads the result back as a program.

The nodes are walked in document order, so the lines are the same on every
run and for both sides.
"""
import json
import os
import subprocess
import sys

NODES = 24
HERE = os.path.dirname(os.path.abspath(__file__))


def escape(seg):
    return seg.replace("~", "~0").replace("/", "~1")


def walk(v, path, out):
    out.append((path, v))
    if isinstance(v, dict):
        for k, x in v.items():
            walk(x, path + [k], out)
    elif isinstance(v, list):
        for i, x in enumerate(v):
            walk(x, path + [str(i)], out)


def ptr(path):
    return "".join("/" + escape(s) for s in path)


def main():
    bmb, edit, file = sys.argv[1:4]
    made = subprocess.run([bmb, "ast", file], capture_output=True, text=True)
    if made.returncode != 0:
        print("no tree")
        return
    with open("doc.json", "w", encoding="utf-8") as f:
        f.write(made.stdout)
    doc = json.loads(made.stdout)
    found = []
    walk(doc, [], found)
    step = max(1, len(found) // NODES)
    taken = found[::step]
    pointers = [ptr(p) for p, _ in taken]
    for p, v in taken:
        pointers.append(ptr(p + ["nosuch"]))
        if isinstance(v, list):
            pointers += [ptr(p + [s]) for s in ("-", str(len(v)), "00", "+0", "x", "18446744073709551616")]
    with open(os.path.join(HERE, "pointers.txt"), encoding="utf-8") as f:
        pointers += f.read().split("\n")[:-1]
    with open("pointers.lst", "w", encoding="utf-8") as f:
        f.write("\n".join(pointers) + "\n")
    patches = []
    def op(**kv):
        return kv
    for k, (p, v) in enumerate(taken):
        here = ptr(p)
        other_path, other = taken[(k * 7 + 3) % len(taken)]
        there = ptr(other_path)
        patches.append([op(op="test", path=here, value=v)])
        patches.append([op(op="test", path=here, value=other)])
        patches.append([op(op="replace", path=here, value=other)])
        patches.append([op(op="replace", path=here + "/nosuch", value=other)])
        patches.append([op(op="remove", path=here)])
        patches.append([op(op="remove", path=here + "/nosuch")])
        patches.append([op(op="add", path=here, value=other)])
        patches.append([op(op="add", path=here + "/added", value=v)])
        patches.append([op(op="add", path=here + "/nosuch/added", value=v)])
        patches.append([{"op": "copy", "from": there, "path": here}])
        patches.append([{"op": "copy", "from": there + "/nosuch", "path": here}])
        patches.append([{"op": "move", "from": there, "path": here}])
        patches.append([{"op": "move", "from": here, "path": here + "/inside"}])
        patches.append([{"op": "move", "from": here, "path": here}])
        if isinstance(v, list):
            for at in ("0", "-", str(len(v)), str(len(v) + 1), "01", "x", ""):
                patches.append([op(op="add", path=here + "/" + at, value=other)])
            for at in ("0", str(max(0, len(v) - 1)), str(len(v)), "-"):
                patches.append([op(op="remove", path=here + "/" + at)])
                patches.append([op(op="replace", path=here + "/" + at, value=v)])
                patches.append([{"op": "move", "from": here + "/" + at, "path": here + "/0"}])
        # All or nothing: the last operation fails.
        patches.append([op(op="replace", path=here, value=other), op(op="test", path=here, value=other), op(op="remove", path=here + "/nosuch")])
        patches.append([op(op="add", path=here, value=other), op(op="test", path=here, value=other), {"op": "copy", "from": here, "path": there}])
    lines = [json.dumps(p, ensure_ascii=False, separators=(",", ":")) for p in patches]
    with open(os.path.join(HERE, "patches.jsonl"), encoding="utf-8") as f:
        lines += f.read().split("\n")[:-1]
    with open("patches.lst", "w", encoding="utf-8") as f:
        f.write("\n".join(lines) + "\n")
    sys.stdout.flush()
    r = subprocess.run([edit, "pointer", "doc.json", "pointers.lst"])
    if r.returncode != 0:
        sys.exit(r.returncode)
    r = subprocess.run([edit, "patch", "doc.json", "patches.lst"])
    sys.exit(r.returncode)


if __name__ == "__main__":
    main()
