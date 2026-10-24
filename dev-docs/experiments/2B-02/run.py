#!/usr/bin/env python3
"""2B-02: SMT encodings of structs and enums, compared on small query families.

  run.py                 generate every query, run z3 with seeds 0,1,2 (+ seed 0
                         again for the repeat check), write results.csv and results.md
  run.py --emit DIR      only write the generated .smt2 files (seed 0) to DIR

Encodings (one query = one proof obligation, as BMB sends it: ADR-0018):
  adt     SMT-LIB algebraic datatypes (declare-datatypes). A struct is one
          constant of a datatype sort, an enum is a datatype with one
          constructor per variant, an array of structs is (Array Int S).
  flat    field decomposition. A struct is a vector of scalar terms, an enum is
          a tag Int plus every variant's fields, with inactive fields fixed to 0
          (canonical form), an array of structs is one (Array Int Int) per field.
  flatj   `flat` without the canonical zeros (inactive fields are junk). Only in
          the congruence family, to show why the canonical form is needed.
  hybrid  datatype sorts at the boundary (parameters), field vectors inside the
          symbolic state (selectors as leaves). Only where it differs from both.

Same solver options as compiler/src/verify.rs `header()`: produce-models,
smt.random_seed/sat.random_seed = seed, rlimit 5000000 (scripts/toolchain.env).
Standard library only. Needs the pinned z3 on PATH (or Z3=path).
"""
import argparse
import csv
import os
import re
import subprocess
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
RLIMIT = 5000000
SEEDS = [0, 1, 2]
I64_MIN, I64_MAX = -(2**63), 2**63 - 1
PRELUDE = """(define-fun bmb_abs ((x Int)) Int (ite (>= x 0) x (- x)))
(define-fun bmb_tdiv ((a Int) (b Int)) Int (ite (>= a 0) (ite (> b 0) (div a b) (- (div a (- b)))) (ite (> b 0) (- (div (- a) b)) (div (- a) (- b)))))
(define-fun bmb_trem ((a Int) (b Int)) Int (- a (* b (bmb_tdiv a b))))"""


def num(v):
    return f"(- {-v})" if v < 0 else str(v)


def rng(t, lo=I64_MIN, hi=I64_MAX):
    return f"(and (<= {num(lo)} {t}) (<= {t} {num(hi)}))"


def conj(xs):
    xs = list(xs)
    if not xs:
        return "true"
    return xs[0] if len(xs) == 1 else "(and " + " ".join(xs) + ")"


class Q:
    """One query: declarations, assumptions, goal, model terms to show."""

    def __init__(self):
        self.decls, self.asserts, self.goal, self.model = [], [], "true", []
        self.n = 0

    def const(self, name, sort):
        self.decls.append(f"(declare-const |{name}| {sort})")
        return f"|{name}|"

    def bind(self, term, sort):
        # BMB binds a merged term above 200 characters to a fresh constant
        # (verify.rs `merge`, backlog 1B-26). Same policy here.
        if len(term) <= 200:
            return term
        self.n += 1
        c = self.const(f"t!{self.n}", sort)
        self.asserts.append(f"(= {c} {term})")
        return c

    def text(self, seed):
        lines = [
            "(set-option :produce-models true)",
            f"(set-option :smt.random_seed {seed})",
            f"(set-option :sat.random_seed {seed})",
            f"(set-option :rlimit {RLIMIT})",
            PRELUDE,
        ]
        lines += self.decls
        lines += [f"(assert {a})" for a in self.asserts]
        lines.append(f"(assert (not {self.goal}))")
        lines.append("(check-sat)")
        lines.append("(get-info :rlimit)")
        if self.model:
            lines.append("(get-value (" + " ".join(t for _, t in self.model) + "))")
        return "\n".join(lines) + "\n"


# ---------------------------------------------------------------- families
# Every family returns {encoding: Q}. `bug` variants have a false goal (sat
# expected, with a counterexample); the others must be proved (unsat).

