# Cycle 261: Verification Fallback Soundness Fix

## Date
2026-02-12

## Scope
Fix silent fallback from VerificationMode::Check to Trust when Z3 solver is unavailable. Previously, Check mode with no solver returned ALL functions as "verified", enabling unsound proof-guided optimizations.

## Philosophy Alignment
| Dimension | Score |
|-----------|-------|
| Core Mission Fit | 5/5 |
| Scope Boundaries | 5/5 |
| Architecture Patterns | 5/5 |
| Dependency Direction | 5/5 |

## Research Summary
- `VerificationMode::Check` with unavailable Z3: returned all functions as verified (Trust behavior)
- `VerificationMode::Warn` with unavailable Z3: correctly returned empty set
- Check mode's behavior was a soundness violation — contracts used for optimization without verification
- Also found counterexample formatting used `{:?}` (debug) instead of `{}` (Display)

## Implementation

### Build Module (`bmb/src/build/mod.rs`)
1. **Soundness fix**: Check mode + no solver now returns `HashSet::new()` (no verified functions)
   - Previously: `cir.functions.iter().map(|f| f.name.clone()).collect()` (all functions "verified")
   - Now: Empty set, no proof-guided optimizations applied
2. **Warning always shown**: Changed from verbose-only to always emit warning via `eprintln!`
3. **Counterexample formatting**: Changed `{:?}` to `{}` for human-readable Display output

### Unit Tests
Added 3 new tests:
- `test_verification_mode_none_skips_optimizations`
- `test_verification_mode_check_is_not_trust`
- `test_verification_mode_builder`

## Test Results
- Standard tests: 3327 / 3327 passed (+3 from 3324)
- Clippy: CLEAN (0 errors)
- Build: SUCCESS

## Evaluation
| Criterion | Score | Notes |
|-----------|-------|-------|
| Correctness | 10/10 | Closes soundness hole |
| Architecture | 10/10 | Aligns Check with Warn behavior |
| Philosophy Alignment | 10/10 | Performance safety = correctness |
| Test Quality | 9/10 | Unit tests verify mode semantics; integration testing requires Z3 |
| Code Quality | 10/10 | Minimal, focused fix |
| **Average** | **9.8/10** | |

## Issues & Improvements
| # | Severity | Description | Action |
|---|----------|-------------|--------|
| I-01 | L | No integration test for Z3-unavailable path | Requires Z3 mock, low priority |

## Next Cycle Recommendation
- Additional compiler quality improvements
- Closure capture analysis (MIR lowering)
- WASM memory growth
