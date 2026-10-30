#!/usr/bin/env python3
"""H-COMMUTE: concurrent edit pairs with semantic conflicts (backlog T1-19).

  semantic.py gen OUT.jsonl [--seed S] [--per-family N] [--defs D]
  semantic.py manifest [--check]           (regenerate the frozen set, compare
                                            with MANIFEST, or rewrite MANIFEST)
  semantic.py run PAIRS.jsonl --work DIR [--bmb PATH] [--limit N] [--rows R.jsonl] > results.md
  semantic.py report PAIRS.jsonl R.jsonl > results.md

Each pair is a common base and two edits A and B made on it. The base is a
BCB workspace (bench/m3/gen.py through bench/bcb/history.py's Tree, D
definitions) with a small scenario of definitions placed into its unit files
at drawn positions; A and B edit the scenario (put, replace or remove whole
definitions). The scenario families (FAMILIES below) include semantic
conflicts (A strengthens a callee's requires or weakens its ensures, changes
a spec fn body a proof of B reveals, removes a definition B calls; two edits
that each weaken one callee of a shared caller; a refinement that holds only
through spec fns B's reader does not reveal, or through a spec fn body B
changed), refinements that should commute, and independent edits. The
drawn constants make the merge prove in some pairs of a family and fail in
others.

Ground truth is not the generator's intent: `run` merges each pair at
definition level (A's and B's operations on the base) and proves the result,
and only pairs whose base, A and B each prove are used. Each pair is also
merged as text (`git merge-file` per unit file) and the clean merges are
proved (a failure is a git false merge), and judged by judge.py (ADR-0065
and its variants) from the three versions alone. Since T1-29 the conflicts
are finished on the merge (`judge.resolve`, ADR-0065 §4) and compared with
proving the whole merge (the report's last section).

The pair file holds the background parameters and, per pair, the scenario's
operations, so it is a deterministic function of (seed, per-family, defs);
MANIFEST pins the SHA-256 of this file, judge.py and the generated set.
"""
import argparse
import contextlib
import copy
import hashlib
import json
import os
import random
import shutil
import subprocess
import sys
import tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "..", "..", ".."))
sys.path.insert(0, os.path.join(ROOT, "bench", "bcb"))
sys.path.insert(0, HERE)
import history  # noqa: E402
import judge  # noqa: E402

FORMAT = 1
FROZEN = {"seed": 1, "per_family": 16, "defs": 100}


# --- BMB text ---------------------------------------------------------------

def spec(name, params, body):
    return f"    spec fn {name}({params}) -> bool {{\n        {body}\n    }}"


def fn(name, params, ret, body, uses=(), requires=(), ensures=()):
    lines = ["    #[effects(pure)]"]
    if uses:
        lines.append(f"    #[uses({', '.join(uses)})]")
    sig = f"    fn {name}({params}) -> {ret}"
    if requires or ensures:
        lines.append(sig)
        if requires:
            lines.append(f"        requires {', '.join(requires)},")
        if ensures:
            lines.append(f"        ensures {', '.join(ensures)},")
        lines.append("    {")
    else:
        lines.append(sig + " {")
    lines += ["        " + b for b in body]
    lines.append("    }")
    return "\n".join(lines)


class Scenario:
    """The definitions of one pair: roles placed in units, and the operations
    of the base, A and B."""

    def __init__(self, rng, prefix, n_units, roles, same_unit):
        self.p = prefix
        u = rng.randrange(n_units)
        self.unit = {r: (u if same_unit else rng.randrange(n_units)) for r in roles}
        self.base, self.a, self.b = [], [], []

    def name(self, role):
        return f"{self.p}_{role}"

    def ref(self, role, cur):
        """`role` spelled from `cur`'s unit."""
        n = self.name(role)
        return n if self.unit[role] == self.unit[cur] else f"m{self.unit[role]}::{n}"

    def refs(self, roles, cur):
        return [self.ref(r, cur) for r in roles]

    def put(self, ops, role, text):
        ops.append({"op": "put", "unit": self.unit[role], "name": self.name(role), "text": text})

    def set(self, ops, role, text):
        ops.append({"op": "set", "name": self.name(role), "text": text})

    def remove(self, ops, role):
        ops.append({"op": "remove", "name": self.name(role)})


# --- families ----------------------------------------------------------------
# Each takes (rng, Scenario factory) and returns (scenario, params).

