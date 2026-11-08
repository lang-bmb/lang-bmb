#!/usr/bin/env python3
"""Writing-surface fixes: meaning kept and habits recovered (backlog 2R-04e).

Deterministic, offline. The checker answers a Rust habit with a diagnostic
whose fix carries a replacement (E02xx, E0601, E0237), and `bmb fix --write`
applies them until none applies. This injects one habit at a time into the
canonical programs of tests/run (each fully proved and in normal form) and
asks what `bmb fix --write` gives back:

  exact      the file is what `bmb fix --write` makes of the original (it
             also rewrites counted `while` loops as `for`, W0907, so that is
             the reference, not the original text)
  normal     not that text, but `bmb fmt` makes it that text
  same       neither, but the same proof verdicts (definition, status) and
             the same LLVM IR (`bmb emit-llvm`): the meaning is kept
  changed    it checks, but its verdicts or IR differ
  stuck      errors remain

Habits (the inverse of a canonical form; each at its first site in a file):

  let-type    `let x: T = e;`          -> `let x = e;`          (E0201)
  unit-ret    `-> () {`                -> `{`                   (E0209)
  suffix      `let x: i64 = 5;`        -> `let x: i64 = 5i64;`  (E0104)
  as-cast     `e as T`                 -> `T::try_from(e).unwrap()` (W0915)
  mut-param   `fn f(x: T`              -> `fn f(mut x: T`       (E0210)
  pub         `fn f(`                  -> `pub fn f(`           (E0207)
  effects     `#[effects(pure)]` line removed                  (E0601)
  turbofish   `f::<T>(`                -> `f(`                  (E0237)

Since ADR-0095 the canonical conversion is `e as T`, and `T::try_from(e)
.unwrap()` an input alias: no error, a lint warning W0915 whose fix `bmb
fix` applies. An alias habit (`alias` below) is counted although the
checker does not refuse it.

Usage: bench/syntax/fix_recovery.py BMB [--write FILE]
"""

import glob
import hashlib
import json
import os
import re
import shutil
import subprocess
import sys
import tempfile

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

HABITS = [
    ("let-type", re.compile(r"let (mut )?(\w+): [^=;]+ = "), lambda m: f"let {m.group(1) or ''}{m.group(2)} = "),
    ("unit-ret", re.compile(r"\) -> \(\) \{"), lambda m: ") {"),
    ("suffix", re.compile(r"(let (?:mut )?\w+: i64 = )(\d+);"), lambda m: f"{m.group(1)}{m.group(2)}i64;"),
    ("as-cast", re.compile(r"(?<![.:\w])(\w+) as (u8|u16|u32|u64|usize|i8|i16|i32|i64)\b"), lambda m: f"{m.group(2)}::try_from({m.group(1)}).unwrap()"),
    ("mut-param", re.compile(r"(\n    fn \w+)\((\w+): "), lambda m: f"{m.group(1)}(mut {m.group(2)}: "),
    ("pub", re.compile(r"\n    fn (\w+)\("), lambda m: f"\n    pub fn {m.group(1)}("),
    ("effects", re.compile(r"\n    #\[effects\(pure\)\]\n"), lambda m: "\n"),
    ("turbofish", re.compile(r"\b(\w+)::<[A-Za-z0-9_, ]+>\("), lambda m: f"{m.group(1)}("),
]


# Input aliases (ADR-0095): taken without an error, rewritten by `bmb fix`.
ALIASES = {"as-cast": "W0915"}


def run(args, **kw):
    return subprocess.run(args, capture_output=True, text=True, **kw)


def verdicts(bmb, path):
    out = run([bmb, "verify", path]).stdout.splitlines()
    return sorted((r["def"], r["status"]) for r in (json.loads(x) for x in out if x.startswith('{"def"')))


def ir(bmb, path, d):
    ll = os.path.join(d, "out.ll")
    r = run([bmb, "emit-llvm", path, "-o", ll])
    if r.returncode != 0 or not os.path.exists(ll):
        return None
    return hashlib.sha256(open(ll, "rb").read()).hexdigest()


def main():
    bmb = sys.argv[1]
    write = sys.argv[sys.argv.index("--write") + 1] if "--write" in sys.argv else None
    files = sorted(glob.glob(os.path.join(ROOT, "tests/run/*.bmb")))
    counts = {h: {"sites": 0, "exact": 0, "normal": 0, "same": 0, "changed": 0, "stuck": 0, "codes": {}} for h, _, _ in HABITS}
    cases = []
    with tempfile.TemporaryDirectory() as tmp:
        base = {}
        for f in files:
            text = open(f, encoding="utf-8").read()
            rd = tempfile.mkdtemp(dir=tmp)
            rp = os.path.join(rd, os.path.basename(f))
            shutil.copy(f, rp)
            run([bmb, "fix", "--write", rp])
            ref = open(rp, encoding="utf-8").read()
            for name, rx, sub in HABITS:
                m = rx.search(text)
                if not m:
                    continue
                bad = text[: m.start()] + sub(m) + text[m.end() :]
                if bad == text:
                    continue
                d = tempfile.mkdtemp(dir=tmp)
                p = os.path.join(d, os.path.basename(f))
                open(p, "w", encoding="utf-8").write(bad)
                chk = [json.loads(x) for x in run([bmb, "check", p]).stdout.splitlines() if x.startswith('{"code"')]
                errs = [x["code"] for x in chk if x.get("severity") == "error"]
                if not errs and name in ALIASES:
                    errs = [ALIASES[name]]
                elif not errs:
                    continue  # not a habit the checker refuses here
                c = counts[name]
                c["sites"] += 1
                c["codes"][errs[0]] = c["codes"].get(errs[0], 0) + 1
                run([bmb, "fix", "--write", p])
                fixed = open(p, encoding="utf-8").read()
                left = [x for x in run([bmb, "check", p]).stdout.splitlines() if '"severity":"error"' in x]
                if fixed == ref:
                    kind = "exact"
                elif left:
                    kind = "stuck"
                elif run([bmb, "fmt", p]).stdout == ref:
                    kind = "normal"
                else:
                    if f not in base:
                        base[f] = (verdicts(bmb, rp), ir(bmb, rp, rd))
                    same = (verdicts(bmb, p), ir(bmb, p, d)) == base[f]
                    kind = "same" if same else "changed"
                c[kind] += 1
                cases.append({"file": os.path.relpath(f, ROOT), "habit": name, "code": errs[0], "result": kind})
    for name, _, _ in HABITS:
        line = {"habit": name, **counts[name]}
        print(json.dumps(line, ensure_ascii=False))
        if write:
            with open(write, "a", encoding="utf-8") as w:
                w.write(json.dumps(line, ensure_ascii=False) + "\n")
    for c in cases:
        if c["result"] in ("changed", "stuck"):
            print(json.dumps(c, ensure_ascii=False))


if __name__ == "__main__":
    main()
