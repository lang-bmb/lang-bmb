#!/usr/bin/env python3
"""Token cost of counting loops: `while` against the range `for` (backlog
2R-04f, ADR-0049 §2, ADR-0057).

Deterministic, offline. For each program, `bmb fix --print` applies the
canonical-loop rewrite (W0907, ADR-0057) and nothing else (the programs
check); the script counts the rewrites and the lexical tokens before and
after (the lexer of `measure.py`), and proves both versions with
`bmb verify` to show that every definition keeps its verdict. Query
identity of the two forms is a compiler test
(`for_queries_are_the_while_expansion`).

Usage:
  python3 bench/syntax/for_tokens.py target/debug/bmb [--rev REV] FILE...
  (`--rev` reads the files as they were at a git revision.)
"""

import json
import os
import subprocess
import sys
import tempfile

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from measure import lex  # noqa: E402

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))


def verdicts(bmb, path):
    out = subprocess.run([bmb, "verify", path], capture_output=True, text=True).stdout
    rows = [json.loads(line) for line in out.splitlines() if line.startswith("{\"def\"")]
    return {r["def"]: r["status"] for r in rows}


def main():
    args = sys.argv[1:]
    if not args:
        raise SystemExit(__doc__)
    bmb, rest = args[0], args[1:]
    rev = None
    if rest[:1] == ["--rev"]:
        rev, rest = rest[1], rest[2:]
    total = [0, 0, 0]
    print("| 프로그램 | 바뀐 루프 | 토큰 전 | 토큰 뒤 | 판정 |")
    print("| --- | ---: | ---: | ---: | --- |")
    with tempfile.TemporaryDirectory() as tmp:
        for f in rest:
            if rev:
                text = subprocess.run(["git", "show", f"{rev}:{f}"], cwd=ROOT, capture_output=True, text=True, check=True).stdout
            else:
                text = open(os.path.join(ROOT, f)).read()
            before = os.path.join(tmp, "before.bmb")
            open(before, "w").write(text)
            rep = subprocess.run([bmb, "fix", before], capture_output=True, text=True).stdout
            codes = [json.loads(line).get("code") for line in rep.splitlines() if line.startswith("{\"round\"")]
            assert set(codes) <= {"W0907"}, f"{f}: other fixes {codes}"
            fixed = subprocess.run([bmb, "fix", "--print", before], capture_output=True, text=True).stdout
            after = os.path.join(tmp, "after.bmb")
            open(after, "w").write(fixed)
            same = verdicts(bmb, before) == verdicts(bmb, after)
            tb, ta = len(lex(text)), len(lex(fixed))
            total[0] += len(codes)
            total[1] += tb
            total[2] += ta
            print(f"| `{f}` | {len(codes)} | {tb} | {ta} | {'같음' if same else '다름'} |")
    n, tb, ta = total
    per = (tb - ta) / n if n else 0
    print(f"| 합 | {n} | {tb} | {ta}({(ta - tb) / tb * 100:+.1f}%) | 루프당 {per:.1f}토큰 |")


if __name__ == "__main__":
    main()