def f_pre_strengthen(rng, mk):
    """A adds a requires clause (a base predicate) to f; B adds a caller of f
    that reveals the predicate and may or may not meet it."""
    s = mk(["in", "cap", "out", "f", "g", "d"])
    h0 = rng.randint(50, 900)
    hc = rng.randint(h0 - 40, h0 + 40)
    v = rng.randint(max(0, h0 - 60), h0)  # near the bound, where the added clause can bite
    w = rng.randint(0, hc)
    S = s
    s.put(s.base, "in", spec(S.name("in"), "x: i64", f"0 <= x && x <= {h0}"))
    s.put(s.base, "cap", spec(S.name("cap"), "x: i64", f"0 <= x && x <= {hc}"))
    s.put(s.base, "out", spec(S.name("out"), "r: i64", "0 <= r && r <= 2000"))

    def f_text(req):
        return fn(S.name("f"), "x: i64", "(r: i64)", ["x + 1"], uses=S.refs(["in", "cap", "out"], "f"),
                  requires=[f"{S.ref(q, 'f')}(x)" for q in req], ensures=[f"{S.ref('out', 'f')}(r)"])
    s.put(s.base, "f", f_text(["in"]))
    s.put(s.base, "g", fn(S.name("g"), "x: i64", "(r: i64)", ["x + 1"], uses=S.refs(["cap", "out"], "g"),
                          requires=[f"{S.ref('cap', 'g')}(x)"], ensures=[f"{S.ref('out', 'g')}(r)"]))
    s.set(s.a, "f", f_text(["in", "cap"]))
    s.put(s.b, "d", fn(S.name("d"), "", "i64", [f"{S.ref('f', 'd')}({v}) + {S.ref('g', 'd')}({w})"],
                       uses=S.refs(["in", "cap", "out"], "d")))
    return s, {"h0": h0, "hc": hc, "v": v}


def f_post_weaken(rng, mk):
    """A swaps f's ensures for a weaker base predicate; B adds a caller that
    relies on the old ensures (revealing the new predicate or not), or
    tightens what an existing caller promises."""
    s = mk(["in", "lo", "hi", "need", "wide", "f", "d"])
    S = s
    e0 = rng.randint(100, 500)
    e1 = rng.randint(e0 + 1, 900)
    b = rng.randint(1, e0)
    n = rng.randint(e0, 1000)
    change = rng.random() < 0.4
    reveal_hi = change or rng.random() < 0.6
    s.put(s.base, "in", spec(S.name("in"), "x: i64", "0 <= x && x <= 1000"))
    s.put(s.base, "lo", spec(S.name("lo"), "r: i64", f"0 <= r && r <= {e0}"))
    s.put(s.base, "hi", spec(S.name("hi"), "r: i64", f"0 <= r && r <= {e1}"))
    s.put(s.base, "need", spec(S.name("need"), "r: i64", f"r <= {n}"))
    s.put(s.base, "wide", spec(S.name("wide"), "r: i64", "r <= 1000"))

    def f_text(post):
        return fn(S.name("f"), "x: i64", "(r: i64)", [f"if x > {b} {{ {b} }} else {{ x }}"],
                  uses=S.refs(["in", post], "f"), requires=[f"{S.ref('in', 'f')}(x)"],
                  ensures=[f"{S.ref(post, 'f')}(r)"])

    def d_text(post):
        return fn(S.name("d"), "x: i64", "(r: i64)", [f"{S.ref('f', 'd')}(x)"],
                  uses=S.refs(["in", "lo"] + (["hi"] if reveal_hi else []) + [post], "d"),
                  requires=[f"{S.ref('in', 'd')}(x)"], ensures=[f"{S.ref(post, 'd')}(r)"])
    s.put(s.base, "f", f_text("lo"))
    s.set(s.a, "f", f_text("hi"))
    if change:
        s.put(s.base, "d", d_text("wide"))
        s.set(s.b, "d", d_text("need"))
    else:
        s.put(s.b, "d", d_text("need"))
    return s, {"e0": e0, "e1": e1, "n": n, "reveal_hi": reveal_hi, "b_changes_caller": change}


def f_spec_body(rng, mk):
    """A changes the body of a spec fn f's ensures uses; B adds a caller that
    reveals it (and may rely on the old bound) or passes it through by name."""
    s = mk(["in", "lo", "need", "f", "d"])
    S = s
    c0 = rng.randint(100, 800)
    b = rng.randint(1, 90)
    ca = rng.randint(b, 900)
    n = rng.randint(c0, 1000)
    through = rng.random() < 0.3
    s.put(s.base, "in", spec(S.name("in"), "x: i64", "0 <= x && x <= 1000"))
    lo = lambda c: spec(S.name("lo"), "r: i64", f"0 <= r && r <= {c}")  # noqa: E731
    s.put(s.base, "lo", lo(c0))
    s.put(s.base, "need", spec(S.name("need"), "r: i64", f"r <= {n}"))
    s.put(s.base, "f", fn(S.name("f"), "x: i64", "(r: i64)", [f"if x > {b} {{ {b} }} else {{ x }}"],
                          uses=S.refs(["in", "lo"], "f"), requires=[f"{S.ref('in', 'f')}(x)"],
                          ensures=[f"{S.ref('lo', 'f')}(r)"]))
    s.set(s.a, "lo", lo(ca))
    if through:
        d = fn(S.name("d"), "x: i64", "(r: i64)", [f"{S.ref('f', 'd')}(x)"],
               requires=[f"{S.ref('in', 'd')}(x)"], ensures=[f"{S.ref('lo', 'd')}(r)"])
    else:
        d = fn(S.name("d"), "x: i64", "(r: i64)", [f"{S.ref('f', 'd')}(x)"], uses=S.refs(["in", "lo", "need"], "d"),
               requires=[f"{S.ref('in', 'd')}(x)"], ensures=[f"{S.ref('need', 'd')}(r)"])
    s.put(s.b, "d", d)
    return s, {"c0": c0, "ca": ca, "n": n, "through": through}


