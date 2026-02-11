# Cycle 232: PIR Module Integration Tests

## Date
2026-02-11

## Scope
Add integration tests for PIR (Proof-Indexed IR) module: proof propagation from source, fact extraction, ProvenFact constructors, and contract fact conversion — previously zero integration test coverage.

## Philosophy Alignment
| Dimension | Score |
|-----------|-------|
| Core Mission Fit | 5/5 |
| Scope Boundaries | 5/5 |
| Architecture Patterns | 5/5 |
| Dependency Direction | 5/5 |

## Research Summary
- PIR module has 0 integration tests despite being core to BMB's proof-guided optimization pipeline
- Full pipeline: Source → AST → CIR (lower_to_cir) → PIR (propagate_proofs) → FunctionFacts (extract_all_pir_facts)
- `propagate_proofs(&CirProgram, &ProofDatabase) -> PirProgram`
- `extract_all_pir_facts(&PirProgram) -> HashMap<String, FunctionFacts>`
- FunctionFacts has preconditions, postconditions, all_facts (Vec<ContractFact>)
- ProvenFact has 3 constructors: from_precondition, from_control_flow, from_smt

## Implementation

### Integration Tests (`bmb/tests/integration.rs`)
Added `source_to_pir()` and `source_to_pir_facts()` helpers and 17 new tests:

**PIR Propagation (7 tests)**
- `test_pir_simple_function_propagation`: Single function → 1 PIR function
- `test_pir_multiple_functions`: 3 functions → 3 PIR functions
- `test_pir_precondition_becomes_entry_fact`: `pre b > 0` → entry_facts non-empty
- `test_pir_postcondition_becomes_exit_fact`: `post ret >= 0` → exit_facts non-empty
- `test_pir_function_return_type`: Bool return type preserved
- `test_pir_function_i64_return_type`: I64 return type preserved
- `test_pir_empty_program`: No contracts → empty facts

**Fact Extraction (5 tests)**
- `test_pir_extract_facts_simple_function`: Single function → 1 entry in facts map
- `test_pir_extract_facts_precondition`: `pre x >= 0` → non-empty preconditions
- `test_pir_extract_facts_postcondition`: `post ret > 0` → non-empty postconditions
- `test_pir_extract_facts_multiple_functions`: 2 functions → 2 fact entries
- `test_pir_extract_function_facts_with_contract`: Both pre+post → preconditions+postconditions+all_facts

**ProvenFact & ContractFact API (5 tests)**
- `test_pir_propagation_rule_enum`: All 5 PropagationRule variants exist
- `test_pir_proven_fact_constructors`: from_precondition, from_control_flow, from_smt
- `test_pir_proven_fact_to_contract_facts_var_cmp`: Compare → VarCmp
- `test_pir_proven_fact_to_contract_facts_non_null`: NonNull → NonNull fact
- `test_pir_proven_fact_to_contract_facts_in_bounds`: InBounds → ArrayBounds fact

## Test Results
- Standard tests: 2781 / 2781 passed (+17 from 2764)
- Clippy: CLEAN (0 errors)
- Build: SUCCESS

## Evaluation
| Criterion | Score | Notes |
|-----------|-------|-------|
| Correctness | 10/10 | All tests pass |
| Architecture | 9/10 | Tests full pipeline: Source → CIR → PIR → Facts |
| Philosophy Alignment | 10/10 | PIR is core to "proofs for speed" philosophy |
| Test Quality | 9/10 | Covers propagation, extraction, and API |
| Code Quality | 9/10 | Clean helpers, descriptive assertions |
| **Average** | **9.4/10** | |

## Issues & Improvements
| # | Severity | Description | Action |
|---|----------|-------------|--------|
| I-01 | L | Branch condition facts not tested (requires if/while with provable conditions) | Future cycle |
| I-02 | L | SMT-verified facts not tested (requires Z3 availability) | Tests avoid Z3 |

## Next Cycle Recommendation
- Add CFG module integration tests (conditional compilation)
