#!/usr/bin/env python3
"""T2-02: SMT encodings of `f64`, compared on query families shaped like the
obligations n-body, spectral-norm and mandelbrot would produce (ADR-0061 §5).

  run.py                 generate every query, run z3 with seeds 0,1,2 (+ seed 0
                         again for the repeat check), write results.csv and results.md
  run.py --emit DIR      only write the generated .smt2 files (seed 0) to DIR
  run.py --only FAM,..   rerun those families and replace their rows in results.csv

Encodings (one query = one proof obligation, as BMB sends it: ADR-0018). In
both, an `f64` value has the sort `(_ FloatingPoint 11 53)`; a `Vec<f64>` is
`(Array Int F64)`.
  bits     every operation interpreted: `fp.add RNE`, `fp.lt`, `fp.sqrt RNE`,
           `(_ to_fp 11 53) RNE` (ADR-0061 candidate (a)).
  opaque   every operation an uninterpreted function or predicate over the sort
           (`f64.add`, `f64.lt`, ...), as ADR-0061 §5 proposes for ordinary
           obligations (candidate (b) outside `by (bit_vector)`).

Families (the goal is what the obligation states; `bug` variants are false,
so sat with a counterexample is expected):
  SN  spectral-norm `eval_a_times_u` row, unrolled k columns: the matrix entry
      1/f64::from((i+j)(i+j+1)/2 + i + 1) times u[j] accumulated into a sum.
      Goal: the integer obligations of the row (index in bounds, the exactness
      obligation |x| <= 2^53 of f64::from, E0815), with i, j < n <= 5500.
  NB  n-body `advance`, k body pairs (10 pairs = one step of the 5 bodies):
      dx, dy, dz, d2, mag = dt / (d2 * sqrt(d2)), six velocity updates, over
      field arrays. Goal: every index in bounds.
  MB  mandelbrot inner loop unrolled k iterations: z = z*z + c, a branch on
      |z|^2 > 4.0 that leaves the loop. Goal: the integer counter stays < 50.
  FP  facts about the floating-point values themselves, which only `bits` can
      prove: sqrt of a non-negative is non-negative or NaN (FP1), and a sum of
      k products of bounded finite values is finite (FPk). In `opaque` these
      are expected to be unprovable (sat); ADR-0061 puts them in
      `by (bit_vector)`.

Same solver options as compiler/src/verify.rs `header()`: produce-models,
smt.random_seed/sat.random_seed = seed, rlimit 5000000. Standard library only.
Needs the pinned z3 on PATH (or Z3=path).
"""
import argparse
import csv
import os
import re
import struct
import subprocess
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
RLIMIT = 5000000
SEEDS = [0, 1, 2]
F = "(_ FloatingPoint 11 53)"
ENCODINGS = ["bits", "opaque"]


class Q:
    def __init__(self, enc):
        self.enc = enc
        self.decls, self.asserts, self.goal = [], [], "true"
        self.n = 0
        if enc == "opaque":
            self.decls += [
                f"(declare-fun f64.add ({F} {F}) {F})", f"(declare-fun f64.sub ({F} {F}) {F})",
                f"(declare-fun f64.mul ({F} {F}) {F})", f"(declare-fun f64.div ({F} {F}) {F})",
                f"(declare-fun f64.sqrt ({F}) {F})", f"(declare-fun f64.from (Int) {F})",
                f"(declare-fun f64.lt ({F} {F}) Bool)", f"(declare-fun f64.le ({F} {F}) Bool)",
                f"(declare-fun f64.is_nan ({F}) Bool)", f"(declare-fun f64.is_finite ({F}) Bool)",
            ]

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

    # f64 operations in the chosen encoding
    def op(self, name, *args):
        if self.enc == "opaque":
            return f"(f64.{name} {' '.join(args)})"
        rne = {"add": "fp.add RNE", "sub": "fp.sub RNE", "mul": "fp.mul RNE", "div": "fp.div RNE",
               "sqrt": "fp.sqrt RNE", "lt": "fp.lt", "le": "fp.leq", "is_nan": "fp.isNaN"}
        if name == "from":
            return f"((_ to_fp 11 53) RNE (to_real {args[0]}))"
        if name == "is_finite":
            return f"(not (or (fp.isInfinite {args[0]}) (fp.isNaN {args[0]})))"
        return f"({rne[name]} {' '.join(args)})"

    def lit(self, x):
        # A literal is its binary64 bit pattern in both encodings (the value is known).
        b = struct.unpack("<Q", struct.pack("<d", float(x)))[0]
        return f"(fp #b{b >> 63:01b} #b{(b >> 52) & 0x7ff:011b} #b{b & ((1 << 52) - 1):052b})"

    def text(self, seed):
        lines = ["(set-option :produce-models true)", f"(set-option :smt.random_seed {seed})",
                 f"(set-option :sat.random_seed {seed})", f"(set-option :rlimit {RLIMIT})"]
        lines += self.decls
        lines += [f"(assert {a})" for a in self.asserts]
        lines += [f"(assert (not {self.goal}))", "(check-sat)", "(get-info :rlimit)"]
        return "\n".join(lines) + "\n"