def f_pre_narrow(rng, mk):
    """A narrows the body of the spec fn f requires; B adds a caller that
    reveals it to prove it for its argument, or changes the argument of an
    existing one."""
    s = mk(["in", "f", "d"])
    S = s
    h0 = rng.randint(100, 900)
    ha = rng.randint(10, h0 - 1)
    v = rng.randint(0, h0)
    change = rng.random() < 0.4

    def inn(h):
        return spec(S.name("in"), "x: i64", f"0 <= x && x <= {h}")

    def d(arg):
        return fn(S.name("d"), "", "i64", [f"{S.ref('f', 'd')}({arg})"], uses=[S.ref("in", "d")])
    s.put(s.base, "in", inn(h0))
    s.put(s.base, "f", fn(S.name("f"), "x: i64", "i64", ["x + 1"], uses=[S.ref("in", "f")],
                          requires=[f"{S.ref('in', 'f')}(x)"]))
    s.set(s.a, "in", inn(ha))
    if change:
        s.put(s.base, "d", d(rng.randint(0, ha)))
        s.set(s.b, "d", d(v))
    else:
        s.put(s.b, "d", d(v))
    return s, {"h0": h0, "ha": ha, "v": v, "b_changes_caller": change}


def f_pre_weaken(rng, mk):
    """Refinement: A drops a requires clause of f; B adds a caller that meets
    the old requires (revealing them, or passing them through by name)."""
    s = mk(["in", "nz", "f", "d"])
    S = s
    h0 = rng.randint(50, 900)
    k = rng.randint(0, h0)
    v = rng.randint(0, h0 - 1)
    v += v >= k
    opaque = rng.random() < 0.4
    s.put(s.base, "in", spec(S.name("in"), "x: i64", f"0 <= x && x <= {h0}"))
    s.put(s.base, "nz", spec(S.name("nz"), "x: i64", f"x != {k}"))

    def f_text(req):
        return fn(S.name("f"), "x: i64", "i64", ["x + 1"], uses=[S.ref("in", "f")],
                  requires=[f"{S.ref(q, 'f')}(x)" for q in req])
    s.put(s.base, "f", f_text(["in", "nz"]))
    s.set(s.a, "f", f_text(["in"]))
    if opaque:
        d = fn(S.name("d"), "x: i64", "i64", [f"{S.ref('f', 'd')}(x)"],
               requires=[f"{S.ref('in', 'd')}(x)", f"{S.ref('nz', 'd')}(x)"])
    else:
        d = fn(S.name("d"), "", "i64", [f"{S.ref('f', 'd')}({v})"], uses=S.refs(["in", "nz"], "d"))
    s.put(s.b, "d", d)
    return s, {"h0": h0, "k": k, "v": v, "opaque": opaque}


def f_post_strengthen(rng, mk):
    """Refinement: A adds an ensures clause to f; B adds a caller that relies
    on the old ensures."""
    s = mk(["in", "lo", "tight", "need", "f", "d"])
    S = s
    c = rng.randint(100, 900)
    b = rng.randint(1, c)
    t = rng.randint(b, c)
    n = rng.randint(c, 1000)
    s.put(s.base, "in", spec(S.name("in"), "x: i64", "0 <= x && x <= 1000"))
    s.put(s.base, "lo", spec(S.name("lo"), "r: i64", f"0 <= r && r <= {c}"))
    s.put(s.base, "tight", spec(S.name("tight"), "r: i64", f"r <= {t}"))
    s.put(s.base, "need", spec(S.name("need"), "r: i64", f"r <= {n}"))

    def f_text(post):
        return fn(S.name("f"), "x: i64", "(r: i64)", [f"if x > {b} {{ {b} }} else {{ x }}"],
                  uses=S.refs(["in"] + post, "f"), requires=[f"{S.ref('in', 'f')}(x)"],
                  ensures=[f"{S.ref(q, 'f')}(r)" for q in post])
    s.put(s.base, "f", f_text(["lo"]))
    s.set(s.a, "f", f_text(["lo", "tight"]))
    s.put(s.b, "d", fn(S.name("d"), "x: i64", "(r: i64)", [f"{S.ref('f', 'd')}(x)"],
                       uses=S.refs(["in", "lo", "need"], "d"), requires=[f"{S.ref('in', 'd')}(x)"],
                       ensures=[f"{S.ref('need', 'd')}(r)"]))
    return s, {"c": c, "t": t, "n": n}


def f_opaque_rewrite(rng, mk):
    """A rewrites f's requires through spec fns f reveals (an equal or a
    wider predicate); B's new caller reveals both, one or none (2D-19)."""
    s = mk(["pos", "alt", "f", "d"])
    S = s
    h = rng.randint(50, 900)
    wide = rng.random() < 0.5
    ha = rng.randint(h + 1, 1000) if wide else h
    reveal = rng.choice(["none", "pos", "both"])
    s.put(s.base, "pos", spec(S.name("pos"), "x: i64", f"0 < x && x <= {h}"))
    s.put(s.base, "alt", spec(S.name("alt"), "x: i64", f"0 < x && x <= {ha}"))

    def f_text(req):
        return fn(S.name("f"), "x: i64", "i64", ["x - 1"], uses=S.refs(["alt", "pos"], "f"),
                  requires=[f"{S.ref(req, 'f')}(x)"])
    s.put(s.base, "f", f_text("pos"))
    s.set(s.a, "f", f_text("alt"))
    uses = {"none": [], "pos": ["pos"], "both": ["alt", "pos"]}[reveal]
    s.put(s.b, "d", fn(S.name("d"), "x: i64", "i64", [f"{S.ref('f', 'd')}(x)"], uses=S.refs(uses, "d"),
                       requires=[f"{S.ref('pos', 'd')}(x)"]))
    return s, {"h": h, "ha": ha, "reveal": reveal}


