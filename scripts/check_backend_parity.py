#!/usr/bin/env python3
"""
BMB Backend Parity Check

Compares runtime function registrations between the inkwell backend (llvm.rs)
and the text IR backend (llvm_text.rs) to catch Rule 7 violations before they
cause runtime divergence.

Inkwell backend registers functions via:  add_function("bmb_xxx", ...)
Text IR backend declares functions via:   declare ... @bmb_xxx(...)

Usage:
    python3 scripts/check_backend_parity.py
    python3 scripts/check_backend_parity.py --verbose
    python3 scripts/check_backend_parity.py --ci      # exit 1 on mismatch
"""

import re
import sys
import os

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
INKWELL = os.path.join(ROOT, "bmb", "src", "codegen", "llvm.rs")
TEXT = os.path.join(ROOT, "bmb", "src", "codegen", "llvm_text.rs")

# Known acceptable differences between backends.
#
# INKWELL_ONLY_EXPECTED: inkwell registers these but text IR uses different names
# or different mechanisms (e.g., no-prefix aliases, intrinsics, module names).
INKWELL_ONLY_EXPECTED = {
    # Module/function marker names — not runtime functions
    "bmb_program", "bmb_user_main",
    # Lifecycle — text IR omits explicit declare (uses different init path)
    "bmb_init_runtime", "bmb_arena_destroy",
    # Print/read — text IR uses short names without bmb_ prefix
    "bmb_println_i64", "bmb_print_i64", "bmb_println_f64", "bmb_print_f64",
    "bmb_println_str", "bmb_read_int",
    # Scalar math — text IR uses short names (abs, min, max)
    "bmb_abs", "bmb_min", "bmb_max",
    # Memory intrinsics — text IR uses store_i64/load_i64 without prefix
    "bmb_load_i64", "bmb_store_i64",
    # Type coercions — text IR uses LLVM instructions directly or short names
    "bmb_i64_to_f64", "bmb_f64_to_i64",
    # Vec ops — text IR uses short names (vec_new, vec_get, etc.)
    "bmb_vec_new", "bmb_vec_push", "bmb_vec_get", "bmb_vec_set",
    "bmb_vec_len", "bmb_vec_pop", "bmb_vec_free", "bmb_vec_with_capacity",
    "bmb_vec_cap", "bmb_vec_clear",
    # CLI args — not present in text IR
    "bmb_arg_count", "bmb_get_arg",
    # Misc — not present in text IR
    "bmb_assert", "bmb_box_new_i64", "bmb_channel_free", "bmb_char_to_string",
    # String concat variants — text IR uses concat/concat3 only
    "bmb_string_concat5", "bmb_string_concat7",
}

# TEXT_ONLY_EXPECTED: text IR declares these but inkwell handles them via different
# mechanisms (intrinsics, param_set heuristics, or unimplemented in inkwell).
TEXT_ONLY_EXPECTED = {
    # File system ops — inkwell does not implement these
    "bmb_file_size", "bmb_file_exists", "bmb_getcwd", "bmb_is_dir",
    "bmb_mkdir", "bmb_readdir", "bmb_remove_file", "bmb_rmdir",
    "bmb_exec_output",
    # Timing
    "bmb_time_ms",
    # Panic handlers — inkwell uses different intrinsic path
    "bmb_panic_bounds", "bmb_panic_divzero",
    # Arc reference counting — inkwell does not implement these
    "bmb_arc_new", "bmb_arc_clone", "bmb_arc_drop", "bmb_arc_get",
    "bmb_arc_strong_count",
    # Power — inkwell uses LLVM pow intrinsic directly
    "bmb_pow",
    # StringBuilder range push — inkwell has different implementation
    "bmb_sb_push_range",
    # String ops where inkwell uses different function names
    "bmb_string_new", "bmb_string_char_at",
}


def extract_inkwell_fns(path: str) -> set[str]:
    """Extract bmb_xxx function names from llvm.rs.

    Captures two patterns:
    1. Direct: add_function("bmb_xxx", ...)
    2. Via reg_str_fn! macro: reg_str_fn!("bmb_name", "bmb_xxx", ...) — 2nd arg is C ABI name
    """
    names = set()
    # Any "bmb_xxx" string literal (add_function direct + reg_str_fn 2nd arg + others)
    pattern = re.compile(r'"(bmb_[a-z0-9_]+)"')
    with open(path, encoding="utf-8") as f:
        for line in f:
            stripped = line.strip()
            # Skip comment lines
            if stripped.startswith("//"):
                continue
            for m in pattern.finditer(line):
                names.add(m.group(1))
    return names


def extract_text_fns(path: str) -> set[str]:
    """Extract bmb_xxx function names from declare @bmb_xxx lines."""
    names = set()
    # Match:  declare ... @bmb_xxx( ...
    pattern = re.compile(r'declare[^@]+@(bmb_[a-z0-9_]+)\(')
    with open(path, encoding="utf-8") as f:
        for line in f:
            for m in pattern.finditer(line):
                names.add(m.group(1))
    return names


def main():
    verbose = "--verbose" in sys.argv or "-v" in sys.argv
    ci_mode = "--ci" in sys.argv

    inkwell = extract_inkwell_fns(INKWELL)
    text = extract_text_fns(TEXT)

    inkwell_only = (inkwell - text) - INKWELL_ONLY_EXPECTED
    text_only = (text - inkwell) - TEXT_ONLY_EXPECTED
    shared = inkwell & text

    print(f"BMB Backend Parity Report")
    print(f"  inkwell registered : {len(inkwell)}")
    print(f"  text declared      : {len(text)}")
    print(f"  shared             : {len(shared)}")
    print(f"  inkwell-only (WARN): {len(inkwell_only)}")
    print(f"  text-only    (WARN): {len(text_only)}")

    if inkwell_only:
        print(f"\nIn inkwell but NOT in text IR (Rule 7 violation risk):")
        for fn in sorted(inkwell_only):
            print(f"  - {fn}")

    if text_only:
        print(f"\nIn text IR but NOT in inkwell (Rule 7 violation risk):")
        for fn in sorted(text_only):
            print(f"  - {fn}")

    if verbose and not inkwell_only and not text_only:
        print(f"\nShared functions ({len(shared)}):")
        for fn in sorted(shared):
            print(f"  {fn}")

    if inkwell_only or text_only:
        print(f"\nPARITY FAIL: {len(inkwell_only) + len(text_only)} mismatch(es) found.")
        print("Fix: ensure each bmb_xxx function is registered in BOTH llvm.rs and llvm_text.rs")
        if ci_mode:
            sys.exit(1)
        return 1
    else:
        print(f"\nPARITY OK: Both backends match on {len(shared)} bmb_* functions.")
        return 0


if __name__ == "__main__":
    sys.exit(main())
