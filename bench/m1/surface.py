"""Scripted agents' writes through a set's edit surface (backlog T1-06, ADR-0073).

A task's `reference` and `weak_reference` are transaction ops. A set whose
`set.json` names a `script` surface plays them through that surface instead
of the hidden transaction tools, so the gate checks that every reference and
weak solution can be written through it:

  diff   one `edit_file` call: the unified diff from the workspace's files to
         the files the ops commit (`bmb apply` on a copy)
  ast    one `edit_ast` call: the JSON Patch from the workspace's syntax tree
         to the tree after the ops (`bmb ast` of both)

Ops that do not commit write nothing either way; the script then only calls
`verify`, which leaves the workspace as a rejected write does.

Standard library only.
"""
import difflib
import json
import os
import shutil
import subprocess
import tempfile


def _read(path):
    with open(path) as f:
        return f.read()


def _files(ws):
    src = os.path.join(ws, "src")
    if not os.path.isdir(src):
        return {}
    return {f"src/{f}": _read(os.path.join(src, f)) for f in sorted(os.listdir(src)) if f.endswith(".bmb")}


def _after(ops, ws, bmb):
    """The workspace directory after `ops` (a copy), or None if they do not commit."""
    tmp = tempfile.mkdtemp(prefix="bmb-surface-")
    copy = os.path.join(tmp, "ws")
    os.makedirs(copy)
    if os.path.isdir(os.path.join(ws, "src")):
        shutil.copytree(os.path.join(ws, "src"), os.path.join(copy, "src"))
    ops_file = os.path.join(tmp, "ops.json")
    with open(ops_file, "w") as f:
        json.dump(ops, f)
    r = subprocess.run([bmb, "apply", copy, ops_file], capture_output=True, text=True)
    if r.returncode != 0:
        shutil.rmtree(tmp, ignore_errors=True)
        return None, None
    return tmp, copy


def unified_diff(old, new):
    """A unified diff from files `old` to `new` ({path: text})."""
    out = []
    for path in sorted(set(old) | set(new)):
        a, b = old.get(path), new.get(path)
        if a == b:
            continue
        out += difflib.unified_diff((a or "").splitlines(), (b or "").splitlines(),
                                    f"a/{path}" if a is not None else "/dev/null",
                                    f"b/{path}" if b is not None else "/dev/null", lineterm="")
    return "\n".join(out) + "\n" if out else ""


def _ptr(segs):
    return "".join("/" + str(s).replace("~", "~0").replace("/", "~1") for s in segs)


def json_patch(a, b, at=()):
    """RFC 6902 operations that turn `a` into `b`: members by key, arrays of
    the same length by index, anything else replaced whole."""
    if a == b:
        return []
    if isinstance(a, dict) and isinstance(b, dict):
        ops = [{"op": "remove", "path": _ptr(at + (k,))} for k in a if k not in b]
        for k, v in b.items():
            if k not in a:
                ops.append({"op": "add", "path": _ptr(at + (k,)), "value": v})
            else:
                ops += json_patch(a[k], v, at + (k,))
        return ops
    if isinstance(a, list) and isinstance(b, list) and len(a) == len(b):
        return [op for i, (x, y) in enumerate(zip(a, b)) for op in json_patch(x, y, at + (i,))]
    return [{"op": "replace", "path": _ptr(at), "value": b}]


def _tree(ws, bmb):
    if not _files(ws):
        return {}
    r = subprocess.run([bmb, "ast", ws], capture_output=True, text=True, check=True)
    return json.loads(r.stdout)


def calls(surface, ops, ws, bmb):
    """The tool calls that write `ops` on workspace `ws` through `surface`."""
    if not ops:
        return [("verify", {})]
    tmp, after = _after(ops, ws, bmb)
    if after is None:
        return [("verify", {})]
    try:
        if surface == "diff":
            diff = unified_diff(_files(ws), _files(after))
            return [("edit_file", {"diff": diff})] if diff else [("verify", {})]
        if surface == "ast":
            patch = json_patch(_tree(ws, bmb), _tree(after, bmb))
            return [("edit_ast", {"patch": patch})] if patch else [("verify", {})]
        raise ValueError(f"unknown surface {surface}")
    finally:
        shutil.rmtree(tmp, ignore_errors=True)
