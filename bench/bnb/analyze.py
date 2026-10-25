#!/usr/bin/env python3
"""Rust-side analyzer of a twin crate (backlog 3A-05, BNB).

Reads src/m<U>.rs as text (the fixed layout twin.py writes, not general Rust)
and rebuilds the structure the BMB maps report, so a test can check that the
twin answers every BNB question the same way the BMB workspace does:

  defs       path -> {unit, name, pred, sig, params, requires, ensures, io, text}
  edges      {(from, to, kind, arg)}: kind `calls` (exec calls in bodies;
             `debug_assert!` lines and comments are contracts, not calls) and
             `writes` (a `&mut` parameter passed on, arg = its position)
  dup_groups sorted groups of >= 2 definitions whose text is equal once the
             name is removed and every call is resolved to its full path

Python 3 standard library only.
"""

import os
import re

HEADER = re.compile(r"^pub fn (\w+)\((.*)\)(?: -> (.+))? \{$")
CALL = re.compile(r"(?<![\w.:])(?:(m\d+)::)?([A-Za-z_]\w*)\(")


def split_args(s):
    out, depth, cur = [], 0, ""
    for ch in s:
        if ch in "([":
            depth += 1
        elif ch in ")]":
            depth -= 1
        if ch == "," and depth == 0:
            out.append(cur.strip())
            cur = ""
        else:
            cur += ch
    if cur.strip():
        out.append(cur.strip())
    return out


def call_args(line, start):
    """Argument list of the call whose `(` is at `start`."""
    depth = 0
    for i in range(start, len(line)):
        if line[i] == "(":
            depth += 1
        elif line[i] == ")":
            depth -= 1
            if depth == 0:
                return split_args(line[start + 1:i])
    return []


def parse_file(path, unit):
    lines = open(path, encoding="utf-8").read().split("\n")
    defs, doc, cur = {}, [], None
    for line in lines:
        if cur is not None:
            cur["body"].append(line)
            if line == "}":
                cur = None
            continue
        if line.startswith("///"):
            doc.append(line)
            continue
        m = HEADER.match(line)
        if m:
            name, params, ret = m.groups()
            ps = [tuple(x.split(": ", 1)) for x in split_args(params)]
            cur = {"unit": unit, "name": name, "header": line, "doc": doc, "params": ps, "ret": ret,
                   "body": []}
            defs[name] = cur
        doc = []
    return defs


def contract_preds(doc, key):
    out = []  # predicate names called in `/// key: ..` lines
    for d in doc:
        if d.startswith(f"/// {key}: "):
            out += [m.group(0)[:-1] for m in re.finditer(r"(?<![\w.:])(?:m\d+::)?\w+\(", d)]
    return out


def resolve(unit, names, m):
    v, name = m.group(1), m.group(2)
    if v:
        return f"{v}::{name}"
    return f"{unit}::{name}" if name in names else None


def analyze(root):
    src = os.path.join(root, "src")
    files = sorted(f for f in os.listdir(src) if re.fullmatch(r"m\d+\.rs", f))
    raw = {f[:-3]: parse_file(os.path.join(src, f), f[:-3]) for f in files}
    names = {u: set(d) for u, d in raw.items()}
    defs, edges = {}, set()
    for u, ds in raw.items():
        for name, d in ds.items():
            path = f"{u}::{name}"
            pred = "/// Predicate (specification only)." in d["doc"]

            def res(n):
                return f"{u}::{n}" if "::" not in n else n
            defs[path] = {
                "unit": u, "name": name, "pred": pred, "params": d["params"],
                "requires": sorted({res(x) for x in contract_preds(d["doc"], "Requires")}),
                "ensures": sorted({res(x) for x in contract_preds(d["doc"], "Ensures")}),
                "invariants": sorted({res(x) for line in d["body"] if line.strip().startswith("// invariant: ")
                                      for x in contract_preds([line.strip().replace("// invariant", "/// Invariant")], "Invariant")}),
                "io": any("println!" in x for x in d["body"]),
            }
            norm = [d["header"].replace(f"pub fn {name}(", "pub fn _(", 1)]
            for line in d["body"]:
                s = line.strip()
                norm.append(CALL.sub(lambda m: (resolve(u, names[u], m) or m.group(0)[:-1]) + "(", line))
                if s.startswith("debug_assert!") or s.startswith("//"):
                    continue
                for m in CALL.finditer(line):
                    to = resolve(u, names[u], m)
                    if to is None:
                        continue
                    edges.add((path, to, "calls", None))
                    mut_params = {p for p, t in d["params"] if t.startswith("&mut ")}
                    for i, a in enumerate(call_args(line, m.end() - 1)):
                        if a in mut_params:
                            edges.add((path, to, "writes", i))
            defs[path]["norm"] = "\n".join(norm)
    by = {}
    for p, d in defs.items():
        by.setdefault(d["norm"], []).append(p)
    groups = sorted(sorted(g) for g in by.values() if len(g) > 1)
    return {"defs": defs, "edges": edges, "dup_groups": groups}


# ---------- answers to the BNB task ops (same semantics as bnb.truth_bmb) ----------

def closure(start, step):
    seen, todo = set(), [start]
    while todo:
        x = todo.pop()
        for y in step(x):
            if y not in seen:
                seen.add(y)
                todo.append(y)
    seen.discard(start)
    return seen


def truth(task, g):
    E, D = g["edges"], g["defs"]
    op, d = task["op"], task.get("def")

    def callers(x):
        return {f for f, t, k, _ in E if t == x and k == "calls"}

    def callees(x):
        return {t for f, t, k, _ in E if f == x and k == "calls"}

    def io_all():
        return closure_set({p for p, v in D.items() if v["io"]}, callers)

    if op == "callers":
        return closure(d, callers) if task.get("closure") else callers(d)
    if op == "callees":
        return closure(d, callees) if task.get("closure") else callees(d)
    if op == "impact":
        # exec fn: itself and its direct callers (proofs see callees by contract)
        return {d} | callers(d)
    if op == "requires":
        return {p for p, v in D.items() if d in v["requires"]}
    if op == "ensures":
        return {p for p, v in D.items() if d in v["ensures"]}
    if op == "writers":
        return {f for f, t, k, _ in E if t == d and k == "writes"}
    if op == "writes_closure":
        return closure(d, lambda x: {t for f, t, k, _ in E if f == x and k == "writes"})
    if op == "io_all":
        return io_all()
    if op == "io_callers":
        return closure(d, callers) & io_all()
    if op == "duplicates":
        return next((set(gr) - {d} for gr in g["dup_groups"] if d in gr), set())
    raise ValueError(op)


def closure_set(starts, step):
    seen, todo = set(starts), list(starts)
    while todo:
        for y in step(todo.pop()):
            if y not in seen:
                seen.add(y)
                todo.append(y)
    return seen
