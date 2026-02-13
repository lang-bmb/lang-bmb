# Cycle 459: Automated Golden Test Runner + Build Chain Verification

## Date
2026-02-14

## Scope
Create automated golden test runner infrastructure and verify the full Stage 1 `build` command self-hosting chain works end-to-end.

## Philosophy Alignment
| Dimension | Score |
|-----------|-------|
| Core Mission Fit | 5/5 |
| Scope Boundaries | 5/5 |
| Architecture Patterns | 5/5 |
| Dependency Direction | 5/5 |

## Research Summary
- Analyzed existing test infrastructure: `bootstrap/tests/tests.txt` (5 test files, 821 total tests), `scripts/run-bootstrap-tests.sh` (automated runner using Stage 1)
- Found golden tests (`tests/bootstrap/test_golden_*.bmb`) have NO automated runner — manually verified each cycle
- Discovered bootstrap compiler already has full `build` command with `system()` calls to opt/clang
- Verified Stage 1 `build` command can self-host: compiles compiler.bmb to native binary that produces identical IR

## Implementation
### Files Created
1. **`scripts/run-golden-tests.sh`** — Automated golden test runner:
   - Reads manifest from `tests/bootstrap/golden_tests.txt`
   - Auto-detects Stage 1 binary from multiple locations
   - 5-step pipeline per test: Stage 1 compile → opt -O2 → llc -O3 → gcc → run
   - Compares output against expected value
   - Supports `--verbose`, `--json` for CI, `--build-stage1` to build Stage 1 first
   - Timeout protection at each step (compile/opt/link/run)

2. **`tests/bootstrap/golden_tests.txt`** — Golden test manifest:
   - 10 test entries with expected output values
   - Format: `filename|expected_output`

### Verification Performed
1. **Golden test automation**: All 10/10 tests pass in 4.7 seconds
2. **Stage 1 `build` self-hosting**: Stage 1 `build` → native binary → produces identical IR (68,624 lines)
3. **Full build chain fixed point**:
   - Stage 1 `build` → Stage 2 binary (540KB)
   - Stage 2 `build` → Stage 3 binary
   - Stage 2 IR == Stage 3 IR (fixed point verified)
4. **Self-built golden tests**: 10/10 pass using self-built Stage 1

### Key Findings
1. **Bootstrap is fully self-hosting via `build` command**: Stage 1 can produce Stage 2 as a native binary using `system()` to invoke opt/clang. No Rust compiler needed.
2. **Build chain achieves fixed point**: S1→S2→S3 all produce identical IR, confirming correctness of the `build` command pipeline.
3. **Minor issue**: `opt -passes='default<O3>,scalarizer'` fails on Windows (first attempt), falls back to `opt -O3` (succeeds). The "system cannot find the file specified" error is from the first attempt.

## Test Results
| Test | Status |
|------|--------|
| Rust tests | 5,229 passed |
| Bootstrap Stage 1 | Built successfully |
| Stage 1 == Stage 2 | Fixed point verified (68,624 lines) |
| Golden tests (automated) | 10/10 PASS (4,735ms) |
| Stage 1 `build` self-host | Success (540KB binary) |
| Build chain fixed point | S1→S2→S3 verified |
| Self-built golden tests | 10/10 PASS |

## Evaluation
| Criterion | Score | Notes |
|-----------|-------|-------|
| Correctness | 10/10 | All tests pass, build chain fixed point verified, golden runner produces correct results |
| Architecture | 9/10 | Clean script design with manifest-driven tests, auto-detection of Stage 1 |
| Philosophy Alignment | 9/10 | Directly supports bootstrap self-hosting verification, reduces manual work |
| Test Quality | 9/10 | Automated 10 golden tests + verified full build chain |
| Documentation | 8/10 | Script has usage docs, manifest is self-documenting |
| Code Quality | 9/10 | Consistent with existing scripts, proper error handling and fallbacks |
| **Average** | **9.0/10** | |

## Issues & Improvements
| # | Severity | Description | Action |
|---|----------|-------------|--------|
| I-01 | L | `opt -passes='default<O3>,scalarizer'` fails on Windows — benign, fallback works | Investigate root cause or fix command quoting |
| I-02 | L | Golden test runner not yet integrated into `quick-check.sh` | Add as Step 3 in future |
| I-03 | L | No CI workflow for golden tests yet | Add golden-tests step to `.github/workflows/` |
| I-04 | M | Roadmap drift from original plan — needs update | Update roadmap in next cycle |

## Next Cycle Recommendation
- Update ROADMAP-452-471.md to reflect actual progress (Cycles 454-459)
- Consider: expand bootstrap to compile more diverse BMB programs beyond compiler.bmb
- OR: Performance optimization — investigate the 6x speed gap between bootstrap and Rust
- OR: Add golden test runner to CI pipeline
