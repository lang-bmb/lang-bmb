# Cycle 63: Add CIR + Contract Verification Tests

## Date
2026-02-08

## Scope
Add 35 tests across two modules: `cir/mod.rs` (Proposition/CompareOp/CirType/CirExpr/EffectSet) and `verify/contract.rs` (FunctionReport/VerificationReport/trust attribute/duplicate detection).

## Philosophy Alignment
| Dimension | Score |
|-----------|-------|
| Core Mission Fit | 5/5 |
| Scope Boundaries | 5/5 |
| Architecture Patterns | 5/5 |
| Dependency Direction | 5/5 |

## Changes

### bmb/src/cir/mod.rs (+24 tests)

**Proposition logic (9 tests):**
| Test | What it verifies |
|------|-----------------|
| `test_proposition_or_empty` | `Or([])` → `False` |
| `test_proposition_or_single` | `Or([True])` → `True` (unwrap) |
| `test_proposition_or_multiple` | `Or([True, False])` → `Or(...)` |
| `test_proposition_and_multiple` | `And([True, True])` → `And(...)` |
| `test_proposition_not_true_becomes_false` | `Not(True)` → `False` |
| `test_proposition_not_false_becomes_true` | `Not(False)` → `True` |
| `test_proposition_is_trivially_true` | `True.is_trivially_true()` correct |
| `test_proposition_is_trivially_false` | `False.is_trivially_false()` correct |
| `test_proposition_compare_constructor` | `Proposition::compare()` creates correct struct |

**CompareOp (4 tests):**
| Test | What it verifies |
|------|-----------------|
| `test_compare_op_negate_all` | All 6 operators negate correctly |
| `test_compare_op_flip_all` | All 6 operators flip correctly |
| `test_compare_op_double_negate` | `op.negate().negate() == op` for all |
| `test_compare_op_display` | Display strings: `<`, `<=`, `>`, `>=`, `==`, `!=` |

**CirType Display (3 tests):**
| Test | What it verifies |
|------|-----------------|
| `test_cir_type_display_numeric` | All 11 numeric type display strings |
| `test_cir_type_display_complex` | Bool/Unit/Char/String/Never/Infer/Ptr/RefMut/Slice/Array/Range |
| `test_cir_type_display_composite` | Struct/Enum/TypeParam/Tuple/Generic/Fn display |

**CirExpr constructors (3 tests):**
| Test | What it verifies |
|------|-----------------|
| `test_cir_expr_var_constructor` | `CirExpr::var("x")` |
| `test_cir_expr_int_constructor` | `CirExpr::int(42)`, `CirExpr::int(-1)` |
| `test_cir_expr_binop_constructor` | `CirExpr::binop(Add, 1, 2)` |

**EffectSet (5 tests):**
| Test | What it verifies |
|------|-----------------|
| `test_effect_set_const` | `const_()` → `is_pure && is_const` |
| `test_effect_set_impure` | `impure()` → `!is_pure && !is_const` |
| `test_effect_set_union_preserves_pure` | `pure.union(pure)` stays pure |
| `test_effect_set_union_const_with_impure` | `const_.union(io)` → loses pure+const |
| `test_effect_set_union_accumulates` | Multiple unions accumulate all effects |

### bmb/src/verify/contract.rs (+11 tests)

**FunctionReport (4 tests):**
| Test | What it verifies |
|------|-----------------|
| `test_function_report_default_state` | All fields start at default values |
| `test_function_report_pre_only_verified` | Pre+Post verified with no contracts → verified |
| `test_function_report_pre_failure_is_not_verified` | Pre failure → not verified, has_failure |
| `test_function_report_post_failure_is_not_verified` | Post failure → not verified, has_failure |

**VerificationReport (4 tests):**
| Test | What it verifies |
|------|-----------------|
| `test_verification_report_empty` | Empty report → all_verified, 0 counts |
| `test_verification_report_all_verified` | 2 verified functions → correct counts |
| `test_verification_report_display_all_verified` | Display contains "All 1 function(s)" |
| `test_verification_report_display_with_failure` | Display shows "Verified: 1/2, Failed: 1" |

**Trust & Detection (3 tests):**
| Test | What it verifies |
|------|-----------------|
| `test_verify_trust_attribute` | `@trust "reason"` → trusted=true, verified |
| `test_verifier_builder_pattern` | `new().with_timeout(30)` chains correctly |
| `test_no_duplicate_with_different_contracts` | Different contracts → no warning |

### Files Modified
- `bmb/src/cir/mod.rs` (+24 tests)
- `bmb/src/verify/contract.rs` (+11 tests)

## Test Results
| Test Suite | Result |
|------------|--------|
| Rust unit tests | 866/866 PASS (was 831, +35) |
| Clippy | PASS (0 warnings) |

## Notes
- `Attribute::Trust` doesn't exist; correct form is `Attribute::WithReason { name: "trust", reason: ... }`
- cir/mod.rs went from 8 to 32 tests (300% increase)
- contract.rs went from 13 to 24 tests (85% increase)

## Evaluation
| Criterion | Score | Notes |
|-----------|-------|-------|
| Correctness | 10/10 | All 866 tests pass |
| Architecture | 10/10 | Tests core CIR data structures and contract APIs |
| Philosophy Alignment | 10/10 | Contract verification is central to BMB |
| Test Quality | 10/10 | Constructor/display/edge case coverage |
| Code Quality | 10/10 | Follows existing test patterns |
| **Average** | **10.0/10** | |
