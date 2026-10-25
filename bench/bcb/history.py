#!/usr/bin/env python3
"""Commit histories for BCB, the cumulative verification cost bench
(backlog 2A-14, ADR-0042).

A history is a fixed-seed list of K commits over a bench/m3 workspace (gen.py)
and its Rust twin (bench/bnb/twin.py, plus one test per executable definition
from rtests.py). Commits edit the shared plan (gen.Unit
definitions and gen.choose choices), so both languages change in lockstep and
the same renderers write them. Every commit keeps the BMB workspace fully
proved; the gate checks it.

Edit kinds (one unit file per edit, in both languages):
  body      the body only; the interface (signature, requires, ensures) stays.
            leaf: a new constant (and template, when the leaf has no postcondition
            predicate) that still meets its contract; chain: new callees from the
            window and result form; scan, remap, reset, show: a new constant or
            callee.
  contract  a leaf's ensures gains or loses its `_ok` predicate (the predicate is
            added the first time).
  spec      a `_ok` predicate's bound changes (never below the leaf's body bound).
  add       a new leaf in a unit; later body edits may call it.
  remove    a leaf (with its `_ok` predicate) that nothing calls, never a unit's
            last two leaves.

The history file (JSONL) holds a header line and one line per commit with the
resulting field values (replay needs no random generator) and the digest of the
whole tree after the commit (SHA-256 over every unit file of both languages).
Replaying a history must reproduce every digest; that freezes the contents.

  history.py gen OUT.jsonl --defs N --commits K --profile P [--seed S] [gen.py params]
  history.py replay HISTORY.jsonl --bmb DIR --rust DIR [--upto K]
                                    (write the tree after commit K; default all)
  history.py stats HISTORY.jsonl    (edit kinds and units touched, as JSON)

Profiles give the share of each edit kind (PROFILES below). Toolchain events
(compiler or Z3 identity change on the BMB side, rustc upgrade on the Rust
side) are not edits: the header lists their commits, at K*j/5 for j = 1..4
(the one-event variant uses j = 2), and the replay harness applies them.
"""

import argparse
import copy
import hashlib
import json
import os
import random
import shutil
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
BENCH = os.path.dirname(HERE)
sys.path.insert(0, os.path.join(BENCH, "m3"))
sys.path.insert(0, os.path.join(BENCH, "bnb"))
import gen  # noqa: E402
import twin  # noqa: E402

sys.path.insert(0, HERE)
import rtests  # noqa: E402

FORMAT = 1
KINDS = ("body", "contract", "spec", "add", "remove")
PROFILES = {
    # Mostly body edits: the common case the contract firewall is about.
    "body-heavy": {"body": 0.70, "contract": 0.10, "spec": 0.05, "add": 0.10, "remove": 0.05},
    # Falsification condition (2) of ADR-0042: body edits only.
    "body-only": {"body": 1.0, "contract": 0.0, "spec": 0.0, "add": 0.0, "remove": 0.0},
    # Interface churn: contracts and predicates change often.
    "contract-heavy": {"body": 0.40, "contract": 0.30, "spec": 0.10, "add": 0.15, "remove": 0.05},
}
EDITS_PER_COMMIT = ((1, 0.60), (2, 0.25), (3, 0.15))
GEN_PARAMS = ("seed", "dpu", "depth", "fanout", "window", "contract_share", "mut_share", "scan_share", "io_share")


