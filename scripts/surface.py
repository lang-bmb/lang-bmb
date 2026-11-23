#!/usr/bin/env python3
"""The language surface fingerprint (ADR-0100 draft).

Under the stage0 freeze (ADR-0037 §3) the compiler may change only for bug
fixes and changes the bootstrap needs. This script makes a change of the
accepted language visible: for every corpus program it keeps a digest of
what the compiler says about it (`bmb check` diagnostics and exit code,
`bmb fmt` normal form, `bmb hash` identities), next to the digest of the
source. An entry whose source is unchanged but whose result changed is a
change of the language (or of the compiler's observable behavior), and a
new diagnostic code is one too.

  scripts/surface.py record [--bmb BIN]   rewrite tests/surface.json
  scripts/surface.py check  [--bmb BIN]   compare; under a freeze, fail on a
                                          change not in "exceptions"

An exception is {"kind": .., "name": .., "reason": ..} with a kind and name
as `check` prints them (a bug fix, a change the bootstrap needs). `record`
moves the exceptions to "history".

Corpora: tests/check, tests/verify, tests/run (one program per file) and
lib/std (one program). An entry whose source was edited cannot be compared,
so the record must follow every edit: `check` fails while an entry's source
digest differs from the recorded one (a stale record compares
nothing), and `record` refuses under a freeze while a comparable entry
changed without an exception. A program's source digest is its own files:
an edit of the standard library leaves the other entries comparable, and a
result it changes is a change like any other. The JSON's "frozen" is false
until the freeze is declared (human approval); until then a change
needs no reason but `check` still fails until it is recorded.
"""
import argparse
import hashlib
import json
import subprocess
import sys
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
STATE = ROOT / "tests/surface.json"
CORPORA = ["tests/check", "tests/verify", "tests/run"]
LIBRARY = "lib/std"


def digest(data):
    return hashlib.sha256(data).hexdigest()[:16]


def programs(root):
    """(name, [files]) for every corpus program, in a fixed order."""
    out = []
    for c in CORPORA:
        for f in sorted((root / c).rglob("*.bmb")):
            out.append((f.relative_to(root).as_posix(), [f]))
    lib = sorted((root / LIBRARY).glob("*.bmb"))
    if lib:
        out.append((LIBRARY, lib))
    return out


def run(bmb, args, files):
    # Relative paths: diagnostics name files as given, and the gate runs in
    # another checkout (scripts/ship.sh).
    rel = [Path(f).resolve().relative_to(ROOT).as_posix() for f in files]
    p = subprocess.run([bmb, *args, *rel], capture_output=True, cwd=ROOT)
    return p.returncode, p.stdout


def observe(bmb, files):
    """The digest of what the compiler says about one program."""
    h = hashlib.sha256()
    for args in (["check"], ["fmt"], ["hash"]):
        code, out = run(bmb, args, files)
        # Paths in diagnostics are as given (relative to ROOT): stable.
        h.update(f"{' '.join(args)} {code}\n".encode())
        h.update(out)
    return h.hexdigest()[:16]


def codes(bmb):
    code, out = run(bmb, ["codes", "--md"], [])
    if code != 0:
        sys.exit(f"surface: `bmb codes --md` failed ({code})")
    return sorted(
        line.split("|")[1].strip()
        for line in out.decode().splitlines()
        if line.startswith("| ") and line.split("|")[1].strip()[:1] in ("E", "W")
    )


def snapshot(bmb, root=ROOT):
    progs = programs(root)
    with ThreadPoolExecutor() as ex:
        seen = list(ex.map(lambda p: observe(bmb, p[1]), progs))
    entries = {}
    for (name, files), out in zip(progs, seen):
        src = digest(b"".join(f.read_bytes() for f in files))
        entries[name] = {"src": src, "out": out}
    return {"codes": codes(bmb), "programs": entries}


def compare(old, new):
    """Changes of the surface: (kind, name) for each."""
    changes = []
    for c in sorted(set(old["codes"]) - set(new["codes"])):
        changes.append(("code removed", c))
    for c in sorted(set(new["codes"]) - set(old["codes"])):
        changes.append(("code added", c))
    for name, e in sorted(new["programs"].items()):
        o = old["programs"].get(name)
        if o and o["src"] == e["src"] and o["out"] != e["out"]:
            changes.append(("result changed", name))
    return changes


def stale(old, new):
    """The entries the record cannot be compared on: added, removed, or
    with another source."""
    o, n = old["programs"], new["programs"]
    return sorted(k for k in set(o) | set(n) if k not in o or k not in n or o[k]["src"] != n[k]["src"])


def unexcused(old, new):
    """The changes of the surface that have no exception."""
    allowed = {(x["kind"], x["name"]) for x in old.get("exceptions", [])}
    return [c for c in compare(old, new) if c not in allowed]


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("cmd", choices=["record", "check"])
    ap.add_argument("--bmb", default=str(ROOT / "target/debug/bmb"))
    a = ap.parse_args()
    new = snapshot(a.bmb)
    if a.cmd == "record":
        old = json.loads(STATE.read_text()) if STATE.exists() else {}
        for x in old.get("exceptions", []):
            if not x.get("reason"):
                sys.exit(f"surface: exception {x} has no reason")
        if old.get("frozen"):
            # Recording would absorb the change: under the freeze it needs
            # its exception first.
            for kind, name in unexcused(old, new):
                print(f"surface: {kind}: {name} (no exception)", file=sys.stderr)
            if unexcused(old, new):
                sys.exit("surface: not recorded: the freeze is on, name each change in \"exceptions\"")
        # Recording consumes the exceptions: each one allowed one change of
        # one entry, and the history keeps it.
        state = {
            "frozen": old.get("frozen", False),
            "exceptions": [],
            "history": old.get("history", []) + old.get("exceptions", []),
            **new,
        }
        STATE.write_text(json.dumps(state, indent=1, sort_keys=True) + "\n")
        print(f"surface: recorded {len(new['programs'])} programs, {len(new['codes'])} codes")
        return 0
    if not STATE.exists():
        sys.exit("surface: no tests/surface.json; run `scripts/surface.py record`")
    old = json.loads(STATE.read_text())
    changes = compare(old, new)
    allowed = {(x["kind"], x["name"]) for x in old.get("exceptions", [])}
    open_changes = unexcused(old, new)
    for kind, name in changes:
        mark = "" if (kind, name) in allowed else " (no exception)"
        print(f"surface: {kind}: {name}{mark}")
    if not changes:
        print(f"surface: unchanged ({len(new['programs'])} programs, {len(new['codes'])} codes)")
    if open_changes and old.get("frozen"):
        print(
            "surface: the stage0 freeze is on (ADR-0037 §3): record each change as an "
            "exception with its reason in tests/surface.json, then "
            "`scripts/surface.py record`",
            file=sys.stderr,
        )
        return 1
    if open_changes:
        # Before the freeze a change needs no reason, but the record must
        # follow it, or the freeze would start from a stale fingerprint.
        print(
            "surface: the fingerprint is stale: `scripts/surface.py record` "
            "(the changes above are the commit's language changes)",
            file=sys.stderr,
        )
        return 1
    behind = stale(old, new)
    if behind:
        print(
            f"surface: the record is behind {len(behind)} entr{'y' if len(behind) == 1 else 'ies'} "
            f"({', '.join(behind[:3])}{', ..' if len(behind) > 3 else ''}): "
            "`scripts/surface.py record`",
            file=sys.stderr,
        )
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
