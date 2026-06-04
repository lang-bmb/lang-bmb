#!/usr/bin/env python3
"""
verify-vc verdict regression harness (T-REG)

Re-runs `bootstrap/compiler.exe verify-vc` over every tracked acceptance
fixture in tests/verify-vc/ and diffs the emitted verdicts against the oracle
table in tests/verify-vc/EXPECTED.md. Catches a *silent verdict regression* —
the gap that the 3-Stage Fixed Point does NOT close (FP guarantees only
self-consistency of the self-compile, never that `cf_pow2 -> unsupported` or
`find_colon_weak -> refuted` is preserved across compiler changes).

EXPECTED.md is the single source of truth: each `## <file>.bmb` section's
markdown table gives `| function | expected verdict | ... |`. The verdict cell's
first whitespace token is the verdict (prose annotations like
"verified (retained-latent ...)" are stripped); the full token is matched
exactly, so `unsupported` never silently matches `unsupported_recursion`.

Scope honesty: this guards the acceptance *fixtures* (isomorph proxies of real
compiler.bmb scanner patterns), NOT the live corpus verdicts (v154/r17/ur8).
The corpus is compiler.bmb itself, which moves every cycle, so it cannot be
pinned cheaply.

Usage:
    python3 scripts/verify_vc_regression.py            # machine output (JSON), default
    python3 scripts/verify_vc_regression.py --human     # human-readable report

Exit code: 0 = all fixtures match the oracle, 1 = any regression / mismatch.
"""

import json
import sys
import re
import argparse
import subprocess
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
TESTS_DIR = REPO / "tests" / "verify-vc"
EXPECTED_MD = TESTS_DIR / "EXPECTED.md"
COMPILER = REPO / "bootstrap" / "compiler.exe"

# Full verdict tokens emitted by verify-vc. Matched exactly — a prefix match
# would let an `unsupported` <-> `unsupported_recursion` regression slip past.
VALID_VERDICTS = {
    "verified",
    "refuted",
    "unsupported",
    "unsupported_recursion",
    "unknown",
}


def parse_expected(md_path):
    """Parse EXPECTED.md -> {filename: {function: verdict}}.

    Raises SystemExit (loud) on any malformed row — an unparseable oracle is a
    failure, never a silent skip.
    """
    if not md_path.exists():
        raise SystemExit(f"FAIL: oracle not found: {md_path}")
    sections = {}
    cur = None
    sep_re = re.compile(r"^[-:\s|]+$")
    for raw in md_path.read_text(encoding="utf-8").splitlines():
        line = raw.rstrip()
        # A level-2 header naming a fixture file starts a section.
        if line.startswith("## ") and not line.startswith("###"):
            toks = line[3:].split()
            if toks and toks[0].endswith(".bmb"):
                cur = toks[0]
                if cur in sections:
                    raise SystemExit(f"FAIL: EXPECTED.md: duplicate section '{cur}'")
                sections[cur] = {}
            else:
                cur = None  # prose section, not a fixture
            continue
        if line.startswith("###"):
            cur = None  # sub-section (e.g. caveats) — not a fixture table
            continue
        if cur is None or not line.startswith("|"):
            continue
        if sep_re.match(line):
            continue  # markdown table separator row
        cells = [c.strip() for c in line.strip().strip("|").split("|")]
        if len(cells) < 2:
            continue
        fn = cells[0]
        if fn.lower() == "function":
            continue  # table header row
        verdict_cell = cells[1].split()
        verdict = verdict_cell[0].lower() if verdict_cell else ""
        if verdict not in VALID_VERDICTS:
            raise SystemExit(
                f"FAIL: EXPECTED.md [{cur}] row '{fn}': "
                f"unrecognized verdict token '{verdict}' "
                f"(expected one of {sorted(VALID_VERDICTS)})"
            )
        if fn in sections[cur]:
            raise SystemExit(f"FAIL: EXPECTED.md [{cur}]: duplicate row '{fn}'")
        sections[cur][fn] = verdict
    return sections


