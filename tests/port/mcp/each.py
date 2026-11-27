#!/usr/bin/env python3
"""One program through the tools of `bmb mcp` that only the server reaches.

    each.py BMB0 BMB ROOT FILE

The workspace `a` is the program imported by BMB0 (the Rust compiler on
both sides); a program it does not take is copied as `a/src/x.bmb`, a
workspace that does not check. The sessions, written as a script for
session.py (in this directory, whose head describes the lines):

  - `--ast-tools --file-tools --stats` on `a`: `verify`, whose diagnostics
    name the node of the tree; the whole tree and the body of the first
    function with `read_ast`; the first file with `read_file`; a diff that
    adds an empty line to it with `edit_file` (stored as it was, so the
    file is `reformatted`);
  - `--ast-tools --compact` on the empty workspace `b`: the whole tree of
    `a` added by one `edit_ast` (what the Rust compiler's test
    `every_program_round_trips_through_json` does), then whether `b/src` is
    `a/src`;
  - `--probe-tool --bounded-16k --write-tool --rename-ops` on `a`: the map
    in windows; for the first three functions with a loop, candidates for
    their first loop and for one that is not there; the first function
    renamed with the identity it has;
  - `--undecided-info` on `a`: `verify` with the #[uses] search.
"""
import json
import os
import shutil
import subprocess
import sys


def loops_in(node):
    if isinstance(node, dict):
        if node.get("kind") in ("while", "for", "loop"):
            return True
        return any(loops_in(v) for v in node.values())
    if isinstance(node, list):
        return any(loops_in(v) for v in node)
    return False


def fns_of(doc, prefix=""):
    out = []
    for name, unit in doc.items():
        path = prefix + name
        for f, body in unit.get("fns", {}).items():
            out.append((f"{path}::{f}", f"/{path.replace('::', '/units/')}/fns/{f}", body))
        out += fns_of(unit.get("units", {}), path + "::")
    return out


def main():
    bmb0, bmb, root, file = sys.argv[1:5]
    here = os.path.dirname(os.path.abspath(__file__))
    quiet = dict(stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    if subprocess.run([bmb0, "import", "a", file], **quiet).returncode != 0:
        os.makedirs("a/src", exist_ok=True)
        shutil.copy(file, "a/src/x.bmb")
    r = subprocess.run([bmb0, "ast", "a"], stdout=subprocess.PIPE, stderr=subprocess.DEVNULL)
    doc = json.loads(r.stdout) if r.returncode == 0 else None
    fns = fns_of(doc) if doc else []
    n = [0]
    lines = []

    def call(tool, args):
        n[0] += 1
        lines.append("> " + json.dumps({"jsonrpc": "2.0", "id": n[0], "method": "tools/call",
                                        "params": {"name": tool, "arguments": args}}, separators=(",", ":")))

    first = sorted(os.listdir("a/src"))[0]
    text = open(os.path.join("a/src", first), encoding="utf-8", errors="replace").read()
    count = text.count("\n")
    lines.append("start a --ast-tools --file-tools --stats")
    call("verify", {})
    call("read_ast", {})
    call("read_ast", {"path": (fns[0][1] + "/body") if fns else "/nope"})
    call("read_file", {"path": "src/" + first})
    call("edit_file", {"diff": f"--- a/src/{first}\n+++ b/src/{first}\n@@ -{count + 1},0 +{count + 1} @@\n+\n"})
    lines.append("stop")
    os.makedirs("b", exist_ok=True)
    lines.append("start b --ast-tools --compact")
    call("edit_ast", {"patch": [{"op": "add", "path": "", "value": doc if doc is not None else {}}]})
    lines.append("stop")
    lines.append("sh diff -r a/src b/src > /dev/null 2>&1; echo \"sources differ: $?\"")
    lines.append("start a --probe-tool --bounded-16k --write-tool --rename-ops")
    call("map", {})
    call("map", {"offset": 20, "limit": 10})
    for path, _, body in [f for f in fns if loops_in(f[2])][:3]:
        call("probe", {"def": path, "candidates": ["true", "false", "0 < 1"]})
        call("probe", {"def": path, "loop": 9, "candidates": ["true"]})
    lines.append("ids a")
    if fns:
        call("write", {"expect": [{"def": fns[0][0], "id": "@id{%s}" % fns[0][0]}],
                       "rename": [{"def": fns[0][0], "to": "renamed_by_the_test"}]})
    lines.append("stop")
    lines.append("start a --undecided-info")
    call("verify", {})
    lines.append("stop")
    lines.append("tree")
    with open("each.txt", "w", encoding="utf-8") as f:
        f.write("\n".join(lines) + "\n")
    code = subprocess.run([sys.executable, os.path.join(here, "session.py"), bmb0, bmb, root, "each.txt"]).returncode
    os.remove("each.txt")
    return code


if __name__ == "__main__":
    sys.exit(main())
