#!/bin/bash
# BMB submodule pointer regression check
#
# Ensures that every submodule pointer in the parent repo references a
# commit that exists on its upstream remote. An upstream-missing pointer
# makes `actions/checkout@v4 submodules: recursive` fail in CI with:
#   fatal: remote error: upload-pack: not our ref <hash>
#
# Origin: Cycles 2427-2428 discovered that 4 submodules (benchmark-bmb,
# gotgan, tree-sitter-bmb, vscode-bmb) had local-ahead commits that were
# never pushed. All 11+ workflows using `submodules: recursive` failed
# at checkout. See claudedocs/cycle-logs/cycle-2427.md, cycle-2428.md.
#
# Usage:
#   ./scripts/verify-submodules.sh
#
# Exits non-zero if any pointer is missing from its upstream remote.

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$PROJECT_ROOT"

if [[ ! -f .gitmodules ]]; then
    echo "verify-submodules.sh: no .gitmodules — nothing to check"
    exit 0
fi

fail=0
missing=()

echo "Verifying submodule pointers against upstream remotes..."

# Iterate over each submodule path declared in .gitmodules.
paths="$(git config --file .gitmodules --get-regexp '^submodule\..*\.path$' | awk '{print $2}')"

for path in $paths; do
    url="$(git config --file .gitmodules --get "submodule.$path.url")"
    if [[ -z "$url" ]]; then
        echo "  [WARN] $path: no URL in .gitmodules — skip"
        continue
    fi

    # Parent pointer hash (status may prefix with +/- which we strip).
    pointer="$(git submodule status -- "$path" 2>/dev/null | awk '{print $1}' | tr -d '+-')"
    if [[ -z "$pointer" ]]; then
        echo "  [WARN] $path: no pointer (not initialised?) — skip"
        continue
    fi

    # A single-commit `git fetch --dry-run` is the cleanest portability check:
    # it validates fetchability without mutating local state. Quiet on success.
    if git -c protocol.version=2 fetch --dry-run --no-tags --depth=1 "$url" "$pointer" >/dev/null 2>&1; then
        echo "  [OK]   $path @ ${pointer:0:10}"
    else
        echo "  [FAIL] $path @ ${pointer:0:10} — not fetchable from $url"
        missing+=("$path @ $pointer ($url)")
        fail=1
    fi
done

if [[ $fail -ne 0 ]]; then
    echo ""
    echo "::error::One or more submodule pointers are not present in upstream."
    echo "  actions/checkout@v4 submodules: recursive will fail in CI."
    echo ""
    echo "Affected pointers:"
    for m in "${missing[@]}"; do
        echo "  - $m"
    done
    echo ""
    echo "Remediation:"
    echo "  (a) cd <path> && git push origin main  (if fast-forward ahead)"
    echo "  (b) git submodule update --remote <path>  (if you want to follow upstream)"
    echo "  (c) git rm <path>  (if the submodule is no longer used)"
    exit 1
fi

echo ""
echo "All submodule pointers are fetchable from their upstream remotes."