def conj(xs):
    xs = list(xs)
    return "true" if not xs else xs[0] if len(xs) == 1 else f"(and {' '.join(xs)})"


def sn(enc, k, bug):
    """spectral-norm row i, columns j0..j0+k-1, u: Vec<f64> of length n."""
    q = Q(enc)
    n, i, j0 = q.const("n", "Int"), q.const("i", "Int"), q.const("j0", "Int")
    u = q.const("u", f"(Array Int {F})")
    q.asserts += [f"(<= 0 {n})", f"(<= {n} 5500)", f"(<= 0 {i})", f"(< {i} {n})", f"(<= 0 {j0})",
                  f"(<= (+ {j0} {k}) {n})"]
    total = q.const("sum0", F)
    goals = []
    for c in range(k):
        j = f"(+ {j0} {c})"
        d = f"(+ (div (* (+ {i} {j}) (+ {i} {j} 1)) 2) {i} 1)"
        goals.append(f"(<= (- 9007199254740992) {d})")
        goals.append(f"(<= {d} 9007199254740992)")
        bound = f"(<= {j} {n})" if bug and c == k - 1 else f"(< {j} {n})"
        goals.append(f"(and (<= 0 {j}) {bound})" if not bug or c != k - 1 else f"(and (<= 0 {j}) (< (+ {j} 1) {n}))")
        a = q.op("div", q.lit(1.0), q.op("from", d))
        total = q.bind(q.op("add", total, q.op("mul", a, f"(select {u} {j})")), F)
    out = q.const("out", F)
    q.asserts.append(f"(= {out} {total})")
    q.goal = conj(goals)
    return q


def nb(enc, k, bug):
    """n-body: k body pairs (i, j) of 5 bodies, field arrays x..vz, mass."""
    q = Q(enc)
    arr = {f: q.const(f, f"(Array Int {F})") for f in ("x", "y", "z", "vx", "vy", "vz", "m")}
    dt = q.lit(0.01)
    pairs = [(i, j) for i in range(5) for j in range(i + 1, 5)]
    goals = []
    for p in range(k):
        i, j = pairs[p % len(pairs)]
        iv, jv = q.const(f"i{p}", "Int"), q.const(f"j{p}", "Int")
        q.asserts += [f"(= {iv} {i})", f"(= {jv} {j})"]
        # The bug variant claims the second index is below itself (false).
        hi = j if bug and p == k - 1 else 5
        goals += [f"(and (<= 0 {iv}) (< {iv} 5))", f"(and (<= 0 {jv}) (< {jv} {hi}))"]
        sel = lambda f, v: f"(select {arr[f]} {v})"  # noqa: E731
        dx = q.bind(q.op("sub", sel("x", iv), sel("x", jv)), F)
        dy = q.bind(q.op("sub", sel("y", iv), sel("y", jv)), F)
        dz = q.bind(q.op("sub", sel("z", iv), sel("z", jv)), F)
        d2 = q.bind(q.op("add", q.op("add", q.op("mul", dx, dx), q.op("mul", dy, dy)), q.op("mul", dz, dz)), F)
        mag = q.bind(q.op("div", dt, q.op("mul", d2, q.op("sqrt", d2))), F)
        for f, d in (("vx", dx), ("vy", dy), ("vz", dz)):
            wi = q.bind(q.op("sub", sel(f, iv), q.op("mul", q.op("mul", d, sel("m", jv)), mag)), F)
            wj = q.bind(q.op("add", sel(f, jv), q.op("mul", q.op("mul", d, sel("m", iv)), mag)), F)
            new = q.const(f"{f}{p}", f"(Array Int {F})")
            q.asserts.append(f"(= {new} (store (store {arr[f]} {iv} {wi}) {jv} {wj}))")
            arr[f] = new
    q.goal = conj(goals)
    return q


