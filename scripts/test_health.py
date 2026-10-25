#!/usr/bin/env python3
"""Rules of scripts/health.py (ADR-0039): fixed rules, ratchets and budgets."""
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import health  # noqa: E402

GOOD = {
    "V1_untested_diag_codes": 0, "V3_unproved_defs": 0, "V4_rlimit_total": 1000,
    "V5_rlimit_headroom": 0.01, "V6_deterministic_verify": True, "V7_unstable_queries": 0.0, "S1_external_crates": 0,
    "S2_external_processes": ["clang", "z3"], "S3_runtime_libs_outside_libc": [],
    "S4_stage0_rust_lines": 100, "S5_tests_bound_to_rust_api": 30, "S6_selfhost_bmb_lines": 0,
    "P1_runnable_programs": 6, "P2_bmb_lines": 3000, "P3_errors_with_help": 0.5,
    "B1_baseline_binary_bytes": 100000, "O1_adr_index_mismatch": 0, "O2_soundness_fuzzers": 6,
}


def bad(changes):
    m = dict(GOOD, **changes)
    return health.judge(m, GOOD)


def main():
    assert bad({}) == ([], []), bad({})
    # fixed rules fail whatever the baseline says
    assert bad({"S1_external_crates": 1})[0]
    assert bad({"V3_unproved_defs": 2})[0]
    assert bad({"V5_rlimit_headroom": 0.5})[0]
    assert bad({"V7_unstable_queries": 0.026})[0]  # strictly below Mariposa's 2.6%
    assert bad({"V7_unstable_queries": 0.025}) == ([], [])
    assert bad({"S3_runtime_libs_outside_libc": ["libm.so.6"]})[0]
    # a new external process needs an accepted baseline (and an ADR)
    assert bad({"S2_external_processes": ["clang", "cvc5", "z3"]})[0]
    # proof cost: within the 10% budget passes, above fails, lower is an improvement
    assert bad({"V4_rlimit_total": 1100}) == ([], [])
    assert bad({"V4_rlimit_total": 1101})[0]
    assert bad({"V4_rlimit_total": 900}) == ([], ["V4_rlimit_total: 1000 -> 900"])
    # ratchets
    assert bad({"P1_runnable_programs": 5})[0]
    assert bad({"P3_errors_with_help": 0.4})[0]
    assert bad({"S5_tests_bound_to_rust_api": 31})[0]
    assert bad({"S5_tests_bound_to_rust_api": 29}) == ([], ["S5_tests_bound_to_rust_api: 30 -> 29"])
    # binary size: 5% budget
    assert bad({"B1_baseline_binary_bytes": 105001})[0]
    assert bad({"B1_baseline_binary_bytes": 105000}) == ([], [])
    # report-only metrics never fail
    assert bad({"S4_stage0_rust_lines": 10**6}) == ([], [])
    print("health rules: ok")


if __name__ == "__main__":
    main()
