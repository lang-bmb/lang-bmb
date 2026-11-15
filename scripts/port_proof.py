#!/usr/bin/env python3
"""Proves a BMB program's definitions in several processes and merges their
caches (backlog INF-41).

`bmb verify` proves one definition after another in one process. The port
is one program of several hundred definitions, and the gate proves it from
an empty cache, so the time grows with the port. Here the definitions are
dealt out to N processes (`bmb verify --def ..`, each with a cache directory
of its own), and the caches are merged into DIR: both cache files hold one
self-contained line per entry under a header, so the merge is the union of
the lines. A `bmb verify --cache DIR` of the whole program afterwards finds
the proofs, proves what is left (a `spec fn` with obligations of its own)
and gives the rows a single run gives.

    port_proof.py [--jobs N] DIR FILE..

Nothing is trusted that the single run would not prove: every proof in DIR
was found in this run, from empty caches, by the same compiler, and the
verdict is the following run's.
"""
import json
import os
import shutil
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
FILES = ("proofs", "defs")
HEADER = {"proofs": 1, "defs": 2}


def definitions(bmb, files):
    """The executable and proof functions (`bmb reads` lists every
    definition with its kind without proving anything). The few `spec fn`s
    that have obligations of their own are left to the run after the merge:
    `verify --def` reports a name without a status line only after proving
    the others, so a list of names it may not know cannot be tried first."""
    r = subprocess.run([bmb, "reads", *files], capture_output=True, text=True)
    if r.returncode != 0:
        sys.exit(f"port_proof: the program does not check\n{r.stdout[:2000]}")
    rows = [json.loads(line) for line in r.stdout.splitlines() if line.startswith('{"def"')]
    return [row["def"] for row in rows if row["kind"] in ("fn", "proof fn")]


def merge(parts, out):
    out.mkdir(parents=True, exist_ok=True)
    for name in FILES:
        head, lines, seen = None, [], set()
        for p in parts:
            f = p / name
            if not f.exists():
                continue
            text = f.read_text(encoding="utf-8").splitlines()
            h = text[: HEADER[name]]
            if head is None:
                head = h
            elif h != head:
                sys.exit(f"port_proof: {f} has another header")
            for line in text[HEADER[name] :]:
                if line not in seen:
                    seen.add(line)
                    lines.append(line)
        if head is not None:
            (out / name).write_text("\n".join(head + lines) + "\n", encoding="utf-8")


def main(argv):
    jobs = os.cpu_count() or 1
    if argv[:1] == ["--jobs"]:
        jobs, argv = int(argv[1]), argv[2:]
    if len(argv) < 2:
        print(__doc__)
        return 2
    out, files = Path(argv[0]), argv[1:]
    bmb = str(ROOT / "target" / "debug" / "bmb")
    names = definitions(bmb, files)
    jobs = max(1, min(jobs, len(names)))
    if out.exists():
        shutil.rmtree(out)
    parts = [out.with_name(f"{out.name}.part{i}") for i in range(jobs)]
    procs = []
    for i, p in enumerate(parts):
        if p.exists():
            shutil.rmtree(p)
        flags = [x for n in names[i::jobs] for x in ("--def", n)]
        procs.append(subprocess.Popen([bmb, "verify", "--cache", str(p), *flags, *files], stdout=subprocess.PIPE, text=True))
    failed = []
    for proc in procs:
        text, _ = proc.communicate()
        rows = [line for line in text.splitlines() if '"status":"verified"' not in line]
        if proc.returncode != 0 or rows:
            failed += rows or [f"bmb verify ended with {proc.returncode}"]
    merge(parts, out)
    for p in parts:
        shutil.rmtree(p, ignore_errors=True)
    if failed:
        print("\n".join(failed[:20]))
        return 1
    print(f"port_proof: {len(names)} definitions in {jobs} processes")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
