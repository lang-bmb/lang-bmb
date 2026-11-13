#!/usr/bin/env python3
"""Sizes behind dev-docs/measurements/t1-44f.md (backlog T1-44f): can a set
of function-sized changes or items reach the token criteria of H-REVIEW
(spec view <= 1/5 of the code view) and H-INTENT (<= 1/2)?

    python3 dev-docs/measurements/t1-44f.py     # needs target/debug/bmb

Every executable function of `lib/std` that has an `ensures` is measured:
the tokens (ceil(bytes / 4), ADR-0021 §4) of its body, of its contract
lines, of the contract text a reviewer is shown (the contract lines and the
unit's `spec fn`s they name, directly or through other `spec fn`s: the
extraction of bench/review/views.py, here over files that carry comments)
and of what `bmb examples` prints for it. Standard library only.
"""
import collections
import glob
import json
import math
import re
import statistics
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
BMB = ROOT / "target" / "debug" / "bmb"
FILES = sorted(glob.glob(str(ROOT / "lib" / "std" / "*.bmb")))
# The smallest `bmb weaken` output of bench/review (ADR-0128: 85..210 tokens).
WEAKEN_MIN, WEAKEN_MAX = 85, 210


def tok(s):
    return math.ceil(len(s.encode()) / 4)


def definitions(path):
    """(module, {name: (kind, lines)}) of one library file."""
    lines = Path(path).read_text().split("\n")
    mod = next(m.group(1) for m in (re.match(r"mod (\w+) \{$", l) for l in lines) if m)
    defs, cur = {}, None
    for raw in lines:
        if not raw.startswith("    "):
            continue
        line = raw[4:]
        if cur is None:
            if not re.match(r"(#\[|spec fn |proof fn |fn )", line):
                continue
            cur = [line]
        else:
            cur.append(line)
        code = [l for l in cur if not l.startswith("#[")]
        if code and ((len(code) == 1 and line.rstrip().endswith("}")) or line == "}"):
            m = re.match(r"(spec fn|proof fn|fn) (\w+)", code[0])
            if m:
                defs[m.group(2)] = (m.group(1), cur)
            cur = None
    return mod, defs


def measured():
    rows = []
    for path in FILES:
        mod, defs = definitions(path)
        specs = {n: d for n, (k, d) in defs.items() if k == "spec fn"}

        def named(ls):
            return {m.group(1) for l in ls for m in re.finditer(r"(?<![\w:])(\w+)\s*\(", l) if m.group(1) in specs}

        for name, (kind, d) in defs.items():
            code = [l for l in d if not l.startswith("#[")]
            if kind != "fn" or "{" not in code:
                continue
            brace = code.index("{")
            contract = [l for l in code[:brace] if not l.strip().startswith("decreases")]
            if not any(l.strip().startswith("ensures") for l in contract):
                continue
            shown, todo = set(), named(contract[1:])
            while todo:
                n = todo.pop()
                if n not in shown:
                    shown.add(n)
                    todo |= named(specs[n]) - shown
            text = "\n".join(contract + [l for n in sorted(shown) for l in specs[n]])
            rows.append({"mod": mod, "name": name, "body": tok("\n".join(code[brace:])),
                         "body_lines": len(code) - brace, "contract": tok("\n".join(contract)), "spec": tok(text)})
    return rows


def paths():
    out = subprocess.run([str(BMB), "map", *FILES], capture_output=True, text=True, check=True).stdout
    found = {}
    for l in out.splitlines():
        d = json.loads(l)
        if d.get("level") == "L0" and d.get("kind") == "def" and d.get("def_kind") == "fn":
            found[tuple(d["path"].split("::")[-2:])] = d["path"]
    return found


def examples(rows):
    where = paths()
    why = collections.Counter()
    for r in rows:
        p = subprocess.run([str(BMB), "examples", where[(r["mod"], r["name"])], "--", *FILES],
                           capture_output=True, text=True)
        ls = [json.loads(x) for x in p.stdout.splitlines() if x.startswith("{")]
        r["examples"], r["rows"] = tok(p.stdout), max(len(ls) - 1, 0)
        reason = ls[-1].get("reason") if ls else None
        if r["rows"]:
            why["rows"] += 1
        elif reason and "not shown in full" in reason:
            why["a parameter or the result has a type whose values are not shown"] += 1
        elif reason and "by name only" in reason:
            why["the contract names an opaque `spec fn`"] += 1
        else:
            why["no row"] += 1
    return why


def quartiles(xs):
    return ", ".join(str(round(x, 2)) for x in statistics.quantiles(xs, n=4))


def main():
    rows = measured()
    print(f"functions with a contract: {len(rows)}")
    for key in ("body_lines", "body", "contract", "spec"):
        print(f"  {key}: quartiles {quartiles([r[key] for r in rows])}, max {max(r[key] for r in rows)}")
    print(f"  contract text larger than the body: {sum(r['spec'] > r['body'] for r in rows)}")
    # H-REVIEW, the change most favourable to the criterion: the whole body
    # rewritten (every body line removed and added), the contract untouched.
    for w in (WEAKEN_MIN, WEAKEN_MAX):
        n = sum(w / (2 * r["body"]) <= 1 / 5 for r in rows)
        print(f"H-REVIEW: whole body rewritten, `bmb weaken` output {w}: ratio <= 1/5 for {n} functions")
    # H-INTENT with no examples at all: contract text against contract text and body.
    n = sum(r["spec"] / (r["spec"] + r["body"]) <= 1 / 2 for r in rows)
    print(f"H-INTENT: contract text alone / (contract text + body) <= 1/2 for {n} functions")
    why = examples(rows)
    for k, v in why.most_common():
        print(f"  bmb examples: {k}: {v}")
    have = [r for r in rows if r["rows"]]
    ratio = [(r["spec"] + r["examples"]) / (r["spec"] + r["body"]) for r in have]
    print(f"  examples tokens where there are rows: quartiles {quartiles([r['examples'] for r in have])}")
    print(f"H-INTENT: (contract text + examples) / (contract text + body), {len(have)} functions: "
          f"quartiles {quartiles(ratio)}, <= 1/2 for {sum(x <= 0.5 for x in ratio)}")


if __name__ == "__main__":
    main()