def f_refine_vs_spec(rng, mk):
    """A weakens f's requires from q to a (a refinement while q's body
    implies a); B changes q's body so that it no longer does and adds a
    caller that meets q (revealing q and a)."""
    s = mk(["q", "a", "f", "d"])
    S = s
    q0 = rng.randint(20, 500)
    a = rng.randint(2, q0)
    qb = rng.randint(1, a - 1)
    v = rng.randint(qb, q0)
    qq = lambda c: spec(S.name("q"), "x: i64", f"x >= {c}")  # noqa: E731
    s.put(s.base, "q", qq(q0))
    s.put(s.base, "a", spec(S.name("a"), "x: i64", f"x >= {a}"))

    def f_text(req):
        return fn(S.name("f"), "x: i64", "i64", ["if x > 0 { x - 1 } else { 0 }"],
                  requires=[f"{S.ref(req, 'f')}(x)"])
    s.put(s.base, "f", f_text("q"))
    s.set(s.a, "f", f_text("a"))
    s.set(s.b, "q", qq(qb))
    s.put(s.b, "d", fn(S.name("d"), "", "i64", [f"{S.ref('f', 'd')}({v})"], uses=S.refs(["a", "q"], "d")))
    return s, {"q0": q0, "a": a, "qb": qb, "v": v}


def f_two_weakenings(rng, mk):
    """A weakens g's ensures, B weakens k's; their shared caller h needs the
    sum of the two bounds to stay under its own."""
    s = mk(["in", "g0", "ga", "k0", "kb", "hh", "g", "k", "h"])
    S = s
    g0, k0 = rng.randint(50, 300), rng.randint(50, 300)
    ga, kb = rng.randint(g0 + 1, 600), rng.randint(k0 + 1, 600)
    hb = rng.randint(max(ga + k0, g0 + kb), ga + kb + 100)
    s.put(s.base, "in", spec(S.name("in"), "x: i64", "0 <= x && x <= 1000"))
    for role, c in (("g0", g0), ("ga", ga), ("k0", k0), ("kb", kb)):
        s.put(s.base, role, spec(S.name(role), "r: i64", f"0 <= r && r <= {c}"))
    s.put(s.base, "hh", spec(S.name("hh"), "r: i64", f"r <= {hb}"))

    def leaf(role, post, bound):
        return fn(S.name(role), "x: i64", "(r: i64)", [f"if x > {bound} {{ {bound} }} else {{ x }}"],
                  uses=S.refs(["in", post], role), requires=[f"{S.ref('in', role)}(x)"],
                  ensures=[f"{S.ref(post, role)}(r)"])
    bg, bk = rng.randint(1, g0), rng.randint(1, k0)
    s.put(s.base, "g", leaf("g", "g0", bg))
    s.put(s.base, "k", leaf("k", "k0", bk))
    s.put(s.base, "h", fn(S.name("h"), "x: i64", "(r: i64)",
                          [f"{S.ref('g', 'h')}(x) + {S.ref('k', 'h')}(x)"],
                          uses=S.refs(["in", "g0", "ga", "k0", "kb", "hh"], "h"),
                          requires=[f"{S.ref('in', 'h')}(x)"], ensures=[f"{S.ref('hh', 'h')}(r)"]))
    s.set(s.a, "g", leaf("g", "ga", bg))
    s.set(s.b, "k", leaf("k", "kb", bk))
    return s, {"ga": ga, "kb": kb, "hb": hb}


def f_body_vs_caller(rng, mk):
    """The contract firewall: A changes f's body under the same contract; B
    adds a caller that relies on the contract."""
    s = mk(["in", "lo", "need", "f", "d"])
    S = s
    c = rng.randint(100, 900)
    b, b2 = rng.randint(1, c), rng.randint(1, c)
    n = rng.randint(c, 1000)
    s.put(s.base, "in", spec(S.name("in"), "x: i64", "0 <= x && x <= 1000"))
    s.put(s.base, "lo", spec(S.name("lo"), "r: i64", f"0 <= r && r <= {c}"))
    s.put(s.base, "need", spec(S.name("need"), "r: i64", f"r <= {n}"))

    def f_text(bound):
        return fn(S.name("f"), "x: i64", "(r: i64)", [f"if x > {bound} {{ {bound} }} else {{ x }}"],
                  uses=S.refs(["in", "lo"], "f"), requires=[f"{S.ref('in', 'f')}(x)"],
                  ensures=[f"{S.ref('lo', 'f')}(r)"])
    s.put(s.base, "f", f_text(b))
    s.set(s.a, "f", f_text(b2 if b2 != b else b2 % c + 1))
    s.put(s.b, "d", fn(S.name("d"), "x: i64", "(r: i64)", [f"{S.ref('f', 'd')}(x)"],
                       uses=S.refs(["in", "lo", "need"], "d"), requires=[f"{S.ref('in', 'd')}(x)"],
                       ensures=[f"{S.ref('need', 'd')}(r)"]))
    return s, {"c": c, "n": n}


