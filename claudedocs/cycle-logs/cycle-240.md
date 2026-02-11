# Cycle 240: MIR Proof-Guided Optimization & Format Tests

## Date
2026-02-11

## Scope
Add integration tests for MIR proof-guided optimization (ProvenFactSet, run_proof_guided_program), optimization pipeline, format_mir output, and ContractFact API.

## Philosophy Alignment
| Dimension | Score |
|-----------|-------|
| Core Mission Fit | 5/5 |
| Scope Boundaries | 5/5 |
| Architecture Patterns | 5/5 |
| Dependency Direction | 5/5 |

## Research Summary
- MIR has separate OptLevel (bmb::mir::OptLevel, not bmb::build::OptLevel)
- ProvenFactSet tracks nonzero via explicit Ne/0, not inferred from Gt/0
- d > 0 sets lower_bound to 1 but doesn't set nonzero flag
- format_mir outputs human-readable IR with @pure, @const attributes
- OptimizationPipeline::for_level(Debug) does 0 iterations
- ContractFact: VarCmp, NonNull, ReturnCmp, etc.

## Implementation

### Integration Tests (`bmb/tests/integration.rs`)
Added 21 new tests:

**MIR Format (5 tests)**
- `test_mir_format_simple_function`: Function name and types in output
- `test_mir_format_multiple_functions`: Multiple functions formatted
- `test_mir_format_with_contract`: Contract function formatting
- `test_mir_format_pure_function`: @pure attribute display
- `test_mir_format_if_else`: Branching structure in output

**MIR Lowering with Contracts (3 tests)**
- `test_mir_lower_preconditions_extracted`: Pre conditions extracted
- `test_mir_lower_postconditions_extracted`: Post conditions extracted
- `test_mir_lower_pure_attribute`: @pure → is_pure flag

**ProvenFactSet (4 tests)**
- `test_mir_proven_fact_set_from_preconditions`: pre x >= 0 → lower bound
- `test_mir_proven_fact_set_upper_bound`: pre x <= 100 → upper bound 100
- `test_mir_proven_fact_set_nonzero`: Explicit nonzero tracking
- `test_mir_proven_fact_set_lower_bound_implies_positive`: d > 0 → lower bound 1

**Optimization Pipeline (4 tests)**
- `test_mir_optimization_pipeline_debug`: 0 iterations for Debug
- `test_mir_optimization_pipeline_release`: Release pipeline runs
- `test_mir_constant_folding_pass`: 2+3 folded to 5
- `test_mir_dead_code_elimination_pass`: Unused binding eliminated

**Proof-Guided & Semantics (2 tests)**
- `test_mir_proof_guided_program_runs`: Proof-guided optimizer runs
- `test_mir_full_pipeline_preserves_semantics`: 2+3*4=14 correct

**ContractFact API (3 tests)**
- `test_mir_contract_fact_varcmp`: VarCmp creation
- `test_mir_contract_fact_nonnull`: NonNull creation
- `test_mir_contract_fact_return_cmp`: ReturnCmp creation

## Test Results
- Standard tests: 2950 / 2950 passed (+21 from 2929)
- Clippy: CLEAN (0 errors)
- Build: SUCCESS

## Evaluation
| Criterion | Score | Notes |
|-----------|-------|-------|
| Correctness | 10/10 | All tests pass |
| Architecture | 9/10 | Tests proof-guided optimization pipeline |
| Philosophy Alignment | 10/10 | Proof-guided optimization is core to BMB |
| Test Quality | 9/10 | Covers format, facts, pipeline, semantics |
| Code Quality | 9/10 | Fixed OptLevel namespace, nonzero semantics |
| **Average** | **9.4/10** | |

## Issues & Improvements
| # | Severity | Description | Action |
|---|----------|-------------|--------|
| I-01 | L | ProvenFactSet doesn't infer nonzero from Gt/0 | Design choice |
| I-02 | L | Individual proof passes (BCE, NCE, DCE) not tested E2E | Need complex programs |

## Next Cycle Recommendation
- Add Type system advanced inference integration tests