def struct_dt(k):
    fields = " ".join(f"(P.f{j} Int)" for j in range(k))
    return f"(declare-datatypes ((P 0)) (((P {fields}))))"


def s1_update_chain(k, n, bug):
    """`let mut p: P = p0; p.f[i%k] += 1` n times; ensures every field bounded."""
    req = lambda t: rng(t, 0, 1000)
    bound = [1000 + len(range(j, n, k)) for j in range(k)]
    if bug:
        bound[0] -= 1
    out = {}
    # adt: each assignment makes a new struct value = constructor application.
    q = Q()
    q.decls.append(struct_dt(k))
    s = q.const("p", "P")
    q.asserts += [rng(f"(P.f{j} {s})") for j in range(k)] + [req(f"(P.f{j} {s})") for j in range(k)]
    q.model = [("p", s)]
    for i in range(n):
        nxt = q.const(f"p!{i + 1}", "P")
        args = [f"(+ (P.f{j} {s}) 1)" if j == i % k else f"(P.f{j} {s})" for j in range(k)]
        q.asserts.append(f"(= {nxt} (P {' '.join(args)}))")
        s = nxt
    q.goal = conj(f"(<= (P.f{j} {s}) {bound[j]})" for j in range(k))
    out["adt"] = q
    # flat and hybrid: field vector; only the written field's term changes.
    for enc in ("flat", "hybrid"):
        q = Q()
        if enc == "flat":
            cur = [q.const(f"p.f{j}", "Int") for j in range(k)]
            q.model = [(f"p.f{j}", c) for j, c in enumerate(cur)]
        else:
            q.decls.append(struct_dt(k))
            p = q.const("p", "P")
            cur = [f"(P.f{j} {p})" for j in range(k)]
            q.model = [("p", p)]
        q.asserts += [rng(c) for c in cur] + [req(c) for c in cur]
        for i in range(n):
            j = i % k
            cur[j] = q.bind(f"(+ {cur[j]} 1)", "Int")
        q.goal = conj(f"(<= {cur[j]} {bound[j]})" for j in range(k))
        out[enc] = q
    return out


def enum_dt(k):
    ctors = []
    for i in range(k):
        fs = "".join(f" (|E::V{i}.{j}| Int)" for j in range(1 + i % 3))
        ctors.append(f"(|E::V{i}|{fs})")
    return f"(declare-datatypes ((E 0)) (({' '.join(ctors)})))"


def arity(i):
    return 1 + i % 3


def flat_enum(q, name, k, canonical=True, ranges=True):
    tag = q.const(f"{name}.tag", "Int")
    q.asserts.append(f"(and (<= 0 {tag}) (< {tag} {k}))")
    fields = []
    for i in range(k):
        fs = [q.const(f"{name}.V{i}.{j}", "Int") for j in range(arity(i))]
        if ranges:
            q.asserts.append(f"(=> (= {tag} {i}) {conj(rng(f) for f in fs)})")
        if canonical:
            q.asserts.append(f"(=> (not (= {tag} {i})) {conj(f'(= {f} 0)' for f in fs)})")
        fields.append(fs)
    return tag, fields


def e1_match(k, bug):
    """`match e { V_i(a, b, ..) => a + b + .. + i }`, payloads in [0, 1000]; ensures r >= 0."""
    goal = lambda r: f"(>= {r} {1 if bug else 0})"
    out = {}
    q = Q()
    q.decls.append(enum_dt(k))
    e = q.const("e", "E")
    q.model = [("e", e)]
    arms = []
    for i in range(k):
        fs = [f"(|E::V{i}.{j}| {e})" for j in range(arity(i))]
        q.asserts.append(f"(=> ((_ is |E::V{i}|) {e}) {conj(rng(f) for f in fs)})")
        q.asserts.append(f"(=> ((_ is |E::V{i}|) {e}) {conj(rng(f, 0, 1000) for f in fs)})")
        arms.append((f"((_ is |E::V{i}|) {e})", f"(+ {' '.join(fs)} {i})"))
    r = arms[-1][1]
    for c, v in reversed(arms[:-1]):
        r = f"(ite {c} {v} {r})"
    q.goal = goal(r)
    out["adt"] = q
    q = Q()
    tag, fields = flat_enum(q, "e", k)
    q.model = [("e.tag", tag)] + [(f"e.V{i}.{j}", f) for i, fs in enumerate(fields) for j, f in enumerate(fs)]
    for i, fs in enumerate(fields):
        q.asserts.append(f"(=> (= {tag} {i}) {conj(rng(f, 0, 1000) for f in fs)})")
    r = f"(+ {' '.join(fields[-1])} {k - 1})"
    for i in reversed(range(k - 1)):
        r = f"(ite (= {tag} {i}) (+ {' '.join(fields[i])} {i}) {r})"
    q.goal = goal(r)
    out["flat"] = q
    return out


