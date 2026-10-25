#!/usr/bin/env python3
"""Large M1 workspace generator (backlog 3A-01).

Writes OUT/src/<unit>.bmb for a workspace of exactly N definitions in the M1
language (ADR-0016 and compiler/README.md surface rules). Every definition is
meant to be proved by `bmb verify`; the templates are the shapes of bench/m1b
(bounded arithmetic, array scans with invariants, helper chains, `&mut`
fill/map loops) with small constant variations.

Determinism and locality: unit `mU` depends only on (seed, U) and the
parameters, never on N. Calls go from unit U only to units U-window..U, so a
definition in unit U has the same callees and callers (from units U..U+window)
in every workspace that contains unit U+window. Growing N appends units; it
does not change existing ones. scripts that compare sizes (scale.py) rely on
this.

Bodies are rendered from a plan: `Unit` (names, kinds, constants) and
`choose` (callees and result forms, drawn once per unit). The Rust twin
(bench/bnb/twin.py, backlog 3A-05) renders the same plan.

Usage: gen.py OUT --defs N [--defs-per-unit 20 | --units U] [--seed 1]
               [--depth 4] [--fanout 2] [--window 3] [--contract-share 0.5]
               [--mut-share 0.15] [--scan-share 0.10] [--io-share 0.05]
Prints a JSON summary line (counts per kind) on stdout.
"""

import argparse
import json
import os
import random
import shutil
import sys

B = 1000000  # bound of every scalar in the generated code (inb)
BASE_SPECS = ("all_inb", "inb", "inb_below", "within")


def base_specs():
    return {
        "inb": f"""    spec fn inb(x: i64) -> bool {{
        0 - {B} <= x && x <= {B}
    }}
""",
        "all_inb": f"""    spec fn all_inb(a: &[i64]) -> bool {{
        forall|k: usize| k < a.len() ==> 0 - {B} <= #[trigger] a[k] && a[k] <= {B}
    }}
""",
        "inb_below": f"""    spec fn inb_below(a: &[i64], n: usize) -> bool {{
        forall|k: usize| k < n ==> 0 - {B} <= #[trigger] a[k] && a[k] <= {B}
    }}
""",
        "within": """    spec fn within(r: usize, a: &[i64]) -> bool {
        r <= a.len()
    }
""",
    }


def q(unit, cur, name):
    """Call or #[uses] spelling of `unit::name` as seen from unit `cur`."""
    return name if unit == cur else f"m{unit}::{name}"


def attrs(effects, uses):
    out = f"    #[effects({effects})]\n"
    if uses:
        out += "    #[uses(" + ", ".join(sorted(set(uses))) + ")]\n"
    return out


class Unit:
    """Plan of one unit: names and kinds only (bodies are rendered later)."""

    def __init__(self, u, args):
        self.u = u
        rng = random.Random(f"{args.seed}:{u}")
        self.rng = rng
        dpu = args.dpu
        n_mut = round(dpu * args.mut_share)
        n_scan = round(dpu * args.scan_share)
        n_io = round(dpu * args.io_share)
        slots = dpu - len(BASE_SPECS) - n_mut - n_scan - n_io
        if slots < args.depth:
            sys.exit(f"gen.py: {dpu} definitions per unit leave {slots} arithmetic slots; need >= depth {args.depth}")
        self.defs = []  # (name, kind, info)
        i = 0

        def name(stem):
            nonlocal i
            i += 1
            return f"{stem}_{i}"

        # Arithmetic functions, layer by layer (layer 0 = leaves). Every layer
        # gets at least one function so unit 0 can call within itself.
        layer = 0
        while slots > 0:
            if layer == 0:
                tmpl = rng.choice(("shift", "halve", "abs", "clamp"))
                c = rng.randint(1, 1000)
                with_post = slots >= 2 and rng.random() < args.contract_share
                n = name(tmpl)
                self.defs.append((n, "leaf", {"tmpl": tmpl, "c": c, "post": with_post}))
                slots -= 1
                if with_post:
                    self.defs.append((n + "_ok", "post", {"of": n, "tmpl": tmpl, "c": c}))
                    slots -= 1
            else:
                self.defs.append((name("mix"), "chain", {"layer": layer}))
                slots -= 1
            layer = (layer + 1) % args.depth
        for k in range(n_scan):
            self.defs.append((name("count_above" if k % 2 == 0 else "peak"), "scan_count" if k % 2 == 0 else "scan_peak", {"c": rng.randint(0, 1000)}))
        mut_kinds = ("fill", "remap", "reset")
        for k in range(n_mut):
            kind = mut_kinds[k % 3]
            self.defs.append((name(kind), kind, {}))
        for _ in range(n_io):
            self.defs.append((name("show"), "show", {}))
        for s in BASE_SPECS:
            self.defs.append((s, "spec", {}))

    def by_kind(self, *kinds, layer=None):
        return [
            (n, k, info)
            for n, k, info in self.defs
            if k in kinds and (layer is None or (k == "chain" and info["layer"] == layer) or (k == "leaf" and layer == 0))
        ]


