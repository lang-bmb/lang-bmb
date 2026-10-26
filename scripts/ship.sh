#!/usr/bin/env bash
# Push the current branch only after the gate passes on exactly what is pushed
# (ADR-0039 §5; AUTONOMY.md §2). Twice a commit was pushed while the gate had
# failed because its output was piped and the exit code was lost: this script
# makes the gate's exit code the only thing that decides.
#
# The gate runs in a checkout of the commit alone (a worktree kept under
# .git/ship-gate with its own target/, so builds stay incremental), and that
# commit is what is pushed. Edits, untracked files and builds in the working
# tree while the gate runs cannot change the verdict: twice they did (a
# half-written ADR index, a target/ rebuilt under the benches).
#
#   scripts/ship.sh            # gate HEAD, then push it to the current branch
set -euo pipefail
cd "$(dirname "$0")/.."

branch=$(git rev-parse --abbrev-ref HEAD)
if [ "$branch" = "main" ]; then
  echo "ship: on main; work on a branch and fast-forward main after CI (AUTONOMY.md §5)" >&2
  exit 1
fi
head=$(git rev-parse HEAD)
if [ -n "$(git status --porcelain --untracked-files=no)" ]; then
  echo "ship: note: uncommitted changes are not part of $head and are not shipped" >&2
fi

gate="$(git rev-parse --path-format=absolute --git-common-dir)/ship-gate"
if [ ! -e "$gate/.git" ]; then
  git worktree prune
  git worktree add --detach "$gate" "$head" > /dev/null
fi
git -C "$gate" checkout --quiet --detach --force "$head"
git -C "$gate" clean --quiet -fdx -e target/

log=$(mktemp)
if ! "$gate/scripts/check.sh" > "$log" 2>&1; then
  tail -40 "$log" >&2
  echo "ship: gate failed at $head; nothing pushed (full log: $log)" >&2
  exit 1
fi
rm -f "$log"
echo "ship: gate passed at $head"
for delay in 0 2 4 8 16; do
  sleep "$delay"
  if git push origin "$head:refs/heads/$branch"; then
    git branch --quiet --set-upstream-to="origin/$branch" "$branch" 2> /dev/null || true
    exit 0
  fi
done
echo "ship: push failed after retries" >&2
exit 1