OPT_DT = "(declare-datatypes ((Opt 0)) (((None) (Some (Some.0 Int)))))"


def e2_option_chain(n, bug):
    """x = match x { Some(v) => if v < 1000 { Some(v + 1) } else { None }, None => None }, n times."""
    lim = 999 if bug else 1000
    out = {}
    q = Q()
    q.decls.append(OPT_DT)
    x = q.const("x", "Opt")
    q.model = [("x", x)]
    q.asserts.append(f"(=> ((_ is Some) {x}) {rng(f'(Some.0 {x})')})")
    for i in range(n):
        nxt = q.const(f"x!{i + 1}", "Opt")
        v = f"(Some.0 {x})"
        q.asserts.append(f"(= {nxt} (ite ((_ is Some) {x}) (ite (< {v} 1000) (Some (+ {v} 1)) None) None))")
        x = nxt
    q.goal = f"(=> ((_ is Some) {x}) (<= (Some.0 {x}) {lim}))"
    out["adt"] = q
    q = Q()
    some = q.const("x.some", "Bool")
    v = q.const("x.v", "Int")
    q.model = [("x.some", some), ("x.v", v)]
    q.asserts.append(f"(=> {some} {rng(v)})")
    q.asserts.append(f"(=> (not {some}) (= {v} 0))")
    for _ in range(n):
        some2 = q.bind(f"(and {some} (< {v} 1000))", "Bool")
        v = q.bind(f"(ite {some2} (+ {v} 1) 0)", "Int")
        some = some2
    q.goal = f"(=> {some} (<= {v} {lim}))"
    out["flat"] = q
    return out


def e3_rebuild_eq(k, bug):
    """`match e { V_i(a, b, ..) => V_i(a, b, ..) }`, ensures r == e. Bug: swap a, b in 2+-field variants."""
    def args(fs):
        if bug and len(fs) >= 2:
            return [fs[1], fs[0]] + fs[2:]
        return fs

    out = {}
    q = Q()
    q.decls.append(enum_dt(k))
    e = q.const("e", "E")
    q.model = [("e", e)]
    arms = []
    for i in range(k):
        fs = [f"(|E::V{i}.{j}| {e})" for j in range(arity(i))]
        q.asserts.append(f"(=> ((_ is |E::V{i}|) {e}) {conj(rng(f) for f in fs)})")
        arms.append((f"((_ is |E::V{i}|) {e})", f"(|E::V{i}| {' '.join(args(fs))})"))
    r = arms[-1][1]
    for c, v in reversed(arms[:-1]):
        r = f"(ite {c} {v} {r})"
    q.goal = f"(= {r} {e})"
    out["adt"] = q
    q = Q()
    tag, fields = flat_enum(q, "e", k)
    q.model = [("e.tag", tag)] + [(f"e.V{i}.{j}", f) for i, fs in enumerate(fields) for j, f in enumerate(fs)]
    # r.tag = tag in every arm; r's fields: the arm's arguments when active, else 0.
    eqs = [f"(= {tag} {tag})"]
    for i, fs in enumerate(fields):
        for f, a in zip(fs, args(fs)):
            eqs.append(f"(= (ite (= {tag} {i}) {a} 0) {f})")
    q.goal = conj(eqs)
    out["flat"] = q
    return out