def f_independent(rng, mk):
    """A changes f1's requires and its caller c1 together; B changes f2's
    ensures and adds a caller of f2. Nothing of one is read by the other."""
    s = mk(["in1", "nz1", "f1", "c1", "in2", "lo2", "hi2", "f2", "d2"])
    S = s
    h = rng.randint(50, 900)
    k = rng.randint(0, h)
    v = k + 1 if k < h else k - 1
    e0 = rng.randint(100, 500)
    e1 = rng.randint(e0, 900)
    s.put(s.base, "in1", spec(S.name("in1"), "x: i64", f"0 <= x && x <= {h}"))
    s.put(s.base, "nz1", spec(S.name("nz1"), "x: i64", f"x != {k}"))
    s.put(s.base, "f1", fn(S.name("f1"), "x: i64", "i64", ["x + 1"], uses=[S.ref("in1", "f1")],
                           requires=[f"{S.ref('in1', 'f1')}(x)"]))
    s.put(s.base, "c1", fn(S.name("c1"), "", "i64", [f"{S.ref('f1', 'c1')}({v})"], uses=[S.ref("in1", "c1")]))
    s.set(s.a, "f1", fn(S.name("f1"), "x: i64", "i64", ["x + 1"], uses=[S.ref("in1", "f1")],
                        requires=[f"{S.ref('in1', 'f1')}(x)", f"{S.ref('nz1', 'f1')}(x)"]))
    s.set(s.a, "c1", fn(S.name("c1"), "", "i64", [f"{S.ref('f1', 'c1')}({v})"], uses=S.refs(["in1", "nz1"], "c1")))
    s.put(s.base, "in2", spec(S.name("in2"), "x: i64", "0 <= x && x <= 1000"))
    s.put(s.base, "lo2", spec(S.name("lo2"), "r: i64", f"0 <= r && r <= {e0}"))
    s.put(s.base, "hi2", spec(S.name("hi2"), "r: i64", f"0 <= r && r <= {e1}"))

    def f2(post):
        return fn(S.name("f2"), "x: i64", "(r: i64)", ["if x > 7 { 7 } else { x }"],
                  uses=S.refs(["in2", post], "f2"), requires=[f"{S.ref('in2', 'f2')}(x)"],
                  ensures=[f"{S.ref(post, 'f2')}(r)"])
    s.put(s.base, "f2", f2("lo2"))
    s.set(s.b, "f2", f2("hi2"))
    s.put(s.b, "d2", fn(S.name("d2"), "x: i64", "(r: i64)", [f"{S.ref('f2', 'd2')}(x)"],
                        uses=S.refs(["in2", "hi2"], "d2"), requires=[f"{S.ref('in2', 'd2')}(x)"],
                        ensures=[f"{S.ref('hi2', 'd2')}(r)"]))
    return s, {"h": h, "k": k, "e0": e0, "e1": e1}


def f_remove_vs_call(rng, mk):
    """A removes a definition nothing calls; B adds a caller of it."""
    s = mk(["in", "f", "d"])
    S = s
    h = rng.randint(10, 900)
    v = rng.randint(0, h)
    s.put(s.base, "in", spec(S.name("in"), "x: i64", f"0 <= x && x <= {h}"))
    s.put(s.base, "f", fn(S.name("f"), "x: i64", "i64", ["x + 1"], uses=[S.ref("in", "f")],
                          requires=[f"{S.ref('in', 'f')}(x)"]))
    s.remove(s.a, "f")
    s.put(s.b, "d", fn(S.name("d"), "", "i64", [f"{S.ref('f', 'd')}({v})"], uses=[S.ref("in", "d")]))
    return s, {"h": h}


FAMILIES = {
    # semantic conflicts (the merge may fail)
    "pre_strengthen": f_pre_strengthen,
    "post_weaken": f_post_weaken,
    "spec_body": f_spec_body,
    "pre_narrow": f_pre_narrow,
    "two_weakenings": f_two_weakenings,
    "remove_vs_call": f_remove_vs_call,
    # refinements whose exemption depends on the reader's reveal set or world
    "opaque_rewrite": f_opaque_rewrite,
    "refine_vs_spec": f_refine_vs_spec,
    # refinements and independent edits (the merge proves)
    "pre_weaken": f_pre_weaken,
    "post_strengthen": f_post_strengthen,
    "body_vs_caller": f_body_vs_caller,
    "independent": f_independent,
}


# --- the background and the trees --------------------------------------------

def background(defs):
    """The BCB tree's unit texts as (head, [blocks], tail)."""
    with contextlib.redirect_stdout(sys.stderr):
        args = history.gen_args({"defs": defs, "gen": {
            "seed": 1, "dpu": 20, "depth": 4, "fanout": 2, "window": 3, "contract_share": 0.5,
            "mut_share": 0.15, "scan_share": 0.1, "io_share": 0.05}})
        tree = history.Tree(args)
    units = []
    for u in range(tree.n_units):
        text = tree.text[("bmb", u)]
        lines = text.rstrip("\n").split("\n")
        assert lines[1].startswith("mod ") and lines[-1] == "}", lines[:2]
        blocks = "\n".join(lines[2:-1]).split("\n\n")
        units.append({"head": "\n".join(lines[:2]), "blocks": [b.rstrip("\n") for b in blocks]})
    return units


