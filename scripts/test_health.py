#!/usr/bin/env python3
"""Rules of scripts/health.py (ADR-0039): fixed rules, ratchets and budgets."""
import json
import re
import subprocess
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import health  # noqa: E402

GOOD = {
    "V1_untested_diag_codes": 0, "V3_unproved_defs": 0, "V4_rlimit_total": 1000,
    "V5_rlimit_headroom": 0.01, "V6_deterministic_verify": True, "V7_unstable_queries": 0.0,
    "V8_bench_rlimit_total": 5000, "V9_bench_defs_at_limit": 1, "V10_library_rlimit_total": 2000,
    "V11_sat_check_rlimit_total": 3000, "V12_sat_check_undecided": 10, "S1_external_crates": 0,
    "S2_external_processes": ["clang", "z3"], "S3_runtime_libs_outside_libc": [],
    "S4_stage0_rust_lines": 100, "S5_tests_bound_to_rust_api": 30, "S6_selfhost_bmb_lines": 0,
    "P1_runnable_programs": 6, "P2_bmb_lines": 3000, "P3_errors_with_help": 0.5,
    "B1_baseline_binary_bytes": 100000, "O1_adr_index_mismatch": 0, "O2_soundness_fuzzers": 6,
}


def bad(changes):
    m = dict(GOOD, **changes)
    return health.judge(m, GOOD)


def sat_check_cost():
    """V11 and V12 add up the cost fields of each `bmb lint --select
    W0901,W0905` summary (T2-12), over the programs of the gate test
    `proofs_are_not_vacuous` and `bench/game`."""
    progs = health.sat_check_programs()
    seen = []

    def fake_run(args, **kw):
        assert args[1:6] == ["lint", "--cache", args[3], "--select", "W0901,W0905"], args
        seen.append(args[6:])
        s = {"summary": True, "pre_rlimit": 2, "pre_undecided": 1, "reach_rlimit": 5, "reach_undecided": 0}
        return subprocess.CompletedProcess(args, 0, '{"code":"W0901"}\n' + json.dumps(s) + "\n", "")

    real = health.run
    health.run = fake_run
    try:
        m = health.sat_check_cost()
    finally:
        health.run = real
    assert seen == progs
    assert m["V11_sat_check_rlimit_total"] == 7 * len(progs)
    assert m["V12_sat_check_undecided"] == len(progs)
    assert m["_sat_check_rlimit"] == f"W0905 {5 * len(progs)} + W0901 {2 * len(progs)}"
    # The same programs as the gate test: every directory it reads, and the
    # game programs with every library, as T2-10 measured.
    rs = (health.ROOT / "compiler" / "tests" / "verify.rs").read_text()
    body = rs[rs.index("fn proofs_are_not_vacuous"):]
    body = body[: body.index("\n}\n")]
    dirs = {Path(f).parent.as_posix() for p in progs for f in p}
    for name in re.findall(r'"([a-z0-9_-]+)"', body):
        assert any(name in d.split("/") for d in dirs), name
    libs = [p[0] for p in progs if len(p) == 1 and p[0].startswith("lib/")]
    assert len(libs) >= 5
    game = [p for p in progs if p[0].startswith("bench/game/")]
    assert game and all(p[1:] == libs for p in game)
    assert any(p[0].startswith("bench/brb-encap/seed/") for p in progs)
    assert ["bench/m1b/seed/src/" + f.name for f in sorted((health.ROOT / "bench/m1b/seed/src").glob("*.bmb"))] in progs


def main():
    assert bad({}) == ([], []), bad({})
    # fixed rules fail whatever the baseline says
    assert bad({"S1_external_crates": 1})[0]
    assert bad({"V3_unproved_defs": 2})[0]
    assert bad({"V5_rlimit_headroom": 0.5})[0]
    assert bad({"V7_unstable_queries": 0.026})[0]  # strictly below Mariposa's 2.6%
    assert bad({"V7_unstable_queries": 0.025}) == ([], [])
    assert bad({"V8_bench_rlimit_total": 5501})[0]  # budget +10%
    assert bad({"V9_bench_defs_at_limit": 2})[0]  # may only go down
    assert bad({"V10_library_rlimit_total": 2201})[0]  # budget +10%
    assert bad({"V10_library_rlimit_total": 2200}) == ([], [])
    assert bad({"V9_bench_defs_at_limit": 0}) == ([], ["V9_bench_defs_at_limit: 1 -> 0"])
    # satisfiability checks (T2-12): rlimit within a 10% budget, undecided only down
    assert bad({"V11_sat_check_rlimit_total": 3300}) == ([], [])
    assert bad({"V11_sat_check_rlimit_total": 3301})[0]
    assert bad({"V12_sat_check_undecided": 11})[0]
    assert bad({"V12_sat_check_undecided": 9}) == ([], ["V12_sat_check_undecided: 10 -> 9"])
    sat_check_cost()
    # V5 is the worse of the corpus and the libraries (T2-09): a library
    # obligation near the limit fails the fixed rule as a corpus one does
    lib = health.with_library_headroom(dict(GOOD, _library_headroom=0.37))
    assert lib["V5_rlimit_headroom"] == 0.37 and "_library_headroom" not in lib
    assert health.judge(lib, GOOD)[0]
    assert health.with_library_headroom(dict(GOOD, _library_headroom=0.0))["V5_rlimit_headroom"] == 0.01
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