def e4_congruence(k):
    """Opaque `spec fn p(e: E) -> bool`. requires p(e1), e1 == e2; prove p(e2)."""
    out = {}
    q = Q()
    q.decls.append(enum_dt(k))
    q.decls.append("(declare-fun |p| (E) Bool)")
    e1, e2 = q.const("e1", "E"), q.const("e2", "E")
    q.model = [("e1", e1), ("e2", e2)]
    q.asserts += [f"(|p| {e1})", f"(= {e1} {e2})"]
    q.goal = f"(|p| {e2})"
    out["adt"] = q
    for enc, canonical in (("flat", True), ("flatj", False)):
        q = Q()
        sorts = " ".join(["Int"] + ["Int"] * sum(arity(i) for i in range(k)))
        q.decls.append(f"(declare-fun |p| ({sorts}) Bool)")
        t1, f1 = flat_enum(q, "e1", k, canonical)
        t2, f2 = flat_enum(q, "e2", k, canonical)
        flat1 = " ".join([t1] + [f for fs in f1 for f in fs])
        flat2 = " ".join([t2] + [f for fs in f2 for f in fs])
        q.model = [("e1.tag", t1), ("e2.tag", t2)] + [
            (f"e{n}.V{i}.{j}", f) for n, fl in ((1, f1), (2, f2)) for i, fs in enumerate(fl) for j, f in enumerate(fs)
        ]
        # Semantic equality: same tag and same active fields.
        same = [f"(= {t1} {t2})"] + [
            f"(=> (= {t1} {i}) {conj(f'(= {a} {b})' for a, b in zip(fs1, fs2))})"
            for i, (fs1, fs2) in enumerate(zip(f1, f2))
        ]
        q.asserts += [f"(|p| {flat1})", conj(same)]
        q.goal = f"(|p| {flat2})"
        out[enc] = q
    return out


def a1_array_invariant(k, bug):
    """Loop step `a[i] = x` keeps `forall j < i. a[j].f0 <= a[j].f1` (requires x.f0 <= x.f1)."""
    out = {}
    q = Q()
    q.decls.append(struct_dt(k))
    a = q.const("a", "(Array Int P)")
    ln, i, x = q.const("a.len", "Int"), q.const("i", "Int"), q.const("x", "P")
    q.asserts += [f"(and (<= 0 {ln}) (<= {ln} 1152921504606846975))", f"(and (<= 0 {i}) (< {i} {ln}))"]
    q.asserts += [rng(f"(P.f{j} {x})") for j in range(k)]
    if not bug:
        q.asserts.append(f"(<= (P.f0 {x}) (P.f1 {x}))")
    inv = lambda arr, hi: (
        f"(forall ((j Int)) (! (=> (and (<= 0 j) (< j {hi})) (<= (P.f0 (select {arr} j)) (P.f1 (select {arr} j)))) :pattern ((select {arr} j))))"
    )
    q.asserts.append(inv(a, i))
    q.goal = inv(f"(store {a} {i} {x})", f"(+ {i} 1)")
    q.model = [("i", i), ("a.len", ln), ("x", x), ("a[0]", f"(select {a} 0)")]
    out["adt"] = q
    q = Q()
    arrs = [q.const(f"a.f{j}", "(Array Int Int)") for j in range(k)]
    ln, i = q.const("a.len", "Int"), q.const("i", "Int")
    xs = [q.const(f"x.f{j}", "Int") for j in range(k)]
    q.asserts += [f"(and (<= 0 {ln}) (<= {ln} 1152921504606846975))", f"(and (<= 0 {i}) (< {i} {ln}))"]
    q.asserts += [rng(c) for c in xs]
    if not bug:
        q.asserts.append(f"(<= {xs[0]} {xs[1]})")
    inv = lambda f0, f1, hi: (
        f"(forall ((j Int)) (! (=> (and (<= 0 j) (< j {hi})) (<= (select {f0} j) (select {f1} j))) :pattern ((select {f0} j))))"
    )
    q.asserts.append(inv(arrs[0], arrs[1], i))
    st = [f"(store {arrs[j]} {i} {xs[j]})" for j in range(k)]
    q.goal = inv(st[0], st[1], f"(+ {i} 1)")
    q.model = [("i", i), ("a.len", ln)] + [(f"x.f{j}", c) for j, c in enumerate(xs)] + [
        (f"a[0].f{j}", f"(select {arrs[j]} 0)") for j in range(k)
    ]
    out["flat"] = q
    return out


