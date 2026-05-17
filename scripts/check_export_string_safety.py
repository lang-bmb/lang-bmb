#!/usr/bin/env python3
"""
BMB @export -> String Safety Check

Scans all ecosystem lib.bmb files for @export pub fn -> String functions that
return static "" literals. These are P0 FFI crashes: the FFI caller calls
bmb_ffi_free_string(ret) which calls free() on the .rodata pointer → SIGABRT.

Fix: replace  { "" }  with  { str_repeat("", 1) }  to force heap allocation.

See: Cycle 2897 (pattern), Cycle 2901 (bmb-text/crypto), Cycle 2904 (bmb-json).

Usage:
    python3 scripts/check_export_string_safety.py
    python3 scripts/check_export_string_safety.py --verbose
    python3 scripts/check_export_string_safety.py --ci      # exit 1 on any issue
"""

import re
import sys
import os
import glob

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))

# Glob all lib.bmb files in the ecosystem
LIB_PATTERN = os.path.join(ROOT, "ecosystem", "*", "src", "lib.bmb")

STATIC_EMPTY_RE = re.compile(r'\{\s*""\s*\}')
EXPORT_RE = re.compile(r'^\s*@export\s*$')
STRING_RETURN_RE = re.compile(r'pub fn .+-> String')


def scan_file(path: str) -> list[tuple[int, str]]:
    """Return (line_number, line_text) for each P0 site in path."""
    issues = []
    lines = open(path, encoding="utf-8").readlines()
    in_export_string_fn = False

    for i, raw in enumerate(lines):
        lineno = i + 1
        stripped = raw.strip()

        if EXPORT_RE.match(stripped):
            # Look ahead for 'pub fn ... -> String'
            for j in range(i + 1, min(i + 4, len(lines))):
                next_stripped = lines[j].strip()
                if STRING_RETURN_RE.search(next_stripped):
                    in_export_string_fn = True
                    break
                elif next_stripped:
                    break

        if in_export_string_fn:
            if STATIC_EMPTY_RE.search(raw):
                issues.append((lineno, raw.rstrip()))
            # End-of-function heuristic: '};' on its own line
            if stripped == "};":
                in_export_string_fn = False

    return issues


def main():
    verbose = "--verbose" in sys.argv or "-v" in sys.argv
    ci_mode = "--ci" in sys.argv

    lib_files = sorted(glob.glob(LIB_PATTERN))
    if not lib_files:
        print(f"No lib.bmb files found matching {LIB_PATTERN}")
        sys.exit(1)

    total_issues = 0
    for path in lib_files:
        rel = os.path.relpath(path, ROOT)
        issues = scan_file(path)
        if issues:
            total_issues += len(issues)
            print(f"\nP0 FOUND in {rel} ({len(issues)} site(s)):")
            for lineno, text in issues:
                print(f"  L{lineno}: {text}")
            print(f"  Fix: replace '\"\"' with 'str_repeat(\"\", 1)'")
        elif verbose:
            print(f"OK  {rel}")

    print(f"\nExport-string safety: {len(lib_files)} files scanned, {total_issues} P0 site(s) found.")

    if total_issues > 0:
        print("FAIL: P0 FFI crash paths detected in @export -> String functions.")
        if ci_mode:
            sys.exit(1)
        return 1
    else:
        print("OK: No static empty-string returns in @export -> String functions.")
        return 0


if __name__ == "__main__":
    sys.exit(main())
