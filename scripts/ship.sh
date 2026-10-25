#!/usr/bin/env bash
# Push the current branch only after the gate passes on exactly what is pushed
# (ADR-0039 §5; AUTONOMY.md §2). Twice a commit was pushed while the gate had
# failed because its output was piped and the exit code was lost: this script
# makes the gate's exit code the only thing that decides.
#
#   scripts/ship.sh            # gate, then push the current branch
set -euo pipefail
cd "$(dirname "$0")/.."

if [ -n "$(git status --porcelain --untracked-files=no)" ]; then
  echo "ship: uncommitted changes; commit first so the gate checks what is pushed" >&2
  exit 1
fi
branch=$(git rev-parse --abbrev-ref HEAD)
if [ "$branch" = "main" ]; then
  echo "ship: on main; work on a branch and fast-forward main after CI (AUTONOMY.md §5)" >&2
  exit 1
fi
head=$(git rev-parse HEAD)
log=$(mktemp)
if ! ./scripts/check.sh > "$log" 2>&1; then
  tail -40 "$log" >&2
  echo "ship: gate failed at $head; nothing pushed (full log: $log)" >&2
  exit 1
fi
if [ "$(git rev-parse HEAD)" != "$head" ] || [ -n "$(git status --porcelain --untracked-files=no)" ]; then
  echo "ship: the tree changed while the gate ran; nothing pushed" >&2
  exit 1
fi
rm -f "$log"
echo "ship: gate passed at $head"
for delay in 0 2 4 8 16; do
  sleep "$delay"
  if git push -u origin "$branch"; then exit 0; fi
done
echo "ship: push failed after retries" >&2
exit 1
