# Cycle 17: Fix Bootstrap Test Runner Hang

## Date
2026-02-07

## Scope
Fix the bootstrap test runner script (scripts/run-bootstrap-tests.sh) which hangs on Windows MSYS2/MinGW when running test executables.

## Philosophy Alignment
| Dimension | Score |
|-----------|-------|
| Core Mission Fit | 3/5 |
| Scope Boundaries | 5/5 |
| Architecture Patterns | 5/5 |
| Dependency Direction | 4/5 |

## Research Summary
- **Root cause**: `$()` command substitution hangs on MSYS2/MinGW when capturing stdout from Windows executables (known MSYS2 issue with pipe buffering)
- **Symptoms**: Script hangs at "Running parser_test..." — compilation succeeds but `OUTPUT=$("$EXE_FILE" 2>&1)` never returns
- **Proof**: Same test executable works perfectly with file redirection (`"$EXE_FILE" > file.txt`)
- **Fix**: Replace `$()` capture with file-based output capture (`> "$OUT_FILE"`)

## Implementation
- Replaced `OUTPUT=$("$EXE_FILE" 2>&1)` with `"$EXE_FILE" > "$OUT_FILE" 2>&1`
- All output parsing now reads from file instead of variable
- Added per-step timeouts (compile: 60s, opt: 60s, link: 60s, run: 30s)
- Added `timeout` wrapper to all external commands
- Added TOTAL counter for better reporting
- Improved error messages (FAIL with step name)
- Fixed arithmetic with `|| true` to prevent `set -e` exit on zero increment

## Test Results
- Bootstrap test runner: 5/5 PASS (821 total: 257+280+264+10+10)
- Rust tests: 334/334 passed
- Script execution: completes in ~10s (was: hung indefinitely)

## Evaluation
| Criterion | Score | Notes |
|-----------|-------|-------|
| Correctness | 10/10 | All 5 test suites pass |
| Architecture | 9/10 | Proper root cause fix, not workaround |
| Philosophy Alignment | 8/10 | Infrastructure, not performance |
| Test Quality | 10/10 | 821 BMB + 334 Rust tests verified |
| Documentation | 8/10 | Root cause documented in comments |
| Code Quality | 9/10 | Clean, portable |
| **Average** | **9.0/10** | |

## Issues
- I-01 (L): `timeout` command may not be available on all platforms (fallback needed for macOS)

## Next Cycle Recommendation
Add concurrency test files to cargo test integration.