class Tree:
    """The plan of a workspace plus its rendered files, kept in step."""

    def __init__(self, args):
        self.args = args
        self.n_units = args.defs // args.dpu
        self.units = [gen.Unit(u, args) for u in range(self.n_units)]
        for u in range(self.n_units):
            gen.choose(self.units, u, args)
        for unit in self.units:
            unit.next = 1 + max(int(n.rsplit("_", 1)[1]) for n, k, _ in unit.defs if k not in ("spec", "post"))
        self.text = {}
        for u in range(self.n_units):
            self.render(u)

    def render(self, u):
        self.text[("bmb", u)] = gen.render_unit(self.units, u, self.args)
        self.text[("rust", u)] = self.rust(u)

    def rust(self, u, extra_fns=(), extra_tests=()):
        """The twin's unit file with its test module (rtests.py); mutant copies
        and their tests are added by kills.py."""
        text = twin.render_unit(self.units, u, self.args)
        text += "".join("\n" + f for f in extra_fns)
        return text + "\n" + rtests.tests_block(self.units[u], extra_tests)

    def digest(self):
        h = hashlib.sha256()
        for (lang, u) in sorted(self.text):
            h.update(f"{lang} {u} ".encode())
            h.update(hashlib.sha256(self.text[(lang, u)].encode()).digest())
        return h.hexdigest()

    def find(self, u, name):
        for i, (n, k, info) in enumerate(self.units[u].defs):
            if n == name:
                return i, k, info
        raise KeyError(f"m{u}::{name}")

    # Pools of possible callees, as gen.choose draws them (current plan).
    def pool(self, u, *kinds, layer=None):
        out = []
        for v in range(max(0, u - self.args.window), u + 1):
            out += [(v, n) for n, _, _ in self.units[v].by_kind(*kinds, layer=layer)]
        return out

    def arith_any(self, u):
        out = self.pool(u, "leaf")
        for L in range(1, self.args.depth):
            out += self.pool(u, "chain", layer=L)
        return out

    def callers(self, v, name):
        """Executable definitions that call `mv::name` (units v..v+window)."""
        out = []
        for u in range(v, min(self.n_units, v + self.args.window + 1)):
            for n, ch in self.units[u].choices.items():
                if [v, name] in [list(c) for c in ch["callees"]]:
                    out.append((u, n))
        return out

    # Edits. `apply` takes the recorded form; the generator below draws one.
    def apply(self, e):
        u = e["unit"]
        unit = self.units[u]
        if e["op"] in ("body", "contract", "spec"):
            i, k, info = self.find(u, e["def"])
            info.update(e.get("set", {}))
            if "choices" in e:
                unit.choices[e["def"]] = {"callees": [tuple(c) for c in e["choices"]["callees"]],
                                          **{x: y for x, y in e["choices"].items() if x != "callees"}}
            for d in e.get("new", []):  # a contract edit adding the `_ok` predicate
                unit.defs.append((d["name"], d["kind"], d["info"]))
        elif e["op"] == "add":
            unit.defs.append((e["def"], "leaf", dict(e["info"])))
            unit.next = max(unit.next, int(e["def"].rsplit("_", 1)[1]) + 1)
        elif e["op"] == "remove":
            names = set(e["defs"])
            unit.defs = [d for d in unit.defs if d[0] not in names]
            for n in names:
                unit.choices.pop(n, None)
        else:
            raise ValueError(e["op"])
        self.render(u)


def ok_def(tree, u, name):
    try:
        return tree.find(u, name + "_ok")
    except KeyError:
        return None