def choose(units, u, args):
    """Random choices of unit `u`'s bodies (callees, result form), drawn once
    from the unit's generator in plan order and kept on the unit. Both
    renderers (BMB here, the Rust twin in bench/bnb/twin.py) read them, so the
    two languages get the same call graph."""
    unit = units[u]
    if hasattr(unit, "choices"):
        return unit.choices
    rng = unit.rng
    lo = max(0, u - args.window)

    def pool(*kinds, layer=None):
        out = []
        for v in range(lo, u + 1):
            out += [(v, n) for n, _, _ in units[v].by_kind(*kinds, layer=layer)]
        return out

    def arith_any():
        out = pool("leaf")
        for L in range(1, args.depth):
            out += pool("chain", layer=L)
        return out

    ch = {}
    for n, kind, info in unit.defs:
        if kind == "chain":
            cands = pool("leaf" if info["layer"] == 1 else "chain", layer=info["layer"] - 1)
            callees = [rng.choice(cands) for _ in range(args.fanout)]
            ch[n] = {"callees": callees, "avg": rng.random() < 0.5}
        elif kind in ("scan_peak", "remap", "show"):
            ch[n] = {"callees": [rng.choice(arith_any())]}
        elif kind == "reset":
            fills = [x for x in pool("fill") if x != (u, n)]
            remaps = [x for x in pool("remap") if x != (u, n)]
            ch[n] = {"callees": [rng.choice(fills), rng.choice(remaps)]}
    unit.choices = ch
    return ch


def render_unit(units, u, args):
    unit = units[u]
    ch = choose(units, u, args)

    def inb_uses(callees):
        return ["inb"] + [q(v, u, "inb") for v, _ in callees]

    defs = {}
    for n, kind, info in unit.defs:
        if kind == "spec":
            defs[n] = base_specs()[n]
        elif kind == "post":
            c = info["c"]
            body = {
                "shift": f"r - x <= {c} && x - r <= {c}",
                "halve": "0 - 1 <= 2 * r - x && 2 * r - x <= 1",
                "abs": "r >= 0 && (r == x || r == 0 - x)",
                "clamp": f"0 - {c} <= r && r <= {c}",
            }[info["tmpl"]]
            defs[n] = f"""    spec fn {n}(x: i64, r: i64) -> bool {{
        {body}
    }}
"""
        elif kind == "leaf":
            c = info["c"]
            body = {
                "shift": f"if x > {c} {{\n            x - {c}\n        }} else {{\n            x + {c}\n        }}",
                "halve": "x / 2",
                "abs": "if x < 0 {\n            0 - x\n        } else {\n            x\n        }",
                "clamp": f"if x > {c} {{\n            {c}\n        }} else if x < 0 - {c} {{\n            0 - {c}\n        }} else {{\n            x\n        }}",
            }[info["tmpl"]]
            ens = "inb(r)" + (f", {n}_ok(x, r)" if info["post"] else "")
            uses = ["inb"] + ([n + "_ok"] if info["post"] else [])
            defs[n] = attrs("pure", uses) + f"""    fn {n}(x: i64) -> (r: i64)
        requires inb(x),
        ensures {ens},
    {{
        {body}
    }}
"""
        elif kind == "chain":
            callees = ch[n]["callees"]
            lines = []
            prev = "x"
            for j, (v, cn) in enumerate(callees):
                lines.append(f"        let v{j}: i64 = {q(v, u, cn)}({prev});")
                prev = f"v{j}"
            res = f"(x + v{args.fanout - 1}) / 2" if ch[n]["avg"] else f"v{args.fanout - 1}"
            defs[n] = attrs("pure", inb_uses(callees)) + f"""    fn {n}(x: i64) -> (r: i64)
        requires inb(x),
        ensures inb(r),
    {{
{chr(10).join(lines)}
        {res}
    }}
"""
        elif kind == "scan_count":
            c = info["c"]
            defs[n] = attrs("pure", ["within"]) + f"""    fn {n}(a: &[i64], t: i64) -> (r: usize)
        ensures within(r, a),
    {{
        let mut c: usize = 0;
        let mut k: usize = 0;
        while k < a.len()
            invariant k <= a.len(), c <= k,
            decreases a.len() - k,
        {{
            if a[k] > t && a[k] != {c} {{
                c = c + 1;
            }}
            k = k + 1;
        }}
        c
    }}
"""
        elif kind == "scan_peak":
            callee = ch[n]["callees"][0]
            defs[n] = attrs("pure", ["all_inb"] + inb_uses([callee])) + f"""    fn {n}(a: &[i64]) -> (r: i64)
        requires all_inb(a),
        ensures inb(r),
    {{
        let mut m: i64 = {info["c"]};
        let mut k: usize = 0;
        while k < a.len()
            invariant k <= a.len(), inb(m),
            decreases a.len() - k,
        {{
            let v: i64 = {q(callee[0], u, callee[1])}(a[k]);
            if v > m {{
                m = v;
            }}
            k = k + 1;
        }}
        m
    }}
"""
        elif kind == "fill":
            defs[n] = attrs("pure", ["all_inb", "inb", "inb_below"]) + f"""    fn {n}(a: &mut [i64], v: i64) -> ()
        requires inb(v),
        ensures all_inb(a),
    {{
        let mut k: usize = 0;
        while k < a.len()
            invariant k <= a.len(), inb(v), inb_below(a, k),
            decreases a.len() - k,
        {{
            a[k] = v;
            k = k + 1;
        }}
    }}
"""
        elif kind == "remap":
            callee = ch[n]["callees"][0]
            defs[n] = attrs("pure", ["all_inb"] + inb_uses([callee])) + f"""    fn {n}(a: &mut [i64]) -> ()
        requires all_inb(a),
        ensures all_inb(a),
    {{
        let mut k: usize = 0;
        while k < a.len()
            invariant k <= a.len(), all_inb(a),
            decreases a.len() - k,
        {{
            a[k] = {q(callee[0], u, callee[1])}(a[k]);
            k = k + 1;
        }}
    }}
"""
        elif kind == "reset":
            f, m = ch[n]["callees"]
            uses = ["all_inb", "inb", q(f[0], u, "inb"), q(f[0], u, "all_inb"), q(m[0], u, "all_inb")]
            defs[n] = attrs("pure", uses) + f"""    fn {n}(a: &mut [i64], v: i64) -> ()
        requires inb(v),
        ensures all_inb(a),
    {{
        {q(f[0], u, f[1])}(a, v);
        {q(m[0], u, m[1])}(a);
    }}
"""
        elif kind == "show":
            callee = ch[n]["callees"][0]
            defs[n] = attrs("io", inb_uses([callee])) + f"""    fn {n}(x: i64) -> ()
        requires inb(x),
    {{
        print_i64({q(callee[0], u, callee[1])}(x));
    }}
"""
    body = "\n".join(defs[k] for k in sorted(defs))
    return f'#[summary("생성 단위 {u}: 상한 산술, 배열 훑기, 변경 경로")]\nmod m{u} {{\n{body}}}\n'


