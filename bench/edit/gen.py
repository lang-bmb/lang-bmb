#!/usr/bin/env python3
"""H-EDIT / H-GRAIN task sets (backlog T1-06, 2D-24): m1b, BVB and BRB with a one-call write.

  python3 bench/edit/gen.py            writes bench/m1b-write, bench/bvb-write and bench/brb-write
  python3 bench/edit/gen.py --check    fails if a committed set differs from a fresh build

Same seed, tasks and BSB mutants as the source set; only the write surface
differs. The server runs with `--write-tool` and the model is shown `write`
(unit, definitions, removals, commit and proof in one call) instead of the
seven transaction tools, which stay callable for the scripted reference and
weak agents (`set.json`, read by bench/m1/harness.py). The prompt's tool list
and "How to work" say so; the language rules are unchanged.

BRB's frozen inputs live with its harness (`bench/brb`), so `bench/brb-write`
holds only what differs for the BMB arm: `prompt-bmb.md` and `set.json`, and a
MANIFEST over them and BRB's own MANIFEST. `bench/pilot/brb_pilot.py
--variant bench/brb-write` uses it; the Rust arm has no variant.
"""
import hashlib
import json
import os
import shutil
import sys
import tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(os.path.dirname(HERE))
BENCH = os.path.join(ROOT, "bench")
sys.path.insert(0, os.path.join(BENCH, "m1"))
import bsb  # noqa: E402
import harness  # noqa: E402

TX = ["tx_begin", "tx_unit", "tx_put", "tx_remove", "tx_remove_unit", "tx_commit", "tx_abort"]
SET_JSON = {"mcp_flags": ["--write-tool"], "hide_tools": TX}

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
OLD_WORK_BRB = """Write through
a transaction, then `verify`. If verification fails, fix the code or the
contract and commit again. Finish when you are confident the function is
correct and `verify` passes."""
NEW_WORK_BRB = """Write with
`write`. If the proof fails, fix the code or the contract and write again.
Finish when you are confident the function is correct and the proof passes."""


def rewrite(src_path, pairs):
    text = open(src_path).read()
    for old, new in pairs:
        if old not in text:
            sys.exit(f"{os.path.relpath(src_path, ROOT)} changed: update bench/edit/gen.py")
        text = text.replace(old, new)
    return text


def write_set_json(out):
    with open(os.path.join(out, "set.json"), "w") as f:
        json.dump(SET_JSON, f, indent=1)
        f.write("\n")


def readme(out, name, source):
    with open(os.path.join(out, "README.md"), "w") as f:
        f.write(f"# {name} (H-EDIT·H-GRAIN, 백로그 T1-06·2D-24)\n\n`bench/edit/gen.py`가 `bench/{source}`에서 만든 판이다(동결). "
                f"직접 고치지 않고 생성기로 다시 만든다. {source}와 쓰기 표면만 다르다(생성기 머리 주석).\n")


def build_set(root, source):
    """m1b or BVB: a whole task set for bench/m1/harness.py."""
    src = os.path.join(BENCH, source)
    out = os.path.join(root, f"{source}-write")
    shutil.rmtree(out, ignore_errors=True)
    shutil.copytree(os.path.join(src, "seed"), os.path.join(out, "seed"))
    shutil.copytree(os.path.join(src, "tasks"), os.path.join(out, "tasks"))
    if os.path.isdir(bsb.mutants_dir(src)):
        shutil.copytree(bsb.mutants_dir(src), bsb.mutants_dir(out))
    with open(os.path.join(out, "prompt.md"), "w") as f:
        f.write(rewrite(os.path.join(src, "prompt.md"), ((OLD_TOOL, NEW_TOOL), (OLD_WORK, NEW_WORK))))
    write_set_json(out)
    harness.SET = out
    with open(os.path.join(out, "MANIFEST"), "w") as f:
        f.write(harness.manifest_text())
    readme(out, f"{source}-write", source)
    return out


def brb_manifest(out):
    lines = []
    for f in ("prompt-bmb.md", "set.json"):
        lines.append(f"{hashlib.sha256(open(os.path.join(out, f), 'rb').read()).hexdigest()}  {f}")
    brb = os.path.join(BENCH, "brb", "MANIFEST")
    lines.append(f"{hashlib.sha256(open(brb, 'rb').read()).hexdigest()}  ../brb/MANIFEST")
    return "\n".join(lines) + "\n"


def build_brb(root):
    out = os.path.join(root, "brb-write")
    shutil.rmtree(out, ignore_errors=True)
    os.makedirs(out)
    with open(os.path.join(out, "prompt-bmb.md"), "w") as f:
        f.write(rewrite(os.path.join(BENCH, "brb", "prompt-bmb.md"), ((OLD_TOOL, NEW_TOOL), (OLD_WORK_BRB, NEW_WORK_BRB))))
    write_set_json(out)
    with open(os.path.join(out, "MANIFEST"), "w") as f:
        f.write(brb_manifest(out))
    readme(out, "brb-write", "brb")
    return out


def build(root):
    return [build_set(root, "m1b"), build_set(root, "bvb"), build_brb(root)]


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
            for out in build(tmp):
                name = os.path.basename(out)
                if tree(out) != tree(os.path.join(BENCH, name)):
                    sys.exit(f"bench/{name} differs from bench/edit/gen.py output: regenerate it")
                print(f"bench/{name} ok")
        finally:
            shutil.rmtree(tmp, ignore_errors=True)
    else:
        for out in build(BENCH):
            print(out)


if __name__ == "__main__":
    main()