def draw_edit(tree, rng, kind):
    """One edit of `kind` (recorded form), or None when no target qualifies."""
    a = tree.args
    for _ in range(64):
        u = rng.randrange(tree.n_units)
        unit = tree.units[u]
        if kind == "add":
            tmpl = rng.choice(("shift", "halve", "abs", "clamp"))
            return {"op": "add", "unit": u, "def": f"{tmpl}_{unit.next}",
                    "info": {"tmpl": tmpl, "c": rng.randint(1, 1000), "post": False}}
        if kind == "remove":
            leaves = [n for n, k, _ in unit.defs if k == "leaf"]
            if len(leaves) <= 2:
                continue
            free = [n for n in leaves if not tree.callers(u, n)]
            if not free:
                continue
            n = rng.choice(sorted(free))
            return {"op": "remove", "unit": u, "defs": sorted([n] + ([n + "_ok"] if ok_def(tree, u, n) else []))}
        if kind == "contract":
            leaves = [(n, info) for n, k, info in unit.defs if k == "leaf"]
            n, info = rng.choice(leaves)
            ok = ok_def(tree, u, n)
            if ok:
                return {"op": "contract", "unit": u, "def": n, "set": {"post": not info["post"]}}
            # First postcondition of this leaf: the predicate is its exact bound.
            return {"op": "contract", "unit": u, "def": n, "set": {"post": True},
                    "new": [{"name": n + "_ok", "kind": "post", "info": {"of": n, "tmpl": info["tmpl"], "c": info["c"]}}]}
        if kind == "spec":
            oks = [(n, info) for n, k, info in unit.defs if k == "post" and info["tmpl"] in ("shift", "clamp")]
            if not oks:
                continue
            n, info = rng.choice(oks)
            _, _, leaf = tree.find(u, info["of"])
            c = rng.randint(leaf["c"], 1000)
            if c == info["c"]:
                continue
            return {"op": "spec", "unit": u, "def": n, "set": {"c": c}}
        if kind == "body":
            n, k, info = rng.choice(unit.defs)
            if k == "leaf":
                ok = ok_def(tree, u, n)
                if ok is None:
                    tmpl = rng.choice(("shift", "halve", "abs", "clamp"))
                    c = rng.randint(1, 1000)
                    if (tmpl, c) == (info["tmpl"], info["c"]) or (tmpl in ("halve", "abs") and tmpl == info["tmpl"]):
                        continue
                    return {"op": "body", "unit": u, "def": n, "set": {"tmpl": tmpl, "c": c}}
                if info["tmpl"] not in ("shift", "clamp"):
                    continue  # halve and abs bodies are fixed by their predicate
                bound = ok[2]["c"]
                c = rng.randint(1, bound)
                if c == info["c"]:
                    continue
                return {"op": "body", "unit": u, "def": n, "set": {"c": c}}
            if k == "chain":
                layer = info["layer"]
                cands = tree.pool(u, "leaf" if layer == 1 else "chain", layer=layer - 1)
                callees = [list(rng.choice(cands)) for _ in range(a.fanout)]
                ch = {"callees": callees, "avg": rng.random() < 0.5}
                old = unit.choices[n]
                if [list(c) for c in old["callees"]] == callees and old["avg"] == ch["avg"]:
                    continue
                return {"op": "body", "unit": u, "def": n, "choices": ch}
            if k == "scan_count":
                c = rng.randint(0, 1000)
                if c == info["c"]:
                    continue
                return {"op": "body", "unit": u, "def": n, "set": {"c": c}}
            if k in ("scan_peak", "remap", "show"):
                callee = list(rng.choice(tree.arith_any(u)))
                if [list(c) for c in unit.choices[n]["callees"]] == [callee]:
                    continue
                e = {"op": "body", "unit": u, "def": n, "choices": {"callees": [callee]}}
                if k == "scan_peak" and rng.random() < 0.5:
                    e["set"] = {"c": rng.randint(0, 1000)}
                return e
            if k == "reset":
                fills = [list(x) for x in tree.pool(u, "fill")]
                remaps = [list(x) for x in tree.pool(u, "remap")]
                callees = [rng.choice(fills), rng.choice(remaps)]
                if [list(c) for c in unit.choices[n]["callees"]] == callees:
                    continue
                return {"op": "body", "unit": u, "def": n, "choices": {"callees": callees}}
            continue  # spec, post, fill: nothing body-only to change
    return None


def pick(rng, table):
    x = rng.random()
    for value, share in table:
        if x < share:
            return value
        x -= share
    return table[-1][0]


