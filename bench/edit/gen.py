#!/usr/bin/env python3
"""H-EDIT / H-GRAIN task sets (backlog T1-06, 2D-24, ADR-0073): m1b, BVB and BRB
through three write surfaces.

  python3 bench/edit/gen.py            writes the sets below
  python3 bench/edit/gen.py --check    fails if a committed set differs from a fresh build

Same seed, tasks and BSB mutants as the source set; only the write surface
differs. The transaction tools stay callable but the model is not shown them
(`hide_tools` in `set.json`, read by bench/m1/harness.py). The prompt's tool
list and "How to work" say what the model has instead; the language rules
are unchanged.

  *-write   definitions (structural address + text): the server runs with
            `--write-tool` and the model is shown `write` (unit,
            definitions, removals, commit and proof in one call). The
            scripted reference and weak agents use the transaction tools.
  *-diff    files (unified diff): `--file-tools`, the model is shown
            `read_file` and `edit_file` (a diff over `src/UNIT.bmb` files,
            committed and proved in one call).
  *-ast     nodes (JSON syntax tree): `--ast-tools`, the model is shown
            `read_ast` and `edit_ast` (a JSON Patch over the tree, committed
            and proved in one call); the example also appears as the tree.

The `-diff` and `-ast` sets' scripted agents write their solutions through
the set's own surface (`script` in `set.json`, bench/m1/surface.py), so the
gate shows every reference and weak solution can be written through it.

BRB's frozen inputs live with its harness (`bench/brb`), so `bench/brb-*`
holds only what differs for the BMB arm: `prompt-bmb.md` and `set.json`, and a
MANIFEST over them and BRB's own MANIFEST. `bench/pilot/brb_pilot.py
--variant bench/brb-write` uses it; the Rust arm has no variant.
"""
import hashlib
import json
import os
import re
import shutil
import subprocess
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

DIFF_TOOL = """- `read_file` — read the file of a top-level unit, `src/UNIT.bmb` (one file per
  unit, in normal form), to write diffs against.
- `edit_file` — the only way to write. One call takes a unified diff over one or
  more files (`--- a/src/UNIT.bmb`, `+++ b/src/UNIT.bmb`, then `@@ -LINE,COUNT
  +LINE,COUNT @@` hunks; `--- /dev/null` creates a file). It commits and then
  proves the whole workspace: a commit is rejected as a whole if a hunk does
  not apply or on any syntax or type error, and the diagnostics say why; after
  a commit you get the proof result, with counterexamples for failures.
- `verify` — prove the whole workspace again without writing.
"""
DIFF_WORK = """Read only what you need through `query` and `read_file`. Write with
`edit_file`. If the proof fails, fix the code or the contract and edit again."""
DIFF_WORK_BRB = """Write with
`edit_file`. If the proof fails, fix the code or the contract and edit again.
Finish when you are confident the function is correct and the proof passes."""
AST_TOOL = """- `read_ast` — read the syntax tree as JSON at a JSON Pointer: `/UNIT/fns/NAME`
  is a function, `/UNIT/fns/NAME/body/tail` the last expression of its body.
  Every node is an object with a `kind` and its parts by name (see the example).
- `edit_ast` — the only way to write. One call takes a JSON Patch (RFC 6902:
  `add`, `remove`, `replace`, `move`, `copy`, `test` at JSON Pointers) over the
  tree. It commits and then proves the whole workspace: a commit is rejected as
  a whole if an operation does not apply, the result is not a syntax tree, or
  on any syntax or type error, and the diagnostics say where (`at`, a pointer);
  after a commit you get the proof result, with counterexamples for failures.
- `verify` — prove the whole workspace again without writing.
"""
AST_WORK = """Read only what you need through `query` and `read_ast`. Write with
`edit_ast`. If the proof fails, fix the code or the contract and edit again."""
AST_WORK_BRB = """Write with
`edit_ast`. If the proof fails, fix the code or the contract and edit again.
Finish when you are confident the function is correct and the proof passes."""

# The write surfaces: (set.json, tool list text, "How to work" for m1b/BVB and for BRB).
SURFACES = {
    "write": (SET_JSON, NEW_TOOL, NEW_WORK, NEW_WORK_BRB),
    "diff": ({"mcp_flags": ["--file-tools"], "hide_tools": TX, "script": "diff"}, DIFF_TOOL, DIFF_WORK,
             DIFF_WORK_BRB),
    "ast": ({"mcp_flags": ["--ast-tools"], "hide_tools": TX, "script": "ast"}, AST_TOOL, AST_WORK, AST_WORK_BRB),
}