def run_verify(bmb_path):
    """Run verify-vc and return its list of result dicts. Raises on hard error."""
    proc = subprocess.run(
        [str(COMPILER), "verify-vc", str(bmb_path)],
        capture_output=True,
        text=True,
    )
    if proc.returncode != 0:
        raise SystemExit(
            f"FAIL: verify-vc exited {proc.returncode} on {bmb_path}\n"
            f"stderr: {proc.stderr.strip()}"
        )
    try:
        data = json.loads(proc.stdout)
    except json.JSONDecodeError as e:
        raise SystemExit(f"FAIL: verify-vc on {bmb_path} produced non-JSON output: {e}\n{proc.stdout[:400]}")
    return data.get("results", [])


def check_file(fname, expected):
    """Diff one fixture's emitted verdicts against the oracle. Returns failures."""
    bmb = TESTS_DIR / fname
    failures = []
    if not bmb.exists():
        return [f"{fname}: fixture file is in EXPECTED.md but missing on disk"]
    results = run_verify(bmb)
    got = {r["function"]: r for r in results}

    exp_fns = set(expected)
    got_fns = set(got)

    for fn in sorted(exp_fns - got_fns):
        failures.append(
            f"{fname}:{fn} - expected '{expected[fn]}' but function MISSING from verify-vc output"
        )
    for fn in sorted(got_fns - exp_fns):
        failures.append(
            f"{fname}:{fn} - verify-vc emitted '{got[fn]['verdict']}' but there is NO oracle row "
            f"(add it to EXPECTED.md)"
        )
    for fn in sorted(exp_fns & got_fns):
        ev = expected[fn]
        gv = got[fn]["verdict"]
        if ev != gv:
            failures.append(f"{fname}:{fn} - expected '{ev}' but got '{gv}' (verdict regression)")
            continue
        # Counterexample symmetry invariant (T-CX): refuted carries one, others don't.
        cex = (got[fn].get("counterexample") or "").strip()
        if gv == "refuted" and not cex:
            failures.append(
                f"{fname}:{fn} - refuted but counterexample MISSING/empty (T-CX invariant)"
            )
        if gv != "refuted" and cex:
            failures.append(
                f"{fname}:{fn} - verdict '{gv}' but carries a counterexample (should be absent)"
            )

    # Cheap backstop: a prose edit that silently drops/adds a row trips the count.
    if len(expected) != len(results):
        failures.append(
            f"{fname}: oracle has {len(expected)} rows but verify-vc returned {len(results)} results"
        )
    return failures


def main():
    ap = argparse.ArgumentParser(description="verify-vc verdict regression harness (T-REG)")
    ap.add_argument("--human", action="store_true", help="human-readable report (default: JSON)")
    args = ap.parse_args()

    if not COMPILER.exists():
        raise SystemExit(f"FAIL: compiler not found: {COMPILER} (build bootstrap/compiler.exe first)")

    expected = parse_expected(EXPECTED_MD)

    # Loud: every *_accept.bmb fixture on disk must have an oracle section.
    on_disk = {p.name for p in TESTS_DIR.glob("*_accept.bmb")}
    orphan_fixtures = sorted(on_disk - set(expected))

    all_failures = []
    checked = []
    for fname in sorted(expected):
        fails = check_file(fname, expected[fname])
        checked.append({"file": fname, "functions": len(expected[fname]), "failures": fails})
        all_failures.extend(fails)
    for f in orphan_fixtures:
        all_failures.append(f"{f}: fixture on disk has no section in EXPECTED.md")

    ok = not all_failures

    if args.human:
        for c in checked:
            mark = "OK " if not c["failures"] else "FAIL"
            print(f"[{mark}] {c['file']} ({c['functions']} functions)")
            for msg in c["failures"]:
                print(f"       - {msg}")
        for f in orphan_fixtures:
            print(f"[FAIL] {f}: fixture on disk has no section in EXPECTED.md")
        print()
        print("RESULT: PASS" if ok else f"RESULT: FAIL ({len(all_failures)} issue(s))")
    else:
        print(json.dumps({
            "command": "verify-vc-regression",
            "ok": ok,
            "checked": checked,
            "orphan_fixtures": orphan_fixtures,
            "failure_count": len(all_failures),
        }))

    sys.exit(0 if ok else 1)


if __name__ == "__main__":
    main()
