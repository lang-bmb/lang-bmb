#!/usr/bin/env python3
"""Transactions over one corpus program, for each FILE: the program is put
into an empty workspace `ws` item by item (a `unit` op per unit and a `put`
op per top-level item of its normal form, as `bmb fmt` prints it), which
must leave the normal form as the workspace's files. Then, each as a
transaction of its own on the workspace the last one left: `expect` of
every identity `bmb hash ws` prints (through standard input), `expect` of
an identity that is another's, `rename` of the first, the middle and the
last path, `rename` of the first unit, a new unit and a `move` of the
first two paths of a unit into it, `remove` of the last path and of the
first trait impl, and `remove_unit` of every unit. After each transaction:
what it printed, its exit code and the digest of every file of the
workspace. With `--put` only the first transaction and the `expect`s run.

    each.py [--put] BMB FILE..
"""
import hashlib
import json
import os
import subprocess
import sys


def run(bmb, args, stdin=None):
    p = subprocess.run([bmb] + args, input=stdin, capture_output=True, text=True)
    return p.returncode, p.stdout, p.stderr


def state(root):
    """The files under `root` with the digest of each, as lines."""
    rows = []
    for d, _, names in sorted(os.walk(root)):
        for n in sorted(names):
            path = os.path.join(d, n)
            with open(path, "rb") as f:
                rows.append(f"  {path} {hashlib.sha256(f.read()).hexdigest()}")
    return rows


def units_of(formatted):
    """`bmb fmt`'s output as (name, text) per unit: a unit ends at its
    unindented `}`."""
    out, name, text = [], "", ""
    for line in formatted.splitlines():
        if not text and not line:
            continue
        if line.startswith("mod "):
            name = line[4:].removesuffix(" {")
        text += line + "\n"
        if line == "}":
            out.append((name, text))
            name, text = "", ""
    return out


def ops_of(units):
    """A `unit` op per unit and a `put` op per top-level item (normal form:
    items at four spaces, one blank line between them)."""
    ops = []
    for name, unit in units:
        first = unit.split("\n", 1)[0]
        summary = json.loads(first[len("#[summary("):-len(")]")])
        ops.append({"op": "unit", "path": name, "summary": summary})
        body = unit[unit.index("{\n") + 2:len(unit.rstrip()) - 1]
        item = ""
        for line in body.splitlines():
            top = line.startswith("    ") and not line.startswith("     ")
            starts = top and (line.startswith("    #[") or not item.rstrip().endswith("]"))
            if starts and item.strip() and item.endswith("\n\n"):
                ops.append({"op": "put", "unit": name, "code": item.strip()})
                item = ""
            item += line + "\n"
        if item.strip():
            ops.append({"op": "put", "unit": name, "code": item.strip()})
    return ops


def apply(bmb, what, ops, through_stdin=False):
    text = json.dumps(ops, ensure_ascii=False)
    if through_stdin:
        code, out, err = run(bmb, ["apply", "ws", "-"], text)
    else:
        with open("ops.json", "w", encoding="utf-8") as f:
            f.write(text)
        code, out, err = run(bmb, ["apply", "ws", "ops.json"])
    print(f"== {what}")
    sys.stdout.write(out)
    sys.stdout.write(err)
    print(f"exit {code}")
    print("\n".join(state("ws")))
    return code, out


def hashes(bmb):
    """path -> id of `bmb hash ws`, in its order."""
    code, out, err = run(bmb, ["hash", "ws"])
    if code != 0:
        print(f"hash: exit {code}")
        sys.stdout.write(out + err)
        return {}
    return {r["def"]: r["id"] for r in map(json.loads, out.splitlines())}


def one(bmb, path, only_put):
    print(f"# {os.path.basename(path)}")
    code, out, err = run(bmb, ["fmt", path])
    if code != 0:
        print(f"fmt: exit {code}")
        return
    units = units_of(out)
    apply(bmb, "put", ops_of(units))
    got = [open(os.path.join("ws/src", n), encoding="utf-8").read() for n in sorted(os.listdir("ws/src"))] if os.path.isdir("ws/src") else []
    print("normal form" if got == [t for _, t in sorted(units)] else "NOT the normal form")
    ids = hashes(bmb)
    paths = list(ids)
    apply(bmb, "expect all", [{"op": "expect", "def": p, "id": ids[p]} for p in paths], True)
    if paths:
        apply(bmb, "expect another", [{"op": "expect", "def": paths[0], "id": ids[paths[-1]] + "0"}], True)
    if only_put:
        return
    for n, k in enumerate(sorted({0, len(paths) // 2, len(paths) - 1} if paths else [])):
        apply(bmb, f"rename {paths[k]}", [{"op": "rename", "def": paths[k], "to": f"renamed_{n}"}])
    if units:
        apply(bmb, "rename unit", [{"op": "rename", "def": units[0][0], "to": "unit_renamed"}])
    paths = [p for p in hashes(bmb) if p.count("::") == 1 and not p.startswith("<") and p.split("::")[1][:1].islower()]
    for p in paths[:2]:
        apply(bmb, f"move {p}", [{"op": "unit", "path": "landing", "summary": "moved"}, {"op": "move", "def": p, "to": "landing"}])
    paths = list(hashes(bmb))
    if paths:
        apply(bmb, f"remove {paths[-1]}", [{"op": "remove", "def": paths[-1]}])
    impls = sorted({p[:p.index(">::") + 1] for p in paths if p.startswith("<") and ">::" in p})
    if impls:
        apply(bmb, f"remove {impls[0]}", [{"op": "remove", "def": impls[0] + "::f"}, {"op": "remove", "def": impls[0]}])
    names = sorted(n[:-4] for n in os.listdir("ws/src")) if os.path.isdir("ws/src") else []
    apply(bmb, "remove units", [{"op": "remove_unit", "path": n} for n in names])


def main():
    args = sys.argv[1:]
    only_put = args[:1] == ["--put"]
    if only_put:
        args = args[1:]
    for path in args[1:]:
        one(args[0], path, only_put)


main()
