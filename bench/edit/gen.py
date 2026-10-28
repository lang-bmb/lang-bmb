#!/usr/bin/env python3
"""H-EDIT / H-GRAIN task set (backlog T1-06, 2D-24): m1b with a one-call write.

  python3 bench/edit/gen.py            writes bench/m1b-write
  python3 bench/edit/gen.py --check    fails if the committed set differs from a fresh build

Same seed, tasks and BSB mutants as m1b; only the write surface differs. The
server runs with `--write-tool` and the model is shown `write` (unit,
definitions, removals, commit and proof in one call) instead of the seven
transaction tools, which stay callable for the scripted reference and weak
agents (`set.json`, read by bench/m1/harness.py). The prompt's tool list and
"How to work" say so; the language rules are unchanged.
"""
import json
import os
import shutil
import sys
import tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(os.path.dirname(HERE))
SRC = os.path.join(ROOT, "bench", "m1b")
sys.path.insert(0, os.path.join(ROOT, "bench", "m1"))
import bsb  # noqa: E402
import harness  # noqa: E402

TX = ["tx_begin", "tx_unit", "tx_put", "tx_remove", "tx_remove_unit", "tx_commit", "tx_abort"]

OLD_TOOL = """- `tx_begin`, `tx_unit`, `tx_put`, `tx_remove`, `tx_remove_unit`, `tx_commit`,
  `tx_abort` — the only way to write. `tx_put` takes one or more complete
  definitions (with attributes) for one unit. A commit is rejected as a whole
  on any syntax or type error, and the diagnostics say why.
- `verify` — prove the whole workspace. Failures come with counterexamples.
"""
NEW_TOOL = """- `write` — the only way to write. One call takes a unit, one or more complete
  definitions (with attributes) to add or replace in it, an optional one-line
  `summary` (creates the unit or replaces its summary) and optional `remove`
  (full paths). It commits and then proves the whole workspace: a commit is
  rejected as a whole on any syntax or type error, and the diagnostics say why;
  after a commit you get the proof result, with counterexamples for failures.
- `verify` — prove the whole workspace again without writing.
"""
OLD_WORK = """Read only what you need through `query`. Write through one transaction, then
`verify`. If verification fails, fix the code or the contract and commit again."""
NEW_WORK = """Read only what you need through `query`. Write with `write`. If the proof
fails, fix the code or the contract and write again."""


def build(root):
    out = os.path.join(root, "m1b-write")
    shutil.rmtree(out, ignore_errors=True)
    shutil.copytree(os.path.join(SRC, "seed"), os.path.join(out, "seed"))
    shutil.copytree(os.path.join(SRC, "tasks"), os.path.join(out, "tasks"))
    shutil.copytree(bsb.mutants_dir(SRC), bsb.mutants_dir(out))
    prompt = open(os.path.join(SRC, "prompt.md")).read()
    for old, new in ((OLD_TOOL, NEW_TOOL), (OLD_WORK, NEW_WORK)):
        if old not in prompt:
            sys.exit("bench/m1b/prompt.md changed: update bench/edit/gen.py")
        prompt = prompt.replace(old, new)
    with open(os.path.join(out, "prompt.md"), "w") as f:
        f.write(prompt)
    with open(os.path.join(out, "set.json"), "w") as f:
        json.dump({"mcp_flags": ["--write-tool"], "hide_tools": TX}, f, indent=1)
        f.write("\n")
    harness.SET = out
    with open(os.path.join(out, "MANIFEST"), "w") as f:
        f.write(harness.manifest_text())
    with open(os.path.join(out, "README.md"), "w") as f:
        f.write("# m1b-write (H-EDIT·H-GRAIN, 백로그 T1-06·2D-24)\n\n`bench/edit/gen.py`가 `bench/m1b`에서 만든 판이다(동결). "
                "직접 고치지 않고 생성기로 다시 만든다. m1b와 쓰기 표면만 다르다(생성기 머리 주석).\n")
    return out


def tree(d):
    out = {}
    for base, _, fs in os.walk(d):
        for f in fs:
            p = os.path.join(base, f)
            out[os.path.relpath(p, d)] = open(p, "rb").read()
    return out


def main():
    if "--check" in sys.argv:
        tmp = tempfile.mkdtemp(prefix="edit-")
        try:
            if tree(build(tmp)) != tree(os.path.join(ROOT, "bench", "m1b-write")):
                sys.exit("bench/m1b-write differs from bench/edit/gen.py output: regenerate it")
            print("bench/m1b-write ok")
        finally:
            shutil.rmtree(tmp, ignore_errors=True)
    else:
        print(build(os.path.join(ROOT, "bench")))


if __name__ == "__main__":
    main()