def mb(enc, k, bug):
    """mandelbrot: k iterations of z = z^2 + c, leaving when |z|^2 > 4."""
    q = Q(enc)
    cr, ci = q.const("cr", F), q.const("ci", F)
    zr, zi = q.lit(0.0), q.lit(0.0)
    it = q.const("it0", "Int")
    q.asserts.append(f"(= {it} 0)")
    four = q.lit(4.0)
    live = "true"
    goals = []
    limit = 50
    for s in range(k):
        r2 = q.bind(q.op("mul", zr, zr), F)
        i2 = q.bind(q.op("mul", zi, zi), F)
        esc = q.op("lt", four, q.op("add", r2, i2))
        live = q.bind(f"(and {live} (not {esc}))", "Bool")
        nzi = q.bind(q.op("add", q.op("mul", q.op("mul", q.lit(2.0), zr), zi), ci), F)
        nzr = q.bind(q.op("add", q.op("sub", r2, i2), cr), F)
        zr, zi = nzr, nzi
        nit = q.const(f"it{s + 1}", "Int")
        q.asserts.append(f"(= {nit} (ite {live} (+ {it} 1) {it}))")
        it = nit
        bound = limit if not (bug and s == k - 1) else s + 1
        goals.append(f"(< {it} {bound})")
    q.goal = conj(goals)
    return q


def fp1(enc):
    q = Q(enc)
    x = q.const("x", F)
    zero = q.lit(0.0)
    r = q.op("sqrt", x)
    q.goal = f"(=> {q.op('le', zero, x)} (or {q.op('is_nan', r)} {q.op('le', zero, r)}))"
    return q


def fpk(enc, k, bug):
    """sum of k products a_i*b_i with |a_i|, |b_i| <= B is finite."""
    q = Q(enc)
    bound = q.lit(1.0e150)
    nb_ = q.lit(-1.0e150)
    acc = q.lit(0.0)
    for t in range(k):
        a, b = q.const(f"a{t}", F), q.const(f"b{t}", F)
        for v in (a, b):
            q.asserts.append(f"{q.op('is_finite', v)}")
            q.asserts.append(f"{q.op('le', nb_, v)}")
            q.asserts.append(f"{q.op('le', v, bound)}")
        acc = q.bind(q.op("add", acc, q.op("mul", a, b)), F)
    if bug:  # a product of two values up to 1e200 can overflow
        a, b = q.const("ua", F), q.const("ub", F)
        q.asserts += [q.op("is_finite", a), q.op("is_finite", b), q.op("le", a, q.lit(1.0e200)),
                      q.op("le", q.lit(-1.0e200), a), q.op("le", b, q.lit(1.0e200)), q.op("le", q.lit(-1.0e200), b)]
        acc = q.op("add", acc, q.op("mul", a, b))
    q.goal = q.op("is_finite", acc)
    return q


