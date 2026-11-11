#!/usr/bin/env python3
"""Checks the backlog for record errors (AUTONOMY.md §7, INF-06).

The backlog is dev-docs/BACKLOG.md (active rows) and dev-docs/backlog/
(finished rows, INF-30); the rules hold over both, read through backlog.py.

Rules:
  - task IDs are unique
  - every dependency names an existing task
  - a done task (✅) cites evidence after the mark (commit hash, test name, ...)
  - an in-progress task (🔄) has all dependencies done
  - every ID referenced in backticks elsewhere in the files exists
  - finished rows and old plans are in the archive (`backlog.py check`)
"""
import re
import sys

import backlog

ID = r"(?:\d|INF|T\d)[A-Z]?-\d{2}[a-z]?"
ID_RE = re.compile(rf"^{ID}$")
REF_RE = re.compile(rf"`({ID})`")
CELL_RE = re.compile(r"(?<!\\)\|")


def main() -> int:
    lines = list(backlog.numbered())
    errors = []
    tasks = {}  # id -> (deps, status, place)

    for name, n, line in lines:
        n = f"{name} line {n}"
        if not line.startswith("|"):
            continue
        # `\|` is a pipe inside a cell (Markdown), not a cell border.
        cells = [c.strip() for c in CELL_RE.split(line.strip().strip("|"))]
        if len(cells) != 5 or not ID_RE.match(cells[0].strip("~")):
            continue
        tid = cells[0].strip("~")
        if tid in tasks:
            errors.append(f"{n}: duplicate task id {tid}")
            continue
        deps = [] if cells[3] in ("—", "-", "") else [d.strip() for d in cells[3].split(",")]
        tasks[tid] = (deps, cells[4], n)

    for tid, (deps, status, n) in tasks.items():
        for d in deps:
            if d not in tasks:
                errors.append(f"{n}: {tid} depends on unknown task {d!r}")
        if status.startswith("✅") and not status[1:].strip():
            errors.append(f"{n}: {tid} is done but cites no evidence")
        if status.startswith("🔄"):
            for d in deps:
                if d in tasks and not tasks[d][1].startswith("✅"):
                    errors.append(f"{n}: {tid} is in progress but dependency {d} is not done")

    for name, n, line in lines:
        for ref in REF_RE.findall(line):
            if ref not in tasks:
                errors.append(f"{name} line {n}: reference to unknown task {ref}")
    stale = backlog.archive(write=False)
    if stale:
        errors.append(f"BACKLOG.md holds {stale} finished rows or old plans: run `scripts/backlog.py archive`")

    if not tasks:
        errors.append("no tasks found")
    for e in errors:
        print(f"backlog: {e}", file=sys.stderr)
    return 1 if errors else 0


if __name__ == "__main__":
    sys.exit(main())
