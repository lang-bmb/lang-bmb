#!/usr/bin/env python3
"""Holds the bounds of the port's self-proof (ADR-0162, decisions 2 and 3).

    trusted_steps.py rows FILE     the status rows of `bmb verify` over the port
    trusted_steps.py steps [DIR]   the text of the port (default `selfhost/`)

`rows` fails when a row is not `verified` or carries `trust`, `escapes` or
`termination`: the bound on definitions with an escape or with termination
left unproved is 0. `bmb verify` itself ends with 0 for a program proved
with trusted facts, so its exit code does not say this.

`steps` fails when one of the places the proof gate trusts by reading is
found somewhere else than the record lists it:

- the answer `unsat` is made in `solver::verdict_word` and nowhere else;
- an obligation is marked proved in `prove::decide` and
  `prove::decide_sub` and nowhere else;
- code generation writes `; proved Exxxx` only in a function that asks
  `Gen::proved` about that code.

These are checks of the text, not proofs: they keep the closed list from
growing without a record, and a new place fails here until the record and
this script name it.
"""
import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
BOUND_KEYS = ("trust", "escapes", "termination")
UNSAT_MAKERS = {("solver.bmb", "verdict_word")}
PROVED_SETTERS = {("prove.bmb", "decide"), ("prove.bmb", "decide_sub")}
FN = re.compile(r"^\s+fn (\w+)\b")


def bad_rows(text):
    """The reasons the rows of `text` break the bound, one per row."""
    out, seen = [], 0
    for line in text.splitlines():
        if not line.startswith('{"def"'):
            if line.strip():
                out.append(f"not a status row: {line[:200]}")
            continue
        seen += 1
        row = json.loads(line)
        if row.get("status") != "verified":
            out.append(f"{row['def']}: status {row.get('status')}")
        for k in BOUND_KEYS:
            if k in row:
                out.append(f"{row['def']}: {k} {row[k]}")
    if seen == 0:
        out.append("no status rows")
    return out


def functions(text):
    """(name, text) of each function: from its `fn` line to the next one."""
    out, name, lines = [], None, []
    for line in text.splitlines():
        m = FN.match(line)
        if m:
            if name is not None:
                out.append((name, "\n".join(lines)))
            name, lines = m.group(1), []
        if name is not None:
            lines.append(line)
    if name is not None:
        out.append((name, "\n".join(lines)))
    return out


def bad_steps(sources):
    """The trusted places of `sources` ({file name: text}) that are outside
    the closed list, and the listed places that are no longer there."""
    out = []
    makers, setters = set(), set()
    for file, text in sorted(sources.items()):
        for name, body in functions(text):
            # `Answer::Unsat =>` reads the answer; any other use makes it.
            if re.search(r"Answer::Unsat\b(?!\s*=>)", body):
                makers.add((file, name))
            if re.search(r"\bproved\s*=\s*true\b|\bproved:\s*true\b", body):
                setters.add((file, name))
            for code in sorted(set(re.findall(r'"[^"\n]*; proved (E\d{4})', body))):
                if f'proved("{code}"' not in body:
                    out.append(f"{file}: `{name}` writes `; proved {code}` without asking `proved(\"{code}\", ..)`")
    for what, found, listed in (("makes the answer unsat", makers, UNSAT_MAKERS), ("marks an obligation proved", setters, PROVED_SETTERS)):
        for file, name in sorted(found - listed):
            out.append(f"{file}: `{name}` {what} and is not in the closed list")
        for file, name in sorted(listed - found):
            out.append(f"{file}: `{name}` is in the closed list and no longer {what}")
    return out


def main(argv):
    if argv[:1] == ["rows"] and len(argv) == 2:
        bad = bad_rows(Path(argv[1]).read_text(encoding="utf-8"))
    elif argv[:1] == ["steps"] and len(argv) <= 2:
        d = Path(argv[1]) if len(argv) == 2 else ROOT / "selfhost"
        bad = bad_steps({p.name: p.read_text(encoding="utf-8") for p in sorted(d.glob("*.bmb"))})
    else:
        print(__doc__)
        return 2
    if bad:
        print("\n".join(f"trusted_steps: {b}" for b in bad[:40]))
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
