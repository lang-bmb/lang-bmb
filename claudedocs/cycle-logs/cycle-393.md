# Cycle 393: Proven facts extraction from verification

## Date
2026-02-13

## Scope
Implement the TODO at verify/incremental.rs:280 — extract proven facts from CIR function contracts when verification succeeds.

## Philosophy Alignment
| Dimension | Score |
|-----------|-------|
| Core Mission Fit | 5/5 |
| Scope Boundaries | 5/5 |
| Architecture Patterns | 5/5 |
| Dependency Direction | 5/5 |

## Implementation

### Core Change
- Updated `witness_to_proof_result()` signature to accept `&CirFunction` alongside `&ProofWitness`
- When outcome is `Verified`, extracts preconditions as `ProofFact` with `ProofEvidence::Precondition`
- Extracts postconditions as `ProofFact` with `ProofEvidence::SmtProof`
- All facts scoped to `ProofScope::Function(func_id)`
- Non-verified outcomes produce empty proven_facts (as before)

### Tests (6 new)
| Test | Description |
|------|-------------|
| test_proven_facts_verified_with_contracts | Verified + contracts → 2 facts |
| test_proven_facts_verified_no_contracts | Verified + no contracts → 0 facts |
| test_proven_facts_failed_no_facts | Failed + contracts → 0 facts |
| test_proven_facts_precondition_evidence | Pre-facts have Precondition evidence |
| test_proven_facts_postcondition_evidence | Post-facts have SmtProof evidence |
| test_proven_facts_scope_is_function | All facts scoped to correct function |

## Test Results
- Unit tests: 2181 passed (+6)
- Main tests: 15 passed
- Integration tests: 2179 passed
- Gotgan tests: 23 passed
- **Total: 4398 tests — ALL PASSING**
- Clippy: PASS (0 warnings)
- Build: SUCCESS

## Evaluation
| Criterion | Score | Notes |
|-----------|-------|-------|
| Correctness | 10/10 | All tests pass, TODO resolved |
| Architecture | 10/10 | Uses existing ProofFact/ProofScope/ProofEvidence types |
| Philosophy Alignment | 10/10 | Contract verification is core to BMB |
| Test Quality | 10/10 | 6 tests covering all outcome paths |
| Code Quality | 10/10 | Clean, follows existing patterns |
| **Average** | **10.0/10** | |

## Next Cycle Recommendation
- Cycle 394: LLVM codegen unit tests — builtins + gen_function_body
