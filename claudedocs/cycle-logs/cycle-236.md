# Cycle 236: CIR Verification Pipeline Integration Tests

## Date
2026-02-11

## Scope
Add integration tests for CIR (Contract IR) module: lowering, Proposition API, EffectSet, CirOutput formatting, fact extraction, SMT generator, CirVerifier, ProofWitness/ProofOutcome.

## Philosophy Alignment
| Dimension | Score |
|-----------|-------|
| Core Mission Fit | 5/5 |
| Scope Boundaries | 5/5 |
| Architecture Patterns | 5/5 |
| Dependency Direction | 5/5 |

## Research Summary
- CIR module bridges typed AST → PIR/MIR via contract-aware IR
- `lower_to_cir(&ast) -> CirProgram` is the main entry point
- `extract_all_facts(&CirProgram)` returns `HashMap<String, (Vec<ContractFact>, Vec<ContractFact>)>` (pre, post pairs)
- `extract_verified_facts` takes 2 args: `(&CirProgram, &HashSet<String>)` — filters by verified functions
- Proposition has constructors: `compare()`, `and()`, `or()`, `not()`, plus `is_trivially_true/false`
- EffectSet has `pure()`, `impure()`, `union()` constructors
- CirVerifier uses builder pattern: `new().with_z3_path().with_timeout()`
- ProofWitness has `verified()`, `failed()`, `skipped()`, `error()` constructors

## Implementation

### Integration Tests (`bmb/tests/integration.rs`)
Added `source_to_cir()` helper and 38 new tests:

**CIR Lowering (7 tests)**
- `test_cir_lower_simple_function`: Parse → CIR → function entry with params
- `test_cir_lower_multiple_functions`: 3 functions lowered
- `test_cir_lower_precondition`: `pre b > 0` → preconditions non-empty
- `test_cir_lower_postcondition`: `post ret > 0` → postconditions non-empty
- `test_cir_lower_pre_and_post`: Both pre and post conditions preserved
- `test_cir_lower_struct`: Struct with 2 fields in CIR structs map
- `test_cir_lower_return_type`: Bool return type preserved

**Proposition API (4 tests)**
- `test_cir_proposition_trivially_true`: True is trivially true
- `test_cir_proposition_trivially_false`: False is trivially false
- `test_cir_proposition_compare`: Compare proposition creation
- `test_cir_proposition_and_or_not`: And/Not combinators

**EffectSet (3 tests)**
- `test_cir_effect_set_pure`: Pure effects
- `test_cir_effect_set_impure`: Impure effects
- `test_cir_effect_set_union`: Union preserves impurity

**CIR Output (3 tests)**
- `test_cir_output_format_text`: Text output contains function name
- `test_cir_output_format_text_with_contract`: Contract info in text output
- `test_cir_output_format_json`: Valid JSON output

**Fact Extraction (5 tests)**
- `test_cir_extract_precondition_facts`: Pre facts from `pre x >= 0`
- `test_cir_extract_postcondition_facts`: Post facts from `post ret > 0`
- `test_cir_extract_all_facts`: Program-level fact map with pre/post pairs
- `test_cir_extract_verified_facts`: Filtered by verified function set
- `test_cir_extract_facts_no_contracts`: No contracts → empty facts

**SMT Generator (6 tests)**
- `test_cir_smt_generator_creation`: Fresh generator with check-sat
- `test_cir_smt_generator_declare_var`: Variable declaration
- `test_cir_smt_generator_translate_proposition`: Comparison translation
- `test_cir_smt_generator_translate_expr`: Integer expression translation
- `test_cir_smt_sort_to_smt`: Int/Bool/Real sort strings
- `test_cir_smt_generator_type_to_sort`: CirType::I64 → Int sort

**CIR Verifier & ProofWitness (8 tests)**
- `test_cir_verifier_creation`: Constructor works
- `test_cir_verification_report_empty`: Empty report defaults
- `test_cir_proof_witness_verified`: Verified witness
- `test_cir_proof_witness_failed`: Failed witness
- `test_cir_proof_witness_skipped`: Skipped witness
- `test_cir_proof_witness_error`: Error witness
- `test_cir_proof_outcome_variants`: All 5 ProofOutcome variants
- `test_cir_verification_report_summary`: Summary generation

**CompareOp (2 tests)**
- `test_cir_compare_op_negate`: Lt↔Ge, Eq↔Ne
- `test_cir_compare_op_flip`: Lt↔Gt, Le↔Ge

## Test Results
- Standard tests: 2867 / 2867 passed (+38 from 2829)
- Clippy: CLEAN (0 errors)
- Build: SUCCESS

## Evaluation
| Criterion | Score | Notes |
|-----------|-------|-------|
| Correctness | 10/10 | All tests pass |
| Architecture | 9/10 | Tests full pipeline: Source → CIR → Facts/SMT/Output |
| Philosophy Alignment | 10/10 | CIR is central to proof-guided optimization |
| Test Quality | 9/10 | Covers lowering, propositions, effects, SMT, verification, output |
| Code Quality | 9/10 | Fixed extract_all_facts/extract_verified_facts signatures |
| **Average** | **9.4/10** | |

## Issues & Improvements
| # | Severity | Description | Action |
|---|----------|-------------|--------|
| I-01 | L | CirVerifier.verify_program not tested with Z3 | Requires Z3 installation |
| I-02 | L | generate_verification_query not tested E2E | Would need solver |
| I-03 | L | CirExpr variants (While, Loop, Match) not tested in lowering | Complex source patterns needed |

## Next Cycle Recommendation
- Add Derive module integration tests (derive macros, auto-trait generation)
