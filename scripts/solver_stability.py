#!/usr/bin/env python3
"""Solver stability fuzzer (backlog 1B-40, H-STABLE, ADR-0040 §4).

  solver_stability.py [--variants N] [--json] [--bmb BMB] FILE.bmb|DIR...

Every obligation the compiler sends to Z3 (`bmb verify --emit-smt`) is solved
again under meaning-preserving changes, after Mariposa (Zhou et al., 2023):

  shuffle   the `assert` commands in another order (declarations first, in
            their order, so every symbol is declared before use)
  rename    every quoted symbol (`|a@0|`, `|unit::f|`) and every `bmb_*`
            definition to a fresh name, the same bijection everywhere
  seed      other `smt.random_seed` and `sat.random_seed` values
  all       the three together

An obligation may take several attempts (stages): the abstracted query of a
bit obligation (ADR-0048 §4), the probe with every leaf definition a macro
(ADR-0067), the full or exact query, and the retry with ground instances
(ADR-0023) when it ran. Its verdict is proved when some stage answers
`unsat`, as in the compiler. Every stage is solved, unchanged and under each
variant (backlog T2-07), and an obligation is unstable when some variant's
verdict differs from the unchanged script's. A retry the compiler did not
need is not in the script, so a variant that leaves the full query undecided
is not given one: the measure errs towards unstable. The resource limit
(`rlimit`) stays the compiler's, so verdicts do not depend on time or
machine load. Variant k of an obligation is seeded by its file, index and k:
the same input gives the same report. The ratio of unstable obligations is
health metric V7 (fixed below 2.6%, Mariposa's measured rate for Dafny and
F* projects; `scripts/health.py`). Standard library only.
"""
import argparse
import hashlib
import json
import os
import random
import re
import subprocess
import sys
import tempfile

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
KINDS = ("shuffle", "rename", "seed", "all")
SYM = re.compile(r"\|[^|]*\||\bbmb_[A-Za-z0-9_]+\b")
SEED = re.compile(r"\(set-option :(smt|sat)\.random_seed \d+\)")


STAGE = re.compile(r"^; (probe of obligation|obligation|exact query of obligation|retry of obligation) (\d+)",
                   re.M)


def blocks(script):
    """The obligations of one emitted script, in order: (header comment,
    [stage bodies]). The stages are the obligation's attempts in the order
    the compiler makes them: the probe (`; probe of obligation k`) comes just
    before the full query (`; obligation k`), or after the abstracted
    attempt of a bit obligation (whose header is `; obligation k`) and
    before its exact query (`; exact query of obligation k`); a retry
    (`; retry of obligation k`) follows."""
    heads, stages = {}, {}
    marks = list(STAGE.finditer(script))
    for n, m in enumerate(marks):
        end = marks[n + 1].start() if n + 1 < len(marks) else len(script)
        line, _, body = script[m.start():end].partition("\n")
        k = int(m.group(2))
        stages.setdefault(k, []).append(body)
        if m.group(1) == "obligation":
            heads[k] = line[len("; obligation "):].strip()
    return [(heads.get(k, str(k)), stages[k]) for k in stages]


def commands(body):
    """Top-level S-expressions of a script body, in order (strings and quoted
    symbols are skipped when matching parentheses)."""
    cmds, depth, start, i = [], 0, None, 0
    while i < len(body):
        c = body[i]
        if c == "|":
            i = body.index("|", i + 1)
        elif c == '"':
            i = body.index('"', i + 1)
        elif c == ";" and depth == 0:
            i = body.find("\n", i)
            if i < 0:
                break
        elif c == "(":
            if depth == 0:
                start = i
            depth += 1
        elif c == ")":
            depth -= 1
            if depth == 0:
                cmds.append(body[start:i + 1])
        i += 1
    return cmds


def variant(cmds, kind, rng):
    cmds = list(cmds)
    if kind in ("shuffle", "all"):
        # Declarations keep their order and come first, so every symbol is
        # declared before an assert uses it; the asserts follow, shuffled.
        asserts = [c for c in cmds if c.startswith("(assert")]
        rng.shuffle(asserts)
        decls = [c for c in cmds if not c.startswith(("(assert", "(check-sat"))]
        cmds = decls + asserts + [c for c in cmds if c.startswith("(check-sat")]
    if kind in ("rename", "all"):
        names = {}

        def fresh(m):
            s = m.group(0)
            if s not in names:
                names[s] = f"|v{len(names)}_{rng.randrange(1 << 30):x}|" if s.startswith("|") else f"bmb_r{len(names)}"
            return names[s]
        cmds = [SYM.sub(fresh, c) for c in cmds]
    if kind in ("seed", "all"):
        s = rng.randrange(1, 1 << 16)
        cmds = [SEED.sub(lambda m: f"(set-option :{m.group(1)}.random_seed {s})", c) for c in cmds]
    return cmds


