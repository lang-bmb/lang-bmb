#!/usr/bin/env python3
"""This repository reads on its own: no file outside the listed exceptions
names a task of the project's private backlog, a working document of the
workspace, or carries Korean prose.

  scripts/standalone.py check

Decision records (`docs/adr`) are historical and the benches (`bench/`) carry
frozen material; both are outside this check. An exception below is a file
or one exact line fragment, with its reason.
"""
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
OUTSIDE = (":!bench", ":!docs/adr")

TASK = re.compile(r"(?<![0-9A-Za-z-])(?:INF|T[0-9]|[0-9][A-Z])-[0-9]{2}[a-z0-9]*(?![0-9A-Za-z])")
WORKSPACE = re.compile(r"dev-docs/|bmb-works|claudedocs|\b(?:HEALTH|BACKLOG|AUTONOMY|JUDGMENT)\.md|\b[Bb]acklog\b")
HANGUL = re.compile("[가-힣]")

# Files whose non-ASCII text is the data under test, or a record.
DATA = (
    "tests/port/lexer/",                # lexer inputs: arbitrary bytes and scripts
    "tests/conformance/",               # non-ASCII in comments and literals is a case
    "tests/port/emit/c1-literals.txt",  # a multi-byte `char` literal
    "tests/port/emit/d1-slices.txt",    # multi-byte text in a slice
    "tests/port/emit/f1-text.txt",      # pushes and compares multi-byte characters
    "tests/port/emit/f1-text.out",
    "tests/port/json_edge.txt",         # numbers such as `1.5E-10`
    "tests/surface.json",               # the history of the surface record
    "compiler/src/json.rs",             # multi-byte JSON strings in its tests
    "scripts/standalone.py",            # this file names what it looks for
)
# Exact fragments, each with the reason it stays.
LINES = (
    # The bytes of the tool list are frozen as a measurement condition.
    '"Write in one call (backlog 2D-24): when `summary` is given',
    # The test asserts that tool descriptions do not carry these words.
    '!db.contains("backlog")',
    # The one non-ASCII summary of the fuzzer, and a unit path that must be refused.
    '"요약"',
    '"한글"',
    # What a tool description leaves out, said in words.
    "ADR or backlog numbers",
)
# Scripts whose output is a Korean report kept outside this repository.
REPORTS = ("scripts/components.py", "scripts/test_components.py", "scripts/cross_solver.py",
           "scripts/gate_power.py", "scripts/test_gate_power.py", "scripts/check.sh")


def tracked(root):
    """The files git tracks; in a copy without a repository, the files of the tree."""
    r = subprocess.run(["git", "ls-files", "--", ".", *OUTSIDE], cwd=root, capture_output=True, text=True)
    if r.returncode == 0:
        return r.stdout.splitlines()
    skip = ("bench/", "docs/adr/", ".git/")
    out = []
    for p in sorted(root.rglob("*")):
        rel = p.relative_to(root).as_posix()
        if p.is_file() and not rel.startswith(skip) and "/target/" not in f"/{rel}" and "/." not in f"/{rel}":
            out.append(rel)
    return out


def problems(root=ROOT):
    files = tracked(root)
    out = []
    for f in files:
        if f.startswith(DATA):
            continue
        try:
            text = (root / f).read_text(encoding="utf-8")
        except (UnicodeDecodeError, FileNotFoundError, IsADirectoryError):
            continue
        for i, line in enumerate(text.splitlines(), 1):
            if any(x in line for x in LINES):
                continue
            if TASK.search(line):
                out.append(f"{f}:{i}: names a backlog task")
            if WORKSPACE.search(line):
                out.append(f"{f}:{i}: names a workspace document")
            if HANGUL.search(line) and f not in REPORTS:
                out.append(f"{f}:{i}: Korean text")
    return out


def main():
    if sys.argv[1:] != ["check"]:
        raise SystemExit(__doc__)
    out = problems()
    for p in out[:40]:
        print(p)
    if out:
        raise SystemExit(f"standalone: {len(out)} lines (the reason belongs in the text itself; data goes in the script's lists)")


if __name__ == "__main__":
    main()