def block_name(block):
    for line in block.split("\n"):
        t = line.strip()
        for kw in ("spec fn ", "fn "):
            if t.startswith(kw):
                return t[len(kw):].split("(")[0].split("<")[0]
    return None


class Tree:
    """Unit files as ordered definition blocks."""

    def __init__(self, units):
        self.units = [{"head": u["head"], "blocks": list(u["blocks"])} for u in units]

    def copy(self):
        return Tree(self.units)

    def find(self, name):
        for u, unit in enumerate(self.units):
            for i, b in enumerate(unit["blocks"]):
                if block_name(b) == name:
                    return u, i
        raise KeyError(name)

    def apply(self, op):
        if op["op"] == "put":
            blocks = self.units[op["unit"]]["blocks"]
            blocks.insert(min(op["at"], len(blocks)), op["text"])
        elif op["op"] == "set":
            u, i = self.find(op["name"])
            self.units[u]["blocks"][i] = op["text"]
        elif op["op"] == "remove":
            u, i = self.find(op["name"])
            del self.units[u]["blocks"][i]
        else:
            raise ValueError(op["op"])

    def text(self, u):
        unit = self.units[u]
        return unit["head"] + "\n" + "\n\n".join(unit["blocks"]) + "\n}\n"

    def write(self, d):
        shutil.rmtree(d, ignore_errors=True)
        src = os.path.join(d, "src")
        os.makedirs(src)
        for u in range(len(self.units)):
            with open(os.path.join(src, f"m{u}.bmb"), "w") as f:
                f.write(self.text(u))
        return src


def trees(bg, pair):
    """The base, A, B and the definition-level merge (None when both write
    one definition)."""
    base = Tree(bg)
    for op in pair["base"]:
        base.apply(op)
    a, b = base.copy(), base.copy()
    for op in pair["a"]:
        a.apply(op)
    for op in pair["b"]:
        b.apply(op)
    names = lambda ops: {op["name"] for op in ops}  # noqa: E731
    m = None
    if not names(pair["a"]) & names(pair["b"]):
        m = base.copy()
        for op in pair["a"] + pair["b"]:
            m.apply(op)
    return base, a, b, m


# --- gen ----------------------------------------------------------------------

def generate(seed, per_family, defs):
    bg = background(defs)
    n_units = len(bg)
    rng = random.Random(f"commute-semantic:{seed}")
    header = {"format": FORMAT, "set": "commute-semantic", "seed": seed, "per_family": per_family,
              "defs": defs, "families": sorted(FAMILIES)}
    lines = [header]
    k = 0
    for fam in sorted(FAMILIES):
        for _ in range(per_family):
            same_unit = rng.random() < 0.5
            prefix = f"s{k}"
            s, params = FAMILIES[fam](rng, lambda roles: Scenario(rng, prefix, n_units, roles, same_unit))
            # Positions: base definitions among the background's, B's new
            # ones anywhere in the base (sometimes right after a definition
            # A changes, where git sees one hunk).
            sizes = [len(u["blocks"]) for u in bg]
            for op in s.base:
                sizes[op["unit"]] += 1
                op["at"] = rng.randint(0, sizes[op["unit"]] - 1)
            for op in s.a + s.b:
                if op["op"] == "put":
                    op["at"] = rng.randint(0, sizes[op["unit"]])
            lines.append({"k": k, "family": fam, "same_unit": same_unit, "params": params,
                          "base": s.base, "a": s.a, "b": s.b})
            k += 1
    return lines


def dump(lines):
    return "".join(json.dumps(x, sort_keys=True, separators=(",", ":")) + "\n" for x in lines)


def sha(data):
    return hashlib.sha256(data if isinstance(data, bytes) else data.encode()).hexdigest()


def manifest_lines():
    out = []
    for f in ("semantic.py", "judge.py", "run.py"):
        out.append(f"{sha(open(os.path.join(HERE, f), 'rb').read())}  {f}")
    frozen = dump(generate(**FROZEN))
    out.append(f"{sha(frozen)}  pairs/semantic-s{FROZEN['seed']}-n{FROZEN['per_family']}-d{FROZEN['defs']}.jsonl")
    return out


# --- run ----------------------------------------------------------------------

def text_merge(base, a, b, work):
    """Merged unit texts and the number of units `git merge-file` conflicts on."""
    merged, conflicts = {}, 0
    for u in range(len(base.units)):
        o, x, y = base.text(u), a.text(u), b.text(u)
        if x == y or y == o:
            merged[u] = x
            continue
        if x == o:
            merged[u] = y
            continue
        paths = []
        for name, text in (("a", x), ("o", o), ("b", y)):
            p = os.path.join(work, f"{u}-{name}")
            with open(p, "w") as f:
                f.write(text)
            paths.append(p)
        r = subprocess.run(["git", "merge-file", "-p", *paths], capture_output=True, text=True)
        if r.returncode != 0:
            conflicts += 1
        merged[u] = r.stdout
    return merged, conflicts