def a2_field_write_frame(k, bug):
    """`a[i].f0 = v`; ensures other elements and a[i]'s other fields unchanged. Bug: claims a[i] unchanged too."""
    out = {}
    q = Q()
    q.decls.append(struct_dt(k))
    a = q.const("a", "(Array Int P)")
    ln, i, v = q.const("a.len", "Int"), q.const("i", "Int"), q.const("v", "Int")
    q.asserts += [f"(and (<= 0 {ln}) (<= {ln} 1152921504606846975))", f"(and (<= 0 {i}) (< {i} {ln}))", rng(v)]
    old = f"(select {a} {i})"
    new = f"(store {a} {i} (P {v} {' '.join(f'(P.f{j} {old})' for j in range(1, k))}))"
    a2 = q.bind(new, "(Array Int P)")
    excl = "" if bug else f" (not (= j {i}))"
    frame = f"(forall ((j Int)) (! (=> (and (<= 0 j) (< j {ln}){excl}) (= (select {a2} j) (select {a} j))) :pattern ((select {a2} j))))"
    rest = conj(f"(= (P.f{j} (select {a2} {i})) (P.f{j} {old}))" for j in range(1, k))
    q.goal = conj([frame, rest])
    q.model = [("i", i), ("a.len", ln), ("v", v), (f"a[0]", f"(select {a} 0)")]
    out["adt"] = q
    q = Q()
    arrs = [q.const(f"a.f{j}", "(Array Int Int)") for j in range(k)]
    ln, i, v = q.const("a.len", "Int"), q.const("i", "Int"), q.const("v", "Int")
    q.asserts += [f"(and (<= 0 {ln}) (<= {ln} 1152921504606846975))", f"(and (<= 0 {i}) (< {i} {ln}))", rng(v)]
    new = [f"(store {arrs[0]} {i} {v})"] + arrs[1:]
    frame = "(forall ((j Int)) (! (=> (and (<= 0 j) (< j {ln}){excl}) {eq}) :pattern ((select {a0} j))))".format(
        ln=ln, excl=excl, eq=conj(f"(= (select {new[j]} j) (select {arrs[j]} j))" for j in range(k)), a0=new[0]
    )
    rest = conj(f"(= (select {new[j]} {i}) (select {arrs[j]} {i}))" for j in range(1, k))
    q.goal = conj([frame, rest])
    q.model = [("i", i), ("a.len", ln), ("v", v)] + [(f"a[0].f{j}", f"(select {arrs[j]} 0)") for j in range(k)]
    out["flat"] = q
    return out


