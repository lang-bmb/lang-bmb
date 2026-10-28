#!/usr/bin/env python3
"""H-NAME task sets (backlog T1-04): the m1b set with its names taken away.

  python3 bench/hname/gen.py [--bmb PATH]          writes bench/m1b-alpha and bench/m1b-mislead
  python3 bench/hname/gen.py --check [--bmb PATH]  fails if the committed twins differ from a fresh build

The hypothesis (dev-docs/reviews/direction-2026-10-28.md §6.2): in BMB the
contracts carry the meaning that names carry elsewhere, so a model's one-shot
success drops less when names lose their meaning. Two twins of m1b, built by
one renaming applied consistently to the seed, the task prompts, signatures,
hidden contracts, reference and weak solutions, navigation queries and the BSB
mutants:

  alpha     every seed definition gets an opaque name (`d001`..), every unit
            `u1`.., and unit summaries a neutral line. `main` keeps its name.
  mislead   definitions trade names with others of the same kind (a fixed-seed
            permutation with no fixed point), so a name says something false.
            Units and summaries stay.

Definitions the tasks ask the agent to create (unit `tasks`, or new names in a
seed unit) keep the names the statement gives them. Paths `unit::name` are
renamed everywhere; a bare name only inside code of its own unit (a seed file,
or a reference `put` into that unit, or the signature of a task on that unit),
so no other unit's identifier or English word in a prompt changes. Prompts
change only inside backticks and in the signature line.

Each twin is frozen like m1b (`harness.py --set DIR manifest --write`, BSB
`mutants/MANIFEST`), and CI checks that the reference solutions all pass and
the weak ones all fail, as for m1b.
"""
import argparse
import json
import os
import random
import re
import shutil
import subprocess
import sys
import tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(os.path.dirname(HERE))
SRC = os.path.join(ROOT, "bench", "m1b")
sys.path.insert(0, os.path.join(ROOT, "bench", "m1"))
import bsb  # noqa: E402
import harness  # noqa: E402

KEEP = {"main"}


def seed_names(bmb):
    out = subprocess.run([bmb, "map", "L0", os.path.join(SRC, "seed", "src")], capture_output=True, text=True,
                         check=True).stdout
    units, defs = [], []
    for line in out.splitlines():
        j = json.loads(line)
        if j.get("level") != "L0":
            continue
        if j["kind"] == "unit":
            units.append(j["path"])
        else:
            u, n = j["path"].split("::")
            defs.append((u, n, j["def_kind"]))
    return sorted(units), sorted(defs)


def alpha_map(units, defs):
    umap = {u: f"u{i + 1}" for i, u in enumerate(units)}
    dmap = {}
    for i, (u, n, _) in enumerate(defs):
        dmap[(u, n)] = n if n in KEEP else f"d{i + 1:03d}"
    return umap, dmap


def mislead_map(units, defs):
    umap = {u: u for u in units}
    rng = random.Random("H-NAME mislead 1")
    dmap = {}
    for kind in sorted({k for _, _, k in defs}):
        group = [(u, n) for u, n, k in defs if k == kind and n not in KEEP]
        for _ in range(1000):
            names = [n for _, n in group]
            rng.shuffle(names)
            pairs = dict(zip(group, names))
            fixed = any(pairs[g] == g[1] for g in group)
            clash = False
            for u in units:
                taken = [pairs[g] for g in group if g[0] == u] + [n for (v, n), m in dmap.items() if v == u]
                if len(taken) != len(set(taken)):
                    clash = True
            if not fixed and not clash:
                break
        else:
            sys.exit(f"no derangement for {kind}")
        dmap.update(pairs)
    for u, n, _ in defs:
        dmap.setdefault((u, n), n)
    return umap, dmap


class Renamer:
    def __init__(self, umap, dmap):
        self.umap, self.dmap = umap, dmap
        self.by_unit = {}
        for (u, n), m in dmap.items():
            self.by_unit.setdefault(u, {})[n] = m

    def paths(self, text):
        """`unit::name` and `mod unit` and `unit::` prefixes."""
        def path(m):
            u, n = m.group(1), m.group(2)
            if u in self.umap:
                # A new definition in a seed unit keeps its name; the unit is renamed.
                return f"{self.umap[u]}::{self.dmap.get((u, n), n)}"
            return m.group(0)
        text = re.sub(r"\b([a-z_][a-z0-9_]*)::([a-z_][a-z0-9_]*)\b", path, text)
        return text

    def unit_words(self, text):
        for u, v in self.umap.items():
            text = re.sub(rf"\bmod {u}\b", f"mod {v}", text)
        return text

    def bare(self, text, unit):
        names = self.by_unit.get(unit, {})
        if not names:
            return text
        return re.sub(r"(?<!::)\b([a-z_][a-z0-9_]*)\b(?!::)",
                      lambda m: names.get(m.group(1), m.group(1)), text)

    def code(self, text, unit):
        return self.unit_words(self.paths(self.bare(text, unit)))

    def prose(self, text, unit):
        """Prompts: inside backticks, and the signature line."""
        def tick(m):
            inner = m.group(1)
            if inner in self.umap:
                return f"`{self.umap[inner]}`"
            if "::" in inner:
                return f"`{self.paths(inner)}`"
            return f"`{self.bare(inner, unit)}`" if unit else m.group(0)
        out = re.sub(r"`([^`]+)`", tick, text)
        lines = out.split("\n")
        for i, line in enumerate(lines):
            if line.startswith("fn ") and unit:
                lines[i] = self.code(line, unit)
        return "\n".join(lines)

    def value(self, v, unit):
        """JSON values: strings are code (paths always, bare names in `unit`)."""
        if isinstance(v, str):
            return self.code(v, unit) if unit else self.paths(v)
        if isinstance(v, list):
            return [self.value(x, unit) for x in v]
        if isinstance(v, dict):
            return {k: self.value(x, unit) for k, x in v.items()}
        return v