def instances():
    out = []
    for k in (1, 4, 16, 64):
        for bug in (False, True):
            out.append(("SN", f"k={k}", bug, lambda e, k=k, b=bug: sn(e, k, b), "int"))
    for k in (1, 4, 10, 40):
        for bug in (False, True):
            out.append(("NB", f"k={k}", bug, lambda e, k=k, b=bug: nb(e, k, b), "int"))
    for k in (1, 4, 16):
        for bug in (False, True):
            out.append(("MB", f"k={k}", bug, lambda e, k=k, b=bug: mb(e, k, b), "int"))
    out.append(("FP1", "-", False, fp1, "fp"))
    for k in (1, 2, 4, 8):
        for bug in (False, True):
            out.append(("FPk", f"k={k}", bug, lambda e, k=k, b=bug: fpk(e, k, b), "fp"))
    return out


def expected(kind, enc, bug):
    if bug:
        return "sat"
    if kind == "fp" and enc == "opaque":
        return "sat (증명 불가)"
    return "unsat"


def z3_bin():
    return os.environ.get("Z3", "z3")


def run(text):
    p = subprocess.run([z3_bin(), "-smt2", "-in"], input=text, capture_output=True, text=True, timeout=900)
    out = p.stdout
    answer = out.split("\n")[0].strip() if out else "error"
    m = re.search(r"\(:rlimit (\d+)\)", out)
    return answer, int(m.group(1)) if m else -1


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--emit")
    ap.add_argument("--only")
    args = ap.parse_args()
    only = set(args.only.split(",")) if args.only else None
    if args.emit:
        os.makedirs(args.emit, exist_ok=True)
        for fam, size, bug, mk, _ in instances():
            for e in ENCODINGS:
                name = f"{fam}_{size.replace('=', '')}_{'bug' if bug else 'proof'}_{e}.smt2"
                open(os.path.join(args.emit, name), "w").write(mk(e).text(0))
        return
    ver = subprocess.run([z3_bin(), "--version"], capture_output=True, text=True).stdout.strip()
    old = []
    if only:
        with open(os.path.join(HERE, "results.csv")) as f:
            old = list(csv.DictReader(f))
    rows = []
    for fam, size, bug, mk, kind in instances():
        if only and fam not in only:
            rows += [r for r in old if r["family"] == fam and r["size"] == size
                     and r["variant"] == ("bug" if bug else "proof")]
            continue
        for e in ENCODINGS:
            q = mk(e)
            res = [run(q.text(s)) for s in SEEDS]
            rep = run(q.text(0))
            answers = sorted({a for a, _ in res})
            rows.append({
                "family": fam, "size": size, "variant": "bug" if bug else "proof", "encoding": e,
                "expected": expected(kind, e, bug), "answer": "/".join(answers),
                "rlimit_min": min(r for _, r in res), "rlimit_max": max(r for _, r in res),
                "bytes": len(q.text(0)), "repeat_same": rep == res[0],
            })
            print(rows[-1], file=sys.stderr, flush=True)
    with open(os.path.join(HERE, "results.csv"), "w", newline="") as f:
        w = csv.DictWriter(f, fieldnames=list(rows[0]))
        w.writeheader()
        w.writerows(rows)
    lines = [
        "# T2-02 결과",
        "",
        f"생성: `python3 dev-docs/experiments/T2-02/run.py` · {ver} · rlimit {RLIMIT} · 시드 {SEEDS}(+ 시드 0 반복)",
        "",
        "판정은 시드 0·1·2의 답(모두 같으면 하나만). rlimit은 시드별 값(최소~최대). `unknown`은 한도 안에 결정하지 못한 것이다.",
        "",
        "| 묶음 | 크기 | 변형 | 인코딩 | 기대 | 판정 | rlimit(최소~최대) | 질의 바이트 | 반복 같음 |",
        "| --- | --- | --- | --- | --- | --- | --- | --- | --- |",
    ]
    for r in rows:
        lines.append(f"| {r['family']} | {r['size']} | {r['variant']} | {r['encoding']} | {r['expected']} | {r['answer']} | "
                     f"{r['rlimit_min']}~{r['rlimit_max']} | {r['bytes']} | {r['repeat_same']} |")
    with open(os.path.join(HERE, "results.md"), "w") as f:
        f.write("\n".join(lines) + "\n")


if __name__ == "__main__":
    main()