def a3_option_array(bug):
    """Loop step `a[i] = if x >= 0 { Some(x) } else { None }` keeps `forall j < i. a[j] is Some(v) ==> v >= 0`.
    Bug: `Some(x - 1)` (and x - 1 may be negative)."""
    val = f"(- |x| 1)" if bug else "|x|"
    out = {}
    q = Q()
    q.decls.append(OPT_DT)
    a = q.const("a", "(Array Int Opt)")
    ln, i, x = q.const("a.len", "Int"), q.const("i", "Int"), q.const("x", "Int")
    q.asserts += [f"(and (<= 0 {ln}) (<= {ln} 1152921504606846975))", f"(and (<= 0 {i}) (< {i} {ln}))", rng(x)]
    q.asserts.append(f"(forall ((j Int)) (! (=> ((_ is Some) (select {a} j)) {rng('(Some.0 (select %s j))' % a)}) :pattern ((select {a} j))))")
    inv = lambda arr, hi: (
        f"(forall ((j Int)) (! (=> (and (<= 0 j) (< j {hi}) ((_ is Some) (select {arr} j))) (>= (Some.0 (select {arr} j)) 0)) :pattern ((select {arr} j))))"
    )
    q.asserts.append(inv(a, i))
    q.goal = inv(f"(store {a} {i} (ite (>= {x} 0) (Some {val}) None))", f"(+ {i} 1)")
    q.model = [("i", i), ("a.len", ln), ("x", x), ("a[0]", f"(select {a} 0)")]
    out["adt"] = q
    q = Q()
    some, vs = q.const("a.some", "(Array Int Bool)"), q.const("a.v", "(Array Int Int)")
    ln, i, x = q.const("a.len", "Int"), q.const("i", "Int"), q.const("x", "Int")
    q.asserts += [f"(and (<= 0 {ln}) (<= {ln} 1152921504606846975))", f"(and (<= 0 {i}) (< {i} {ln}))", rng(x)]
    # Element facts of an array of enums: range of active payloads, canonical zero of inactive ones.
    q.asserts.append(f"(forall ((j Int)) (! (and (=> (select {some} j) {rng('(select %s j)' % vs)}) (=> (not (select {some} j)) (= (select {vs} j) 0))) :pattern ((select {vs} j))))")
    inv = lambda s, v, hi: (
        f"(forall ((j Int)) (! (=> (and (<= 0 j) (< j {hi}) (select {s} j)) (>= (select {v} j) 0)) :pattern ((select {v} j))))"
    )
    q.asserts.append(inv(some, vs, i))
    c = f"(>= {x} 0)"
    q.goal = inv(f"(store {some} {i} {c})", f"(store {vs} {i} (ite {c} {val} 0))", f"(+ {i} 1)")
    q.model = [("i", i), ("a.len", ln), ("x", x), ("a[0].some", f"(select {some} 0)"), ("a[0].v", f"(select {vs} 0)")]
    out["flat"] = q
    return out


def instances():
    """(family, size, variant, {enc: Q}) in a fixed order."""
    for k in (4, 16):
        for n in (8, 32, 128):
            for bug in (False, True):
                yield "S1_update_chain", f"k={k},n={n}", "bug" if bug else "proof", s1_update_chain(k, n, bug)
    for k in (2, 8, 32):
        for bug in (False, True):
            yield "E1_match", f"k={k}", "bug" if bug else "proof", e1_match(k, bug)
    for n in (8, 32, 128):
        for bug in (False, True):
            yield "E2_option_chain", f"n={n}", "bug" if bug else "proof", e2_option_chain(n, bug)
    for k in (2, 8, 32):
        for bug in (False, True):
            yield "E3_rebuild_eq", f"k={k}", "bug" if bug else "proof", e3_rebuild_eq(k, bug)
    for k in (2, 8, 32):
        yield "E4_congruence", f"k={k}", "proof", e4_congruence(k)
    for k in (2, 8, 32):
        for bug in (False, True):
            yield "A1_array_invariant", f"k={k}", "bug" if bug else "proof", a1_array_invariant(k, bug)
    for k in (2, 8, 32):
        for bug in (False, True):
            yield "A2_field_write_frame", f"k={k}", "bug" if bug else "proof", a2_field_write_frame(k, bug)
    for bug in (False, True):
        yield "A3_option_array", "-", "bug" if bug else "proof", a3_option_array(bug)


# ---------------------------------------------------------------- running

def z3_bin():
    return os.environ.get("Z3", "z3")


