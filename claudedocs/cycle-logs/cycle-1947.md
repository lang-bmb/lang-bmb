# Cycle 1947: bench.sh Windows fix + runtime rebuild
Date: 2026-03-22

## Inherited → Addressed
- Cycle 1946 clean

## Scope & Implementation
- Fixed bench.sh BUILD_DIR: `/tmp/bmb-bench` → `target/bench` (Windows path issue)
- Fixed bench.sh emit-ir: `-o $ir` → `-o $ir_base` (bmb adds .ll extension automatically)
- Fixed bench.sh TEMP: added `export TEMP="${TMPDIR:-/tmp}"` for Windows subshells
  - Root cause: clang "unable to make temporary file" when TEMP is unset in MSYS2 subshells
- Rebuilt bmb_runtime.o with new functions (now_ns, now_ms, sleep_ms, current_dir)
- Installed MSYS2 Python (mingw-w64-ucrt-x86_64-python) for benchmark statistics
- Verified: `tak` benchmark passes (BMB == C, 1.00x ratio)
- Full benchmark run started in background (310 benchmarks)

## Review & Resolution
- bench.sh build pipeline: emit-ir → opt -O3 → clang -O3 ✅
- tak benchmark: 29ms BMB vs 29ms C (PASS) ✅

## Carry-Forward
- Pending Human Decisions: None
- Discovered out-of-scope: None
- Next Recommendation: Cycle 1948 — analyze full benchmark results