def run(args):
    lines = [json.loads(x) for x in open(args.pairs)]
    header, pairs = lines[0], lines[1:]
    if args.limit:
        pairs = pairs[:args.limit]
    os.makedirs(args.work, exist_ok=True)
    bmb = judge.Bmb(args.bmb, os.path.join(args.work, "cache"))
    bg = background(header["defs"])
    tmp = tempfile.mkdtemp(prefix="commute-sem-", dir=args.work)
    rows = []
    for pair in pairs:
        base, a, b, m = trees(bg, pair)
        src = {n: t.write(os.path.join(tmp, n)) for n, t in (("o", base), ("a", a), ("b", b))}
        row = {"k": pair["k"], "family": pair["family"], "params": pair["params"], "same_unit": pair["same_unit"]}
        valid = {n: bmb.proves(s, "verify_sides") for n, s in src.items()}
        row["valid"] = all(valid.values())
        if not row["valid"]:
            row["invalid"] = [n for n, ok in valid.items() if not ok]
            rows.append(row)
            print(json.dumps(row), file=sys.stderr, flush=True)
            continue
        bmb.take()
        src_m = m.write(os.path.join(tmp, "m")) if m else None
        row["merge_proves"], row["merge_encoded"] = (
            bmb.proves_stats(src_m, "verify_merge") if m else (None, None))
        merged, row["text_conflicts"] = text_merge(base, a, b, tmp)
        row["text_proves"] = None
        if row["text_conflicts"] == 0:
            d = os.path.join(tmp, "t", "src")
            shutil.rmtree(os.path.join(tmp, "t"), ignore_errors=True)
            os.makedirs(d)
            for u, text in merged.items():
                with open(os.path.join(d, f"m{u}.bmb"), "w") as f:
                    f.write(text)
            row["text_proves"] = bmb.proves(d, "verify_text")
        verdict = judge.judge(bmb, src["o"], src["a"], src["b"], old=True)
        row.update(verdict)
        # ADR-0065 §4 (T1-29): finish the conflicts on the merge, and the
        # whole merge proved without caches for the cost comparison.
        row["resolve"] = {v: judge.resolve(bmb, verdict, src_m, v) for v in judge.RESOLVE}
        row["whole"] = None
        if any(r and r["merge"] for r in row["resolve"].values()):
            row["whole"] = bmb.verify(src_m, kind="verify_whole_cold")
        row["cost"] = bmb.take()
        rows.append(row)
        print(json.dumps(row), file=sys.stderr, flush=True)
    shutil.rmtree(tmp, ignore_errors=True)
    if args.rows:
        with open(args.rows, "w") as f:
            for r in rows:
                f.write(json.dumps(r, sort_keys=True) + "\n")
    report(header, rows, args)