def rewrite(src_path, pairs):
    text = open(src_path).read()
    for old, new in pairs:
        if old not in text:
            sys.exit(f"{os.path.relpath(src_path, ROOT)} changed: update bench/edit/gen.py")
        text = text.replace(old, new)
    return text


def write_set_json(out, surface="write"):
    with open(os.path.join(out, "set.json"), "w") as f:
        json.dump(SURFACES[surface][0], f, indent=1)
        f.write("\n")


def readme(out, name, source, surface="write"):
    refs = "백로그 T1-06·2D-24" if surface == "write" else "백로그 T1-06, ADR-0073"
    with open(os.path.join(out, "README.md"), "w") as f:
        f.write(f"# {name} (H-EDIT·H-GRAIN, {refs})\n\n`bench/edit/gen.py`가 `bench/{source}`에서 만든 판이다(동결). "
                f"직접 고치지 않고 생성기로 다시 만든다. {source}와 쓰기 표면만 다르다(생성기 머리 주석).\n")


def example_tree(text):
    """The prompt's example unit as `read_ast` shows it (`bmb ast`)."""
    m = re.search(r"## Example\n```\n(.*?)```\n", text, re.S)
    if not m:
        sys.exit("the prompt has no example: update bench/edit/gen.py")
    tmp = tempfile.mkdtemp(prefix="edit-example-")
    try:
        path = os.path.join(tmp, "example.bmb")
        with open(path, "w") as f:
            f.write(m.group(1))
        r = subprocess.run([harness.bmb_bin(), "ast", path], capture_output=True, text=True)
        if r.returncode != 0:
            sys.exit(f"bmb ast on the prompt's example failed: {r.stdout}{r.stderr}")
    finally:
        shutil.rmtree(tmp, ignore_errors=True)
    (unit, tree), = json.loads(r.stdout).items()
    shown = json.dumps(tree, ensure_ascii=False, separators=(",", ":"))
    return m.group(0), f"{m.group(0)}\nThe same unit as `read_ast` shows it at `/{unit}`:\n```\n{shown}\n```\n"


def prompt(src_path, surface, work_old, brb=False):
    _, tool, work, work_brb = SURFACES[surface]
    pairs = [(OLD_TOOL, tool), (work_old, work_brb if brb else work)]
    if surface == "ast":
        pairs.append(example_tree(open(src_path).read()))
    return rewrite(src_path, pairs)


def build_set(root, source, surface="write"):
    """m1b or BVB: a whole task set for bench/m1/harness.py."""
    src = os.path.join(BENCH, source)
    out = os.path.join(root, f"{source}-{surface}")
    shutil.rmtree(out, ignore_errors=True)
    shutil.copytree(os.path.join(src, "seed"), os.path.join(out, "seed"))
    shutil.copytree(os.path.join(src, "tasks"), os.path.join(out, "tasks"))
    if os.path.isdir(bsb.mutants_dir(src)):
        shutil.copytree(bsb.mutants_dir(src), bsb.mutants_dir(out))
    with open(os.path.join(out, "prompt.md"), "w") as f:
        f.write(prompt(os.path.join(src, "prompt.md"), surface, OLD_WORK))
    write_set_json(out, surface)
    harness.SET = out
    with open(os.path.join(out, "MANIFEST"), "w") as f:
        f.write(harness.manifest_text())
    readme(out, f"{source}-{surface}", source, surface)
    return out


def brb_manifest(out):
    lines = []
    for f in ("prompt-bmb.md", "set.json"):
        lines.append(f"{hashlib.sha256(open(os.path.join(out, f), 'rb').read()).hexdigest()}  {f}")
    brb = os.path.join(BENCH, "brb", "MANIFEST")
    lines.append(f"{hashlib.sha256(open(brb, 'rb').read()).hexdigest()}  ../brb/MANIFEST")
    return "\n".join(lines) + "\n"


def build_brb(root, surface="write"):
    out = os.path.join(root, f"brb-{surface}")
    shutil.rmtree(out, ignore_errors=True)
    os.makedirs(out)
    with open(os.path.join(out, "prompt-bmb.md"), "w") as f:
        f.write(prompt(os.path.join(BENCH, "brb", "prompt-bmb.md"), surface, OLD_WORK_BRB, brb=True))
    write_set_json(out, surface)
    with open(os.path.join(out, "MANIFEST"), "w") as f:
        f.write(brb_manifest(out))
    readme(out, f"brb-{surface}", "brb", surface)
    return out


def build(root):
    return [b for surface in SURFACES
            for b in (build_set(root, "m1b", surface), build_set(root, "bvb", surface), build_brb(root, surface))]


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
