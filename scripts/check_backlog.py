#!/usr/bin/env python3
"""Checks dev-docs/BACKLOG.md for record errors (AUTONOMY.md §7, INF-06).

Rules:
  - task IDs are unique
  - every dependency names an existing task
  - a done task (✅) cites evidence after the mark (commit hash, test name, ...)
  - an in-progress task (🔄) has all dependencies done
  - every ID referenced in backticks elsewhere in the file exists
"""
import re
import sys
from pathlib import Path

ID = r"(?:\d|INF)[A-Z]?-\d{2}[a-z]?"
ID_RE = re.compile(rf"^{ID}$")
REF_RE = re.compile(rf"`({ID})`")


def main() -> int:
    path = Path(__file__).resolve().parent.parent / "dev-docs" / "BACKLOG.md"
    text = path.read_text(encoding="utf-8")
    errors = []
    tasks = {}  # id -> (deps, status, line)

    for n, line in enumerate(text.splitlines(), 1):
        if not line.startswith("|"):
            continue
        cells = [c.strip() for c in line.strip().strip("|").split("|")]
        if len(cells) != 5 or not ID_RE.match(cells[0].strip("~")):
            continue
        tid = cells[0].strip("~")
        if tid in tasks:
            errors.append(f"line {n}: duplicate task id {tid}")
            continue
        deps = [] if cells[3] in ("—", "-", "") else [d.strip() for d in cells[3].split(",")]
        tasks[tid] = (deps, cells[4], n)

    for tid, (deps, status, n) in tasks.items():
        for d in deps:
            if d not in tasks:
                errors.append(f"line {n}: {tid} depends on unknown task {d!r}")
        if status.startswith("✅") and not status[1:].strip():
            errors.append(f"line {n}: {tid} is done but cites no evidence")
        if status.startswith("🔄"):
            for d in deps:
                if d in tasks and not tasks[d][1].startswith("✅"):
                    errors.append(f"line {n}: {tid} is in progress but dependency {d} is not done")

    for n, line in enumerate(text.splitlines(), 1):
        for ref in REF_RE.findall(line):
            if ref not in tasks:
                errors.append(f"line {n}: reference to unknown task {ref}")

    if not tasks:
        errors.append("no tasks found")
    for e in errors:
        print(f"BACKLOG.md {e}", file=sys.stderr)
    return 1 if errors else 0


if __name__ == "__main__":
    sys.exit(main())