def solve(scripts, z3="z3"):
    """Verdicts of several obligation scripts in one Z3 process (each starts
    with `(reset)`)."""
    text = "\n".join(scripts) + "\n"
    r = subprocess.run([z3, "-in"], input=text, capture_output=True, text=True)
    lines = [line.strip() for line in r.stdout.splitlines()]
    out = [x for x in lines if x in ("sat", "unsat", "unknown")]
    errors = [x for x in lines if x.startswith("(error")]
    if errors or len(out) != len(scripts):
        sys.exit(f"solver_stability: {len(scripts)} queries, {len(out)} answers\n{r.stdout[-2000:]}{r.stderr[-2000:]}")
    return out


def sources(paths):
    """(name, [files]) per program: a directory is a workspace or a seed (its
    src/*.bmb, or its *.bmb), a .bmb file is a program alone."""
    progs = []
    for p in paths:
        if os.path.isdir(p):
            d = os.path.join(p, "src") if os.path.isdir(os.path.join(p, "src")) else p
            fs = sorted(os.path.join(d, f) for f in os.listdir(d) if f.endswith(".bmb"))
            if fs:
                progs.append((os.path.relpath(p, ROOT), fs))
        else:
            progs.append((os.path.relpath(p, ROOT), [p]))
    return progs


def proved(answers):
    """An obligation's verdict from its stages' answers: some `unsat`."""
    return "unsat" in answers


def unstable_obligations(name, f, obs, variants):
    """(answers per obligation, {index: changes}) for the obligations `obs`
    (stage command lists) of definition file `f` of program `name`: every
    stage solved unchanged and under each variant, the verdicts compared."""
    queries = ["\n".join(c) for stages in obs for c in stages]
    tried = []
    for i, stages in enumerate(obs):
        for kind in KINDS:
            for k in range(variants):
                key = f"{name}:{f}:{i}:{kind}:{k}"
                rng = random.Random(hashlib.sha256(key.encode()).digest())
                tried.append((i, kind, k, len(stages)))
                queries += ["\n".join(variant(c, kind, rng)) for c in stages]
    answers = solve(queries)
    base, at = [], 0
    for stages in obs:
        base.append(answers[at:at + len(stages)])
        at += len(stages)
    changed = {}
    for i, kind, k, n in tried:
        got = answers[at:at + n]
        at += n
        if proved(got) != proved(base[i]):
            changed.setdefault(i, []).append(f"{kind}#{k}:{'/'.join(got)}")
    return base, changed


def measure(paths, variants=1, bmb=None):
    bmb = bmb or os.environ.get("BMB", os.path.join(ROOT, "target", "release", "bmb"))
    rows, unstable = [], []
    total = stages = 0
    for name, files in sources(paths):
        with tempfile.TemporaryDirectory() as d:
            subprocess.run([bmb, "verify", "--emit-smt", d, *files], capture_output=True, text=True)
            for f in sorted(os.listdir(d)):
                text = open(os.path.join(d, f)).read()
                obs = blocks(text)
                if not obs:
                    continue
                cmds = [[commands(body) for body in bodies] for _, bodies in obs]
                base, changed = unstable_obligations(name, f, cmds, variants)
                for i, (head, bodies) in enumerate(obs):
                    total += 1
                    stages += len(bodies)
                    if i in changed:
                        unstable.append({"program": name, "def": f[:-len(".smt2")], "obligation": head,
                                         "base": "/".join(base[i]), "changed": changed[i]})
        rows.append(name)
    return {"programs": len(rows), "obligations": total, "stages": stages, "variants_per_kind": variants,
            "unstable": len(unstable), "ratio": round(len(unstable) / total, 4) if total else 0.0,
            "unstable_obligations": unstable}


def default_paths():
    """The corpus and the bench seeds: every program the gate proves."""
    t = os.path.join(ROOT, "tests")
    out = [os.path.join(t, "verify", "pass", f) for f in sorted(os.listdir(os.path.join(t, "verify", "pass")))
           if f.endswith(".bmb")]
    out += [os.path.join(t, "run", f) for f in sorted(os.listdir(os.path.join(t, "run"))) if f.endswith(".bmb")]
    for b in ("m1", "m1b", "bvb"):
        s = os.path.join(ROOT, "bench", b, "seed")
        if os.path.isdir(s):
            out.append(s)
    return out


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("paths", nargs="*")
    ap.add_argument("--variants", type=int, default=1, help="variants per kind and obligation")
    ap.add_argument("--json", action="store_true")
    ap.add_argument("--bmb")
    args = ap.parse_args()
    r = measure(args.paths or default_paths(), args.variants, args.bmb)
    if args.json:
        print(json.dumps(r, ensure_ascii=False))
        return
    print(f"programs {r['programs']}, obligations {r['obligations']} ({r['stages']} stages), "
          f"variants {len(KINDS)} x {args.variants}")
    print(f"unstable {r['unstable']} ({100 * r['ratio']:.2f}%)")
    for u in r["unstable_obligations"]:
        print(f"  {u['program']} {u['def']} [{u['obligation']}]: base {u['base']}, {', '.join(u['changed'])}")


if __name__ == "__main__":
    main()
