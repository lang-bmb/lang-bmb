#!/usr/bin/env python3
"""Contract clauses as inline expressions (backlog 2R-04b).

Deterministic, offline. BMB requires every `requires`/`ensures` clause to be
a call of a named `spec fn` (E0501); Verus writes the expression inline.
2R-02 (syntax-eval-1.md §2.3) counted the single-use predicates. This
transforms them: each `spec fn` named by exactly one contract clause and by
nothing else (no other spec body, invariant or assert) is inlined into that
clause, its parameters replaced by the call's arguments (each in
parentheses), and the result is measured against the written form:

  tokens        the lexical tokens of the corpus (measure.py's lexer)
  L1 bytes      the map's L1 rows (`bmb map L1`): the predicate's row goes,
                the clause's text becomes the expression
  contracts     `bmb query contracts`' answer for the clause: a name now,
                an expression inline
  quantifiers   how many of the inlined predicates hold a quantifier
                (variant T2 of 2R-02: only quantifier-free clauses inline)
  leaves        how many are leaves (call no other spec fn): their query is
                the same either way, since a leaf spec fn is a macro in the
                query (ADR-0067)

The inline form does not check (E0501), so its proof cost and the BSB
verdicts cannot be measured without the compiler change; the leaf count is
the evidence for the cost.

Usage: bench/syntax/inline_contracts.py BMB [--write FILE]
"""

import glob
import json
import os
import re
import subprocess
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import measure  # noqa: E402

HEADER = re.compile(r"spec fn (\w+)\s*\(([^)]*)\)\s*->\s*bool\s*\{")


def params_of(texts):
    """`unit::name` -> parameter names, from the source text."""
    out = {}
    for text in texts:
        for unit, toks in measure.units_of(text):
            src = " ".join(toks)
            for m in HEADER.finditer(src):
                ps = [p.split(":")[0].strip() for p in m.group(2).split(",") if ":" in p]
                out[unit + "::" + m.group(1)] = ps
    return out


def calls_in(clause, name):
    """The argument token lists of each call `name(..)` in a clause."""
    out = []
    i = 0
    while i < len(clause):
        if clause[i] == name and i + 1 < len(clause) and clause[i + 1] == "(":
            k = measure.match_close(clause, i + 1, "(", ")")
            args, depth, cur = [], 0, []
            for t in clause[i + 2 : k]:
                if t in "([{":
                    depth += 1
                elif t in ")]}":
                    depth -= 1
                if t == "," and depth == 0:
                    args.append(cur)
                    cur = []
                else:
                    cur.append(t)
            if cur:
                args.append(cur)
            out.append((i, k, args))
            i = k + 1
        else:
            i += 1
    return out


def analyse(label, paths, bmb):
    texts = [open(p, encoding="utf-8").read() for p in paths]
    _, items, _ = measure.tally([(None, t) for t in texts])
    specs = {i.name: i for i in items if i.kind == "spec"}
    names = set(specs)
    params = params_of(texts)
    where = {n: set() for n in names}
    refs = {n: [] for n in names}
    for it in items:
        u = it.name.split("::")[0]
        if it.kind == "fn":
            for c in ("requires", "ensures"):
                for m in measure.mentioned(it.clauses[c], u, names):
                    where[m].add("contract")
                    refs[m].append((it, c))
            for m in measure.mentioned(it.inv, u, names):
                where[m].add("inv")
        else:
            for m in measure.mentioned(it.body, u, names):
                where[m].add("spec")
    single = [n for n in sorted(names) if where[n] == {"contract"} and len(refs[n]) == 1 and n in params]
    saved = 0
    quant = 0
    leaves = 0
    inline_texts = {}
    for n in single:
        sp = specs[n]
        it, c = refs[n][0]
        short = n.split("::", 1)[1]
        calls = calls_in(it.clauses[c], short) or calls_in(it.clauses[c], n.split("::")[-1])
        if not calls:
            continue
        _, _, args = calls[0]
        ps = params[n]
        body = list(sp.body)
        sub = []
        for t in body:
            if t in ps and ps.index(t) < len(args):
                sub += ["("] + args[ps.index(t)] + [")"]
            else:
                sub.append(t)
        expr = ["("] + sub + [")"]
        call_len = 2 + sum(len(a) for a in args) + max(0, len(args) - 1) + 1
        saved += sp.total + call_len - len(expr)
        if "forall" in body or "exists" in body:
            quant += 1
        if not measure.mentioned(body, n.split("::")[0], names):
            leaves += 1
        inline_texts[(it.name, short)] = " ".join(sub)
    # L1 bytes: the written rows, the predicate rows removed, the clause text replaced
    m = subprocess.run([bmb, "map", "L1", *paths], capture_output=True, text=True).stdout.splitlines()
    rows = [json.loads(x) for x in m if x.startswith('{"level"')]
    l1 = sum(len(x) + 1 for x in m if x.startswith('{"level"'))
    l1_inline = 0
    single_set = set(single)
    for r in rows:
        if r["path"] in single_set:
            continue
        line = json.dumps(r, ensure_ascii=False, separators=(",", ":"))
        for c in ("requires", "ensures"):
            for k, cl in enumerate(r.get(c) or []):
                callee = cl.split("(")[0]
                key = (r["path"], callee.split("::")[-1])
                if key in inline_texts and r["path"].split("::")[0] + "::" + callee.split("::")[-1] in single_set:
                    line = line.replace(json.dumps(cl, ensure_ascii=False), json.dumps(inline_texts[key], ensure_ascii=False), 1)
        l1_inline += len(line) + 1
    toks = sum(len(measure.lex(t)) for t in texts)
    return {
        "corpus": label,
        "spec_fns": len(names),
        "single_use_contract_only": len(single),
        "with_quantifier": quant,
        "leaves": leaves,
        "tokens": toks,
        "tokens_inline": toks - saved,
        "l1_bytes": l1,
        "l1_bytes_inline": l1_inline,
    }


def main():
    bmb = sys.argv[1]
    write = sys.argv[sys.argv.index("--write") + 1] if "--write" in sys.argv else None
    corp = [
        ("m1b seed", sorted(glob.glob(os.path.join(ROOT, "bench/m1b/seed/src/*.bmb")))),
        ("m1 seed", sorted(glob.glob(os.path.join(ROOT, "bench/m1/seed/src/*.bmb")))),
    ]
    for label, paths in corp:
        r = analyse(label, paths, bmb)
        print(json.dumps(r, ensure_ascii=False))
        if write:
            with open(write, "a", encoding="utf-8") as w:
                w.write(json.dumps(r, ensure_ascii=False) + "\n")


if __name__ == "__main__":
    main()
