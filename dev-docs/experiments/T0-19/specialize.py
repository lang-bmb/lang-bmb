#!/usr/bin/env python3
"""Prototype of the proposed prover fix (backlog T0-24), on query text.

  specialize.py SCRIPT.smt2 N     SCRIPT from `bmb verify --emit-smt`, N an obligation number

Prints z3's answer, rlimit and reason for three queries of obligation N:
the first query as `bmb` sends it, its ground-instance retry (ADR-0023) as it
is, and the retry plus a specialized instance of the definition for each
application of a non-recursive spec fn that occurs under a quantifier of the
retry query (where the retry has no instance for it): every argument that does
not mention a bound variable is substituted, the other parameters stay
quantified with the definition's guard and the application as pattern. Every
such axiom is an instance of the definition, so `unsat` there is a proof.
Recursive definitions are left out (they keep the ground instances to depth 3).
Needs `z3` on PATH. Standard library only.
"""
import re
import subprocess
import sys


def parse(s):
    toks = re.findall(r'\(|\)|\|[^|]*\||"[^"]*"|[^\s()]+', s)

    def rd(i):
        if toks[i] == "(":
            out, i = [], i + 1
            while toks[i] != ")":
                x, i = rd(i)
                out.append(x)
            return out, i + 1
        return toks[i], i + 1
    res, i = [], 0
    while i < len(toks):
        x, i = rd(i)
        res.append(x)
    return res


def pr(e):
    return e if isinstance(e, str) else "(" + " ".join(pr(x) for x in e) + ")"


def atoms(e, acc):
    if isinstance(e, str):
        acc.add(e)
    else:
        for x in e:
            atoms(x, acc)
    return acc


def z3(q):
    r = subprocess.run(["z3", "-in", "-st"], input=q + "\n(get-info :reason-unknown)\n",
                       capture_output=True, text=True)
    rl = re.search(r":rlimit-count\s+(\d+)", r.stdout)
    why = re.search(r':reason-unknown "([^"]*)"', r.stdout)
    return r.stdout.split("\n")[0], int(rl.group(1)) if rl else None, why.group(1) if why else ""


def main():
    script, num = open(sys.argv[1]).read(), sys.argv[2]
    parts = re.split(r"\n(?=; )", script)
    first = next(p for p in parts if p.startswith(f"; obligation {num}:"))
    retry = next((p for p in parts if p.startswith(f"; retry of obligation {num} ")), None)
    print("first query     ", z3(first))
    if retry is None:
        print("(no retry: the first query decided)")
        return
    print("retry as is     ", z3(retry))
    # Quantified definitions `(forall (ps) (! (=> guard (= (f ps) body)) ..))`
    # of the first query that are not recursive.
    defs = {}
    for c in parse(first.split("\n", 1)[1]):
        if c[0] != "assert" or not isinstance(c[1], list) or c[1][0] != "forall":
            continue
        binders, bang = c[1][1], c[1][2]
        if bang[0] != "!":
            continue
        body = bang[1]
        if body[0] != "=>" or not isinstance(body[2], list) or body[2][0] != "=" \
                or not isinstance(body[2][1], list):
            continue
        app = body[2][1]
        if app[1:] == [b[0] for b in binders] and app[0] not in atoms(body[2][2], set()):
            defs[app[0]] = (binders, body)
    # Applications under a quantifier, with the variables bound there.
    found = []

    def walk(e, bound):
        if isinstance(e, str):
            return
        if e and e[0] in ("forall", "exists") and len(e) == 3:
            walk(e[2], bound | {b[0] for b in e[1]})
            return
        if e and isinstance(e[0], str) and e[0] in defs and bound \
                and len(e) - 1 == len(defs[e[0]][0]):
            found.append((e, bound))
        for x in e:
            walk(x, bound)
    for c in parse(retry.split("\n", 1)[1]):
        walk(c, frozenset())
    extra = []
    for e, bound in found:
        binders, body = defs[e[0]]
        sub = {b[0]: a for b, a in zip(binders, e[1:]) if not (atoms(a, set()) & bound)}
        rest = [b for b in binders if b[0] not in sub]

        def s(x):
            return sub.get(x, x) if isinstance(x, str) else [s(y) for y in x]
        app = s([e[0]] + [b[0] for b in binders])
        ax = ("(assert (forall (%s) (! %s :pattern (%s))))" % (" ".join(pr(b) for b in rest), pr(s(body)), pr(app))
              if rest else "(assert %s)" % pr(s(body)))
        if ax not in extra:
            extra.append(ax)
    lines = retry.rsplit("(check-sat)", 1)[0].split("\n")
    at = max(i for i, x in enumerate(lines) if x.startswith("(declare-const")) + 1
    lines[at:at] = extra
    print("retry+specialized", z3("\n".join(lines) + "(check-sat)"), f"{len(extra)} specialized")


if __name__ == "__main__":
    main()
