#!/usr/bin/env python3
"""The backlog's files (INF-30).

  backlog.py archive   move done and cancelled rows, and all but the two
                       newest plans, out of dev-docs/BACKLOG.md
  backlog.py check     fail when `archive` would move something
  backlog.py size      characters of the active backlog (health O3)

dev-docs/BACKLOG.md is the active backlog: rows in progress, waiting or
blocked, and the two newest plans. Finished rows live in dev-docs/backlog/
under the heading they had, and the readers (check_backlog.py, progress.py,
health.py) read both through `text()`, so an ID stays unique and a reference
to a finished task still resolves. Standard library only.
"""
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
ACTIVE = ROOT / "dev-docs" / "BACKLOG.md"
ARCHIVE = ROOT / "dev-docs" / "backlog"
PLANS = "plans.md"
KEEP_PLANS = 2

ID = r"(?:\d|INF|T\d)[A-Z]?-\d{2}[a-z]?"
ROW_ID = re.compile(rf"^~{{0,2}}({ID})~{{0,2}}$")
CELL = re.compile(r"(?<!\\)\|")
HEADER = ["| ID | 태스크 | 완료 기준 | 의존 | 상태 |", "| --- | --- | --- | --- | --- |"]

# The archive file of a `##` section, first match wins.
GROUPS = [
    (r"^트랙", "tracks.md"),
    (r"^(0 착수|인프라)", "infra.md"),
    (r"^1[A-E] ", "stage1.md"),
    (r"^2단계", "stage2.md"),
    (r"^[345]단계", "stage3-5.md"),
]
TITLES = {
    "tracks.md": "트랙 T0~T4", "infra.md": "착수와 인프라", "stage1.md": "1단계",
    "stage2.md": "2단계", "stage3-5.md": "3~5단계", PLANS: "지난 실행 계획",
}


def cells(line):
    """Cells of a table line; `\\|` is a pipe inside a cell."""
    return [c.strip() for c in CELL.split(line.strip().strip("|"))]


def task(line):
    """(id, cancelled, status cell) of a task row, else None."""
    if not line.startswith("|"):
        return None
    c = cells(line)
    if len(c) != 5:
        return None
    m = ROW_ID.match(c[0])
    return (m.group(1), c[0].startswith("~"), c[4]) if m else None


def finished(line):
    t = task(line)
    return bool(t) and (t[1] or t[2].startswith("✅"))


def files(root=ROOT):
    """The active backlog, then the archive files in name order."""
    arch = root / "dev-docs" / "backlog"
    return [root / "dev-docs" / "BACKLOG.md"] + (sorted(arch.glob("*.md")) if arch.is_dir() else [])


def numbered(root=ROOT):
    """(file name, line number, line) of every backlog line."""
    for p in files(root):
        for n, line in enumerate(p.read_text(encoding="utf-8").splitlines(), 1):
            yield p.name, n, line


def text(root=ROOT):
    return "\n".join(line for _, _, line in numbered(root))


def group_of(heading):
    for pat, name in GROUPS:
        if re.match(pat, heading):
            return name
    return None


def split(active):
    """(new active lines, {archive file: [(heading, row)]}, old plan sections)."""
    out, moved, plans = [], {}, []
    top = sub = None
    plan = None  # lines of the plan section being read
    seen_plans = 0
    for line in active.splitlines():
        if line.startswith("## "):
            top, sub, plan = line[3:].strip(), None, None
        elif line.startswith("### "):
            sub, plan = line[4:].strip(), None
            if sub.startswith("실행 계획("):
                seen_plans += 1
                if seen_plans > KEEP_PLANS:
                    plan = [line]
                    plans.append(plan)
                    continue
        elif plan is not None:
            plan.append(line)
            continue
        name = group_of(top) if top else None
        if name and finished(line):
            moved.setdefault(name, []).append((sub or top, line))
            continue
        out.append(line)
    return out, moved, plans


def add_rows(existing, name, rows):
    """The archive file `name` with `rows` appended under their headings."""
    lines = existing.splitlines() if existing else [f"# 백로그 보관: {TITLES[name]}", "",
        "> 완료·취소된 행이다. 활성 백로그는 `dev-docs/BACKLOG.md`, 옮기는 도구는 `scripts/backlog.py archive`.",
        "> 완료된 행의 근거를 고칠 때는 여기서 고친다."]
    for heading, row in rows:
        mark = f"## {heading}"
        if mark not in lines:
            lines += ["", mark, ""] + HEADER + [row]
            continue
        i = lines.index(mark) + 1
        while i < len(lines) and not lines[i].startswith("## "):
            i += 1
        while not lines[i - 1].strip():
            i -= 1
        lines.insert(i, row)
    return "\n".join(lines) + "\n"


def add_plans(existing, plans):
    """Plans are newest first, as in the active backlog."""
    head = [f"# 백로그 보관: {TITLES[PLANS]}", "",
            "> 활성 백로그(`dev-docs/BACKLOG.md` \"지금 할 일\")에는 최근 둘만 둔다. 새것이 위다.", ""]
    old = existing.splitlines()[len(head):] if existing else []
    body = []
    for p in plans:
        while p and not p[-1].strip():
            p = p[:-1]
        body += ["#" + p[0][1:]] + p[1:] + [""]  # `###` becomes `##`
    return "\n".join(head + body + old).rstrip("\n") + "\n"


def archive(root=ROOT, write=True):
    """Number of rows and plans moved (or that would be)."""
    active = root / "dev-docs" / "BACKLOG.md"
    arch = root / "dev-docs" / "backlog"
    out, moved, plans = split(active.read_text(encoding="utf-8"))
    n = sum(len(r) for r in moved.values()) + len(plans)
    if not write or not n:
        return n
    arch.mkdir(exist_ok=True)
    for name, rows in moved.items():
        p = arch / name
        p.write_text(add_rows(p.read_text(encoding="utf-8") if p.exists() else "", name, rows), encoding="utf-8")
    if plans:
        p = arch / PLANS
        p.write_text(add_plans(p.read_text(encoding="utf-8") if p.exists() else "", plans), encoding="utf-8")
    active.write_text("\n".join(out) + "\n", encoding="utf-8")
    return n


def size(root=ROOT):
    return len((root / "dev-docs" / "BACKLOG.md").read_text(encoding="utf-8"))


def main():
    cmd = sys.argv[1] if len(sys.argv) > 1 else "check"
    if cmd == "archive":
        print(f"moved {archive()} rows and plans")
        return 0
    if cmd == "size":
        print(size())
        return 0
    if cmd == "check":
        n = archive(write=False)
        if n:
            print(f"backlog: {n} finished rows or old plans in BACKLOG.md: run `scripts/backlog.py archive`",
                  file=sys.stderr)
        return 1 if n else 0
    raise SystemExit(__doc__)


if __name__ == "__main__":
    sys.exit(main())