def report(header, rows, args):
    used = [r for r in rows if r["valid"]]
    print(f"## 의미 충돌 집합: 시드 {header['seed']}, 가족당 {header['per_family']}쌍, 배경 정의 {header['defs']}개\n")
    print(f"생성: `python3 dev-docs/experiments/commute/semantic.py gen P.jsonl --seed {header['seed']} "
          f"--per-family {header['per_family']} --defs {header['defs']}`, 측정: `semantic.py run P.jsonl --work W`.\n")
    print(f"쌍 {len(rows)}개 중 기준·A·B가 각자 증명된 쌍 {len(used)}개(나머지는 표본에서 뺌: "
          f"{', '.join(str(r['k']) for r in rows if not r['valid']) or '없음'}).\n")
    verdicts = ("old",) + judge.VARIANTS
    names = {"old": "첫 재생(T1-12)의 규칙(`bmb hash`의 본문·계약 해시와 L2 간선)을 지금 컴파일러로. ADR-0059 뒤로 해시가 전이적이다", "none": "키, 면제 없음", "own": "키 + 정의 자신의 `#[uses]`로 면제",
             "reader": "키 + 읽는 쪽 `#[uses]`로 면제(ADR-0065 초안)", "world": "키 + 읽는 쪽 면제 + 질의가 펼친 술어를 상대가 안 씀"}

    def commute(r, v):
        return r["old"] if v == "old" else r[v]["commute"]

    def summary(rs, title):
        fails = [r for r in rs if r["merge_proves"] is not True]
        proves = [r for r in rs if r["merge_proves"] is True]
        clean = [r for r in rs if r["text_conflicts"] == 0]
        print(f"| {title} | {len(rs)} | {len(proves)} | {len(fails)} | {len(rs) - len(clean)} | "
              f"{sum(1 for r in clean if r['text_proves'] is False)} | "
              + " | ".join(f"{sum(1 for r in rs if commute(r, v) and r['merge_proves'] is not True)}"
                           f"·{sum(1 for r in rs if not commute(r, v) and r['merge_proves'] is True)}"
                           for v in verdicts) + " |")

    print("| 가족 | 쌍 | 병합 증명 | 병합 실패 | git 충돌 | git 거짓 병합 | "
          + " | ".join(f"{v} 위반·거짓 충돌" for v in verdicts) + " |")
    print("| --- | ---: | ---: | ---: | ---: | ---: | " + " | ".join("---:" for _ in verdicts) + " |")
    for fam in header["families"]:
        rs = [r for r in used if r["family"] == fam]
        if rs:
            summary(rs, f"`{fam}`")
    summary(used, "**합**")
    print("\n열 `v 위반·거짓 충돌`: 판정 v가 교환 가능으로 본 쌍 중 정의 단위 병합이 증명되지 않은 수"
          "(건전성 위반) · 충돌로 본 쌍 중 병합이 증명된 수(거짓 충돌). 판정:\n")
    for v in verdicts:
        print(f"- `{v}`: {names[v]}")
    print()
    # detection and costs
    fails = [r for r in used if r["merge_proves"] is not True]
    clean_fail = [r for r in fails if r["text_conflicts"] == 0]
    print("| 항목 | 값 |\n| --- | ---: |")
    print(f"| 병합이 실패하는 쌍(의미 충돌 또는 쓰기 충돌) | {len(fails)} |")
    print(f"| 그중 git이 깨끗이 병합(거짓 병합) | {sum(1 for r in clean_fail if r['text_proves'] is False)} / {len(fails)} |")
    for v in verdicts:
        print(f"| 그중 `{v}`가 충돌로 판정(검출) | {sum(1 for r in fails if not commute(r, v))} / {len(fails)} |")
    asks = sum(r["queries"]["a"][0] + r["queries"]["b"][0] for r in used)
    obls = sum(r["queries"]["a"][1] + r["queries"]["b"][1] for r in used)
    cost = {}
    for r in used:
        for k, (n, s) in r["cost"].items():
            c = cost.setdefault(k, [0, 0.0])
            c[0] += n
            c[1] += s
    print(f"| 면제 질의(읽는 쪽): 행 · 해법기 의무 | {asks} · {obls} |")
    for k in sorted(cost):
        print(f"| CPU `{k}`: 호출 · 초 | {cost[k][0]} · {cost[k][1]:.1f} |")
    print(f"| `id` 쓰기 집합이면 생기는 쓰기 충돌(자기 텍스트로는 없음) | "
          f"{sum(1 for r in used if r['ww_id'] and not r['ww'])} |")
    print("\n| k | 가족 | 매개변수 | 병합 증명 | git 충돌·증명 | " + " | ".join(verdicts) + " | 면제 질의 |")
    print("| ---: | --- | --- | --- | --- | " + " | ".join("---" for _ in verdicts) + " | ---: |")
    for r in rows:
        if not r["valid"]:
            print(f"| {r['k']} | `{r['family']}` | {json.dumps(r['params'])} | 표본 제외: {r['invalid']} | | "
                  + " | ".join("" for _ in verdicts) + " | |")
            continue
        print(f"| {r['k']} | `{r['family']}` | `{json.dumps(r['params'], separators=(',', ':'))}` | {r['merge_proves']} | "
              f"{r['text_conflicts']}·{r['text_proves']} | "
              + " | ".join("교환" if commute(r, v) else "충돌" for v in verdicts)
              + f" | {r['queries']['a'][0] + r['queries']['b'][0]} |")
    if used and all("resolve" in r for r in used):
        print("\n### 비교환 쌍의 해소(T1-29, ADR-0065 결정 4)\n")
        judge.report_resolve([{"verdict": r, "merge_proves": r["merge_proves"], "resolve": r["resolve"],
                               "whole": r["whole"], "cost": r["cost"], "family": r["family"],
                               "merge_encoded": r["merge_encoded"]} for r in used], families=True)


def main():
    ap = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    sub = ap.add_subparsers(dest="cmd", required=True)
    g = sub.add_parser("gen")
    g.add_argument("out")
    g.add_argument("--seed", type=int, default=FROZEN["seed"])
    g.add_argument("--per-family", type=int, default=FROZEN["per_family"])
    g.add_argument("--defs", type=int, default=FROZEN["defs"])
    m = sub.add_parser("manifest")
    m.add_argument("--check", action="store_true")
    r = sub.add_parser("run")
    r.add_argument("pairs")
    r.add_argument("--work", required=True)
    r.add_argument("--bmb", default=os.path.join(ROOT, "target", "debug", "bmb"))
    r.add_argument("--limit", type=int)
    r.add_argument("--rows", help="also write the per-pair rows (JSONL) here")
    t = sub.add_parser("report", help="the report again from the rows of a run")
    t.add_argument("pairs")
    t.add_argument("rows")
    args = ap.parse_args()
    if args.cmd == "gen":
        with open(args.out, "w") as f:
            f.write(dump(generate(args.seed, args.per_family, args.defs)))
    elif args.cmd == "manifest":
        path = os.path.join(HERE, "MANIFEST")
        lines = manifest_lines()
        if args.check:
            want = open(path).read().splitlines()
            if want != lines:
                sys.exit("MANIFEST differs:\n" + "\n".join(lines))
            print("MANIFEST ok")
        else:
            with open(path, "w") as f:
                f.write("\n".join(lines) + "\n")
    elif args.cmd == "report":
        header = json.loads(open(args.pairs).readline())
        report(header, [json.loads(x) for x in open(args.rows)], args)
    else:
        run(args)


if __name__ == "__main__":
    main()