def events(k):
    return {"four": [k * j // 5 for j in range(1, 5)], "one": [k * 2 // 5]}


def generate(args):
    tree = Tree(args)
    rng = random.Random(f"bcb:{args.seed}:{args.history_seed}:{args.profile}")
    shares = [(kind, PROFILES[args.profile][kind]) for kind in KINDS]
    header = {"format": FORMAT, "bench": "BCB", "commits": args.commits, "profile": args.profile,
              "history_seed": args.history_seed, "defs": args.defs,
              "gen": {p: getattr(args, p) for p in GEN_PARAMS},
              "events": events(args.commits), "digest0": tree.digest()}
    lines = [header]
    for k in range(1, args.commits + 1):
        edits = []
        for _ in range(pick(rng, EDITS_PER_COMMIT)):
            e = None
            while e is None:
                e = draw_edit(tree, rng, pick(rng, shares))
            tree.apply(copy.deepcopy(e))
            edits.append(e)
        lines.append({"k": k, "edits": edits, "digest": tree.digest()})
    return lines


def gen_args(header):
    g = header["gen"]
    argv = ["-", "--defs", str(header["defs"]), "--defs-per-unit", str(g["dpu"])]
    for p in GEN_PARAMS:
        if p != "dpu":
            argv += ["--" + p.replace("_", "-"), str(g[p])]
    return gen.parse(argv)


def load(path):
    lines = [json.loads(x) for x in open(path, encoding="utf-8")]
    if lines[0].get("format") != FORMAT:
        sys.exit(f"{path}: unknown history format")
    return lines[0], lines[1:]


def replay(header, commits, upto=None):
    """Yields (k, tree, changed units) after the initial tree (k = 0) and each commit;
    fails when a digest differs from the recorded one."""
    tree = Tree(gen_args(header))
    if tree.digest() != header["digest0"]:
        sys.exit("history: initial tree differs from the recorded digest (generator changed?)")
    yield 0, tree, set(range(tree.n_units))
    for c in commits:
        if upto is not None and c["k"] > upto:
            return
        changed = set()
        for e in c["edits"]:
            tree.apply(copy.deepcopy(e))
            changed.add(e["unit"])
        if tree.digest() != c["digest"]:
            sys.exit(f"history: commit {c['k']} differs from the recorded digest")
        yield c["k"], tree, changed


def write_tree(tree, bmb, rust, units=None):
    """Writes the BMB workspace (bmb/src/mU.bmb) and the Rust twin (rust/Cargo.toml,
    rust/src/{lib.rs, mU.rs}); only `units` when given (the rest is on disk)."""
    if units is None:
        for d in (os.path.join(bmb, "src"), os.path.join(rust, "src")):
            if os.path.isdir(d):
                shutil.rmtree(d)
            os.makedirs(d)
        twin.write(rust, tree.args)  # Cargo.toml and lib.rs; unit files are rewritten below
        units = range(tree.n_units)
    for u in units:
        for lang, d, ext in (("bmb", bmb, "bmb"), ("rust", rust, "rs")):
            path = os.path.join(d, "src", f"m{u}.{ext}")
            text = tree.text[(lang, u)]
            try:
                if open(path, encoding="utf-8").read() == text:
                    continue
            except OSError:
                pass
            with open(path, "w", encoding="utf-8") as f:
                f.write(text)


def stats(header, commits):
    kinds = {k: 0 for k in KINDS}
    sizes = {}
    for c in commits:
        sizes[len(c["edits"])] = sizes.get(len(c["edits"]), 0) + 1
        for e in c["edits"]:
            kinds[e["op"]] += 1
    return {"commits": len(commits), "profile": header["profile"], "edits": kinds,
            "edits_per_commit": dict(sorted(sizes.items())), "events": header["events"]}


def main(argv=None):
    p = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    sub = p.add_subparsers(dest="cmd", required=True)
    g = sub.add_parser("gen")
    g.add_argument("out")
    g.add_argument("--commits", type=int, required=True)
    g.add_argument("--profile", choices=sorted(PROFILES), required=True)
    g.add_argument("--history-seed", type=int, default=1)
    r = sub.add_parser("replay")
    r.add_argument("history")
    r.add_argument("--bmb", required=True)
    r.add_argument("--rust", required=True)
    r.add_argument("--upto", type=int)
    s = sub.add_parser("stats")
    s.add_argument("history")
    args, rest = p.parse_known_args(argv)
    if args.cmd == "gen":
        ga = gen.parse(["-"] + rest)
        for k in ("commits", "profile", "history_seed"):
            setattr(ga, k, getattr(args, k))
        lines = generate(ga)
        with open(args.out, "w", encoding="utf-8") as f:
            for x in lines:
                f.write(json.dumps(x, sort_keys=True, separators=(",", ":")) + "\n")
        print(json.dumps(stats(lines[0], lines[1:])))
    elif rest:
        p.error("unrecognized arguments: " + " ".join(rest))
    elif args.cmd == "replay":
        header, commits = load(args.history)
        last = None
        for k, tree, _ in replay(header, commits, args.upto):
            last = (k, tree)
        write_tree(last[1], args.bmb, args.rust)
        print(json.dumps({"k": last[0], "digest": last[1].digest()}))
    else:
        header, commits = load(args.history)
        print(json.dumps(stats(header, commits)))


if __name__ == "__main__":
    main()