def ops(r, items, task_unit):
    out = []
    for op in items:
        op = dict(op)
        u = op.get("unit") or task_unit
        if "code" in op:
            op["code"] = r.code(op["code"], u)
        for k in ("unit", "path"):
            if k in op and op[k] in r.umap:
                op[k] = r.umap[op[k]]
        if "def" in op:
            op["def"] = r.paths(op["def"])
        out.append(op)
    return out


def task(r, t):
    t = dict(t)
    unit = t.get("unit") or (t["def"].split("::")[0] if "def" in t else None)
    seed_unit = unit if unit in r.umap else None
    if "prompt" in t:
        t["prompt"] = r.prose(t["prompt"], seed_unit)
    for k in ("signature",):
        if k in t and seed_unit:
            t[k] = r.code(t[k], seed_unit)
    if "unit" in t and unit in r.umap:
        t["unit"] = r.umap[unit]
    if "def" in t:
        t["def"] = r.paths(t["def"])
    for k in ("reference", "weak_reference"):
        if k in t:
            t[k] = ops(r, t[k], unit)
    if "accept" in t:
        t["accept"] = {k: (r.paths(v) if isinstance(v, str) else v) for k, v in t["accept"].items()}
    if "truth" in t:
        t["truth"] = r.value(t["truth"], None)
    if "also_check" in t:
        t["also_check"] = [task(r, x) for x in t["also_check"]]
    if "frozen" in t:
        t["frozen"] = r.value(t["frozen"], None)
    return t


def build(name, umap, dmap, neutral_summaries, root):
    r = Renamer(umap, dmap)
    out = os.path.join(root, f"m1b-{name}")
    shutil.rmtree(out, ignore_errors=True)
    os.makedirs(os.path.join(out, "seed", "src"))
    os.makedirs(os.path.join(out, "tasks"))
    shutil.copy(os.path.join(SRC, "prompt.md"), os.path.join(out, "prompt.md"))
    for f in sorted(os.listdir(os.path.join(SRC, "seed", "src"))):
        u = f[:-4]
        text = open(os.path.join(SRC, "seed", "src", f)).read()
        text = r.code(text, u)
        if neutral_summaries:
            text = re.sub(r'#\[summary\("[^"]*"\)\]', f'#[summary("unit {umap[u]}")]', text)
        with open(os.path.join(out, "seed", "src", f"{umap[u]}.bmb"), "w") as fh:
            fh.write(text)
    for f in sorted(os.listdir(os.path.join(SRC, "tasks"))):
        t = json.load(open(os.path.join(SRC, "tasks", f)))
        with open(os.path.join(out, "tasks", f), "w") as fh:
            json.dump(task(r, t), fh, ensure_ascii=False, indent=1)
            fh.write("\n")
    md = bsb.mutants_dir(SRC)
    if os.path.isdir(md):
        os.makedirs(bsb.mutants_dir(out))
        for f in sorted(os.listdir(md)):
            if not f.endswith(".json"):
                continue
            m = json.load(open(os.path.join(md, f)))
            t = json.load(open(os.path.join(SRC, "tasks", f)))
            unit = t.get("unit")
            m = r.value(m, unit if unit in umap else None)
            with open(os.path.join(bsb.mutants_dir(out), f), "w") as fh:
                json.dump(m, fh, ensure_ascii=False, indent=1)
                fh.write("\n")
        with open(os.path.join(bsb.mutants_dir(out), "MANIFEST"), "w") as fh:
            fh.write(bsb.manifest_text(out))
    harness.SET = out
    with open(os.path.join(out, "MANIFEST"), "w") as fh:
        fh.write(harness.manifest_text())
    with open(os.path.join(out, "README.md"), "w") as fh:
        fh.write(f"# m1b-{name} (H-NAME, 백로그 T1-04)\n\n`bench/hname/gen.py`가 `bench/m1b`에서 만든 쌍둥이 과제 집합이다(동결). "
                 "직접 고치지 않고 생성기로 다시 만든다. 규칙은 생성기 머리 주석에 있다.\n")
    return out


def tree(d):
    out = {}
    for base, _, fs in os.walk(d):
        for f in fs:
            p = os.path.join(base, f)
            out[os.path.relpath(p, d)] = open(p, "rb").read()
    return out


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--bmb", default=os.path.join(ROOT, "target", "debug", "bmb"))
    ap.add_argument("--check", action="store_true")
    args = ap.parse_args()
    units, defs = seed_names(args.bmb)
    root = tempfile.mkdtemp(prefix="hname-") if args.check else os.path.join(ROOT, "bench")
    try:
        for name, (umap, dmap), neutral in (("alpha", alpha_map(units, defs), True),
                                            ("mislead", mislead_map(units, defs), False)):
            out = build(name, umap, dmap, neutral, root)
            if args.check and tree(out) != tree(os.path.join(ROOT, "bench", f"m1b-{name}")):
                sys.exit(f"bench/m1b-{name} differs from bench/hname/gen.py output: regenerate it")
            print(out if not args.check else f"bench/m1b-{name} ok")
    finally:
        if args.check:
            shutil.rmtree(root, ignore_errors=True)


if __name__ == "__main__":
    main()