def run(text):
    p = subprocess.run([z3_bin(), "-smt2", "-in"], input=text, capture_output=True, text=True, timeout=600)
    out = p.stdout
    lines = out.split("\n")
    answer = lines[0].strip() if lines else "error"
    m = re.search(r"\(:rlimit (\d+)\)", out)
    rl = int(m.group(1)) if m else -1
    model = ""
    if m:
        model = " ".join(out[m.end():].split())
        if model.startswith("(error"):
            model = ""
    return answer, rl, model


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--emit", metavar="DIR")
    args = ap.parse_args()
    if args.emit:
        os.makedirs(args.emit, exist_ok=True)
        for fam, size, var, qs in instances():
            for enc, q in qs.items():
                name = f"{fam}.{size.replace(',', '_').replace('=', '')}.{var}.{enc}.smt2"
                with open(os.path.join(args.emit, name), "w") as f:
                    f.write(q.text(0))
        return
    ver = subprocess.run([z3_bin(), "--version"], capture_output=True, text=True).stdout.strip()
    rows = []
    for fam, size, var, qs in instances():
        for enc, q in qs.items():
            per_seed = {}
            for seed in SEEDS + ["0r"]:
                s = 0 if seed == "0r" else seed
                per_seed[seed] = run(q.text(s))
            base = per_seed[0]
            repeat_same = per_seed["0r"] == base
            for seed in SEEDS:
                answer, rl, model = per_seed[seed]
                rows.append({
                    "family": fam, "size": size, "variant": var, "encoding": enc, "seed": seed,
                    "expected": "sat" if var == "bug" else "unsat", "answer": answer, "rlimit": rl,
                    "query_bytes": len(q.text(seed).encode()), "model_terms": len(q.model) if answer == "sat" else 0,
                    "model_chars": len(model) if answer == "sat" else 0,
                    "repeat_same": repeat_same if seed == 0 else "",
                    "model": model if answer == "sat" else "",
                })
            print(f"{fam:22} {size:10} {var:5} {enc:6} {base[0]:7} {base[1]}", file=sys.stderr)
    fields = list(rows[0].keys())
    with open(os.path.join(HERE, "results.csv"), "w", newline="") as f:
        w = csv.DictWriter(f, fieldnames=fields, lineterminator="\n")
        w.writeheader()
        w.writerows(rows)
    write_md(rows, ver)


def write_md(rows, ver):
    groups = {}
    for r in rows:
        groups.setdefault((r["family"], r["size"], r["variant"], r["encoding"]), []).append(r)
    out = [
        "# 2B-02 결과",
        "",
        f"생성: `python3 dev-docs/experiments/2B-02/run.py` · {ver} · rlimit {RLIMIT} · 시드 {SEEDS}(+ 시드 0 반복)",
        "",
        "판정은 시드 0·1·2의 답(모두 같으면 하나만). rlimit은 시드별 값(최소~최대). 반복은 시드 0을 다시 돌려 답·rlimit·모델이 같은지.",
        "",
        "| 묶음 | 크기 | 변형 | 인코딩 | 기대 | 판정 | rlimit(최소~최대) | 질의 바이트 | 반복 같음 | 모델 항 | 모델 글자 |",
        "| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |",
    ]
    for (fam, size, var, enc), rs in groups.items():
        answers = sorted({r["answer"] for r in rs})
        rls = [r["rlimit"] for r in rs]
        ans = answers[0] if len(answers) == 1 else "/".join(answers) + " (시드에 따라 다름)"
        mark = "" if answers == [rs[0]["expected"]] else " ⚠"
        out.append(
            f"| {fam} | {size} | {var} | {enc} | {rs[0]['expected']} | {ans}{mark} | {min(rls)}~{max(rls)} | "
            f"{rs[0]['query_bytes']} | {rs[0]['repeat_same']} | {rs[0]['model_terms']} | {rs[0]['model_chars']} |"
        )
    out += ["", "## 반례 예(시드 0, z3 `get-value` 원문)", ""]
    shown = set()
    for r in rows:
        key = (r["family"], r["encoding"])
        if r["seed"] == 0 and r["answer"] == "sat" and key not in shown and r["size"] in ("k=2", "k=4,n=8", "n=8", "-"):
            shown.add(key)
            out.append(f"- {r['family']} {r['size']} {r['variant']} `{r['encoding']}`: `{r['model']}`")
    with open(os.path.join(HERE, "results.md"), "w") as f:
        f.write("\n".join(out) + "\n")


if __name__ == "__main__":
    main()
