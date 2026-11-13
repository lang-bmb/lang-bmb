#!/usr/bin/env python3
"""What a gate run left in the tree (backlog INF-36).

  tree_state.py snapshot > FILE   the tree's state before the run
  tree_state.py left FILE         fail when files appeared or changed since

A test once wrote `x.bmb` into its working directory; the file stayed in the
gate's tree and was committed with the next change. The state is `git
status` with every untracked file listed (ignored ones are not: `target/`,
caches), so a file a test creates, changes or deletes shows, and what was
already uncommitted before the run does not. Standard library only.
"""
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent


def snapshot(root=ROOT):
    r = subprocess.run(["git", "status", "--porcelain", "--untracked-files=all"],
                       cwd=root, capture_output=True, text=True, check=True)
    return sorted(r.stdout.splitlines())


def left(before, root=ROOT):
    """Status lines that were not there before the run."""
    old = set(before)
    return [l for l in snapshot(root) if l not in old]


def main():
    args = sys.argv[1:]
    if args == ["snapshot"]:
        print("\n".join(snapshot()))
        return 0
    if len(args) == 2 and args[0] == "left":
        new = left(Path(args[1]).read_text().splitlines())
        for l in new:
            print(f"tree: the run left: {l}", file=sys.stderr)
        return 1 if new else 0
    raise SystemExit(__doc__)


if __name__ == "__main__":
    sys.exit(main())