def parse(argv=None):
    p = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    p.add_argument("out")
    p.add_argument("--defs", type=int, required=True, help="total definitions N")
    g = p.add_mutually_exclusive_group()
    g.add_argument("--defs-per-unit", dest="dpu", type=int)
    g.add_argument("--units", type=int)
    p.add_argument("--seed", type=int, default=1)
    p.add_argument("--depth", type=int, default=4, help="arithmetic call layers (helper chain depth)")
    p.add_argument("--fanout", type=int, default=2, help="callees per helper chain function")
    p.add_argument("--window", type=int, default=3, help="units back a call may reach")
    p.add_argument("--contract-share", type=float, default=0.5, help="leaves with their own spec fn postcondition")
    p.add_argument("--mut-share", type=float, default=0.15, help="share of &mut functions (fill/remap/reset)")
    p.add_argument("--scan-share", type=float, default=0.10, help="share of array scans")
    p.add_argument("--io-share", type=float, default=0.05, help="share of io functions")
    args = p.parse_args(argv)
    if args.units:
        if args.defs % args.units:
            sys.exit("gen.py: --defs must be a multiple of --units")
        args.dpu = args.defs // args.units
    args.dpu = args.dpu or 20
    if args.defs % args.dpu:
        sys.exit("gen.py: --defs must be a multiple of --defs-per-unit")
    if args.mut_share > 0 and round(args.dpu * args.mut_share) < 3:
        sys.exit("gen.py: --mut-share must give at least 3 &mut functions per unit (fill, remap, reset)")
    return args


def main(argv=None):
    args = parse(argv)
    n_units = args.defs // args.dpu
    units = [Unit(u, args) for u in range(n_units)]
    src = os.path.join(args.out, "src")
    if os.path.isdir(src):
        shutil.rmtree(src)
    os.makedirs(src)
    for u in range(n_units):
        with open(os.path.join(src, f"m{u}.bmb"), "w", encoding="utf-8") as f:
            f.write(render_unit(units, u, args))
    kinds = {}
    for unit in units:
        for _, k, _ in unit.defs:
            kinds[k] = kinds.get(k, 0) + 1
    spec = kinds.get("spec", 0) + kinds.get("post", 0)
    print(json.dumps({
        "defs": sum(kinds.values()), "units": n_units, "spec_fns": spec,
        "exec_fns": sum(kinds.values()) - spec, "kinds": dict(sorted(kinds.items())),
        "params": {k: getattr(args, k) for k in ("seed", "dpu", "depth", "fanout", "window", "contract_share", "mut_share", "scan_share", "io_share")},
    }, sort_keys=False))


if __name__ == "__main__":
    main()
