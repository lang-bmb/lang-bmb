//! CIR to MIR ContractFact Conversion
//!
//! Phase 1.3: Bridge CIR's rich Proposition system to MIR's ContractFact system.
//! This enables proof-guided optimizations to benefit from CIR's full contract
//! representation.
//!
//! # Rationale
//!
//! CIR has a complete first-order logic proposition system:
//! - Quantifiers (Forall, Exists)
//! - Logical connectives (And, Or, Not, Implies)
//! - Array bounds (InBounds)
//! - Non-null assertions (NonNull)
//!
//! MIR's ContractFact is simpler but sufficient for optimization:
//! - VarCmp (x op constant)
//! - VarVarCmp (x op y)
//! - ArrayBounds (index < len(array))
//! - NonNull (ptr != null)
//!
//! This module extracts as many ContractFacts as possible from CIR Propositions.

use super::{CirExpr, CirFunction, CirProgram, CompareOp, Proposition};
use crate::mir::{CmpOp, ContractFact};

/// Convert CIR comparison operator to MIR comparison operator
fn cir_op_to_mir_op(op: CompareOp) -> CmpOp {
    match op {
        CompareOp::Lt => CmpOp::Lt,
        CompareOp::Le => CmpOp::Le,
        CompareOp::Gt => CmpOp::Gt,
        CompareOp::Ge => CmpOp::Ge,
        CompareOp::Eq => CmpOp::Eq,
        CompareOp::Ne => CmpOp::Ne,
    }
}

/// Flip a comparison operator (for swapping lhs/rhs)
fn flip_cmp_op(op: CmpOp) -> CmpOp {
    match op {
        CmpOp::Lt => CmpOp::Gt,
        CmpOp::Le => CmpOp::Ge,
        CmpOp::Gt => CmpOp::Lt,
        CmpOp::Ge => CmpOp::Le,
        CmpOp::Eq => CmpOp::Eq,
        CmpOp::Ne => CmpOp::Ne,
    }
}

/// Extract ContractFacts from a CIR Proposition
///
/// This performs a best-effort extraction, producing as many facts as possible
/// from the proposition. Complex propositions that can't be represented as
/// ContractFacts are silently ignored.
pub fn proposition_to_facts(prop: &Proposition) -> Vec<ContractFact> {
    let mut facts = Vec::new();
    extract_facts_recursive(prop, &mut facts);
    facts
}

/// Recursively extract facts from a proposition
fn extract_facts_recursive(prop: &Proposition, facts: &mut Vec<ContractFact>) {
    match prop {
        // Base case: True/False don't produce facts
        Proposition::True | Proposition::False => {}

        // Comparison: lhs op rhs
        Proposition::Compare { lhs, op, rhs } => {
            let mir_op = cir_op_to_mir_op(*op);

            match (lhs.as_ref(), rhs.as_ref()) {
                // var op constant
                (CirExpr::Var(var), CirExpr::IntLit(val)) => {
                    facts.push(ContractFact::VarCmp {
                        var: var.clone(),
                        op: mir_op,
                        value: *val,
                    });
                }
                // constant op var -> flip
                (CirExpr::IntLit(val), CirExpr::Var(var)) => {
                    facts.push(ContractFact::VarCmp {
                        var: var.clone(),
                        op: flip_cmp_op(mir_op),
                        value: *val,
                    });
                }
                // var op var
                (CirExpr::Var(lhs_var), CirExpr::Var(rhs_var)) => {
                    facts.push(ContractFact::VarVarCmp {
                        lhs: lhs_var.clone(),
                        op: mir_op,
                        rhs: rhs_var.clone(),
                    });
                }
                // Handle expressions like var op len(array)
                (CirExpr::Var(var), CirExpr::Len(arr_expr)) => {
                    if let CirExpr::Var(arr_var) = arr_expr.as_ref() {
                        // var < len(arr) is an array bounds fact
                        if matches!(mir_op, CmpOp::Lt) {
                            facts.push(ContractFact::ArrayBounds {
                                index: var.clone(),
                                array: arr_var.clone(),
                            });
                        }
                    }
                }
                // len(array) op var
                (CirExpr::Len(arr_expr), CirExpr::Var(var)) => {
                    if let CirExpr::Var(arr_var) = arr_expr.as_ref() {
                        // len(arr) > var means var < len(arr) which is bounds
                        if matches!(mir_op, CmpOp::Gt) {
                            facts.push(ContractFact::ArrayBounds {
                                index: var.clone(),
                                array: arr_var.clone(),
                            });
                        }
                    }
                }
                _ => {}
            }
        }

        // Logical NOT: negate inner if possible
        Proposition::Not(inner) => {
            // For simple cases, we can negate comparisons
            if let Proposition::Compare { lhs, op, rhs } = inner.as_ref() {
                let negated_op = op.negate();
                let new_prop = Proposition::Compare {
                    lhs: lhs.clone(),
                    op: negated_op,
                    rhs: rhs.clone(),
                };
                extract_facts_recursive(&new_prop, facts);
            }
        }

        // Logical AND: extract from all conjuncts
        Proposition::And(props) => {
            for p in props {
                extract_facts_recursive(p, facts);
            }
        }

        // Logical OR: only extract if all branches give same fact
        // (conservative - we can't assume either branch)
        Proposition::Or(_) => {
            // Can't extract facts from OR (would need intersection)
        }

        // Implication: extract from consequent if antecedent is known true
        // (conservative - we only extract if implication is a precondition)
        Proposition::Implies(_, _) => {
            // Generally can't extract facts from implications without context
        }

        // Quantifiers: extract from body with variable bound
        Proposition::Forall { var: _, ty: _, body } => {
            // Facts from forall body hold for all values
            extract_facts_recursive(body, facts);
        }
        Proposition::Exists { var: _, ty: _, body: _ } => {
            // Can't extract universal facts from existential
        }

        // Predicates: can't extract facts from opaque predicates
        Proposition::Predicate { name: _, args: _ } => {}

        // Array bounds: direct conversion
        Proposition::InBounds { index, array } => {
            if let (CirExpr::Var(idx_var), CirExpr::Var(arr_var)) =
                (index.as_ref(), array.as_ref())
            {
                facts.push(ContractFact::ArrayBounds {
                    index: idx_var.clone(),
                    array: arr_var.clone(),
                });
            }
        }

        // Non-null: direct conversion
        Proposition::NonNull(expr) => {
            if let CirExpr::Var(var) = expr.as_ref() {
                facts.push(ContractFact::NonNull { var: var.clone() });
            }
        }

        // Old reference: ignore (only meaningful in postconditions)
        Proposition::Old(_, _) => {}
    }
}

/// Extract all ContractFacts from a CIR function's preconditions
pub fn extract_precondition_facts(func: &CirFunction) -> Vec<ContractFact> {
    let mut facts = Vec::new();
    for pre in &func.preconditions {
        facts.extend(proposition_to_facts(&pre.proposition));
    }
    // Also include parameter constraints
    for param in &func.params {
        for constraint in &param.constraints {
            facts.extend(proposition_to_facts(constraint));
        }
    }
    facts
}

/// Extract all ContractFacts from a CIR function's postconditions
pub fn extract_postcondition_facts(func: &CirFunction) -> Vec<ContractFact> {
    let mut facts = Vec::new();
    for post in &func.postconditions {
        facts.extend(proposition_to_facts(&post.proposition));
    }
    facts
}

/// Extract ContractFacts for all functions in a CIR program
///
/// Returns a map from function name to (preconditions, postconditions)
pub fn extract_all_facts(
    program: &CirProgram,
) -> std::collections::HashMap<String, (Vec<ContractFact>, Vec<ContractFact>)> {
    let mut result = std::collections::HashMap::new();

    for func in &program.functions {
        let pre_facts = extract_precondition_facts(func);
        let post_facts = extract_postcondition_facts(func);
        result.insert(func.name.clone(), (pre_facts, post_facts));
    }

    result
}

/// Extract ContractFacts only for functions that have been verified
///
/// # v0.60: Soundness Guarantee
///
/// This function is the cornerstone of BMB's soundness guarantee for proof-guided
/// optimizations. It only extracts facts from functions that have passed verification,
/// ensuring that the compiler never uses unproven assumptions for optimization.
///
/// ## Why This Matters
///
/// Using unverified contracts for optimization is **unsound**:
/// - The compiler assumes `pre idx < len(arr)` holds
/// - It eliminates bounds check based on this assumption
/// - If the contract is wrong, the program has undefined behavior
///
/// By only extracting facts from verified functions, we guarantee that every
/// optimization is backed by a mathematical proof.
///
/// # Arguments
/// * `program` - The CIR program containing all functions
/// * `verified_functions` - Set of function names that passed SMT verification
///
/// # Returns
/// Map from function name to (preconditions, postconditions) for verified functions only
pub fn extract_verified_facts(
    program: &CirProgram,
    verified_functions: &std::collections::HashSet<String>,
) -> std::collections::HashMap<String, (Vec<ContractFact>, Vec<ContractFact>)> {
    let mut result = std::collections::HashMap::new();

    for func in &program.functions {
        // Only extract facts for verified functions
        if verified_functions.contains(&func.name) {
            let pre_facts = extract_precondition_facts(func);
            let post_facts = extract_postcondition_facts(func);
            result.insert(func.name.clone(), (pre_facts, post_facts));
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_var_cmp_extraction() {
        // x >= 0
        let prop = Proposition::Compare {
            lhs: Box::new(CirExpr::Var("x".to_string())),
            op: CompareOp::Ge,
            rhs: Box::new(CirExpr::IntLit(0)),
        };

        let facts = proposition_to_facts(&prop);
        assert_eq!(facts.len(), 1);
        assert!(matches!(
            &facts[0],
            ContractFact::VarCmp { var, op: CmpOp::Ge, value: 0 } if var == "x"
        ));
    }

    #[test]
    fn test_constant_var_flip() {
        // 0 < x  ->  x > 0
        let prop = Proposition::Compare {
            lhs: Box::new(CirExpr::IntLit(0)),
            op: CompareOp::Lt,
            rhs: Box::new(CirExpr::Var("x".to_string())),
        };

        let facts = proposition_to_facts(&prop);
        assert_eq!(facts.len(), 1);
        assert!(matches!(
            &facts[0],
            ContractFact::VarCmp { var, op: CmpOp::Gt, value: 0 } if var == "x"
        ));
    }

    #[test]
    fn test_var_var_cmp() {
        // x < y
        let prop = Proposition::Compare {
            lhs: Box::new(CirExpr::Var("x".to_string())),
            op: CompareOp::Lt,
            rhs: Box::new(CirExpr::Var("y".to_string())),
        };

        let facts = proposition_to_facts(&prop);
        assert_eq!(facts.len(), 1);
        assert!(matches!(
            &facts[0],
            ContractFact::VarVarCmp { lhs, op: CmpOp::Lt, rhs }
            if lhs == "x" && rhs == "y"
        ));
    }

    #[test]
    fn test_and_extraction() {
        // x >= 0 && y > 0
        let prop = Proposition::And(vec![
            Proposition::Compare {
                lhs: Box::new(CirExpr::Var("x".to_string())),
                op: CompareOp::Ge,
                rhs: Box::new(CirExpr::IntLit(0)),
            },
            Proposition::Compare {
                lhs: Box::new(CirExpr::Var("y".to_string())),
                op: CompareOp::Gt,
                rhs: Box::new(CirExpr::IntLit(0)),
            },
        ]);

        let facts = proposition_to_facts(&prop);
        assert_eq!(facts.len(), 2);
    }

    #[test]
    fn test_in_bounds_extraction() {
        // InBounds { index: i, array: arr }
        let prop = Proposition::InBounds {
            index: Box::new(CirExpr::Var("i".to_string())),
            array: Box::new(CirExpr::Var("arr".to_string())),
        };

        let facts = proposition_to_facts(&prop);
        assert_eq!(facts.len(), 1);
        assert!(matches!(
            &facts[0],
            ContractFact::ArrayBounds { index, array }
            if index == "i" && array == "arr"
        ));
    }

    #[test]
    fn test_non_null_extraction() {
        let prop = Proposition::NonNull(Box::new(CirExpr::Var("ptr".to_string())));

        let facts = proposition_to_facts(&prop);
        assert_eq!(facts.len(), 1);
        assert!(matches!(
            &facts[0],
            ContractFact::NonNull { var } if var == "ptr"
        ));
    }

    #[test]
    fn test_len_comparison() {
        // i < len(arr)
        let prop = Proposition::Compare {
            lhs: Box::new(CirExpr::Var("i".to_string())),
            op: CompareOp::Lt,
            rhs: Box::new(CirExpr::Len(Box::new(CirExpr::Var("arr".to_string())))),
        };

        let facts = proposition_to_facts(&prop);
        assert_eq!(facts.len(), 1);
        assert!(matches!(
            &facts[0],
            ContractFact::ArrayBounds { index, array }
            if index == "i" && array == "arr"
        ));
    }

    #[test]
    fn test_not_negation() {
        // !(x == 0)  ->  x != 0
        let prop = Proposition::Not(Box::new(Proposition::Compare {
            lhs: Box::new(CirExpr::Var("x".to_string())),
            op: CompareOp::Eq,
            rhs: Box::new(CirExpr::IntLit(0)),
        }));

        let facts = proposition_to_facts(&prop);
        assert_eq!(facts.len(), 1);
        assert!(matches!(
            &facts[0],
            ContractFact::VarCmp { var, op: CmpOp::Ne, value: 0 } if var == "x"
        ));
    }

    #[test]
    fn test_forall_extraction() {
        // forall i: i64. i >= 0
        let prop = Proposition::Forall {
            var: "i".to_string(),
            ty: crate::cir::CirType::I64,
            body: Box::new(Proposition::Compare {
                lhs: Box::new(CirExpr::Var("i".to_string())),
                op: CompareOp::Ge,
                rhs: Box::new(CirExpr::IntLit(0)),
            }),
        };

        let facts = proposition_to_facts(&prop);
        assert_eq!(facts.len(), 1);
    }

    #[test]
    fn test_or_no_extraction() {
        // x > 0 || y > 0 - can't extract facts
        let prop = Proposition::Or(vec![
            Proposition::Compare {
                lhs: Box::new(CirExpr::Var("x".to_string())),
                op: CompareOp::Gt,
                rhs: Box::new(CirExpr::IntLit(0)),
            },
            Proposition::Compare {
                lhs: Box::new(CirExpr::Var("y".to_string())),
                op: CompareOp::Gt,
                rhs: Box::new(CirExpr::IntLit(0)),
            },
        ]);

        let facts = proposition_to_facts(&prop);
        assert!(facts.is_empty());
    }

    // ================================================================
    // Additional coverage tests (Cycle 424)
    // ================================================================

    #[test]
    fn test_true_false_no_facts() {
        assert!(proposition_to_facts(&Proposition::True).is_empty());
        assert!(proposition_to_facts(&Proposition::False).is_empty());
    }

    #[test]
    fn test_exists_no_extraction() {
        let prop = Proposition::Exists {
            var: "i".to_string(),
            ty: crate::cir::CirType::I64,
            body: Box::new(Proposition::Compare {
                lhs: Box::new(CirExpr::Var("i".to_string())),
                op: CompareOp::Ge,
                rhs: Box::new(CirExpr::IntLit(0)),
            }),
        };
        assert!(proposition_to_facts(&prop).is_empty());
    }

    #[test]
    fn test_predicate_no_extraction() {
        let prop = Proposition::Predicate {
            name: "is_sorted".to_string(),
            args: vec![CirExpr::Var("arr".to_string())],
        };
        assert!(proposition_to_facts(&prop).is_empty());
    }

    #[test]
    fn test_implies_no_extraction() {
        let prop = Proposition::Implies(
            Box::new(Proposition::Compare {
                lhs: Box::new(CirExpr::Var("x".to_string())),
                op: CompareOp::Gt,
                rhs: Box::new(CirExpr::IntLit(0)),
            }),
            Box::new(Proposition::Compare {
                lhs: Box::new(CirExpr::Var("y".to_string())),
                op: CompareOp::Gt,
                rhs: Box::new(CirExpr::IntLit(0)),
            }),
        );
        assert!(proposition_to_facts(&prop).is_empty());
    }

    #[test]
    fn test_old_no_extraction() {
        let prop = Proposition::Old(
            Box::new(CirExpr::Var("result".to_string())),
            Box::new(Proposition::Compare {
                lhs: Box::new(CirExpr::Var("x".to_string())),
                op: CompareOp::Ge,
                rhs: Box::new(CirExpr::IntLit(0)),
            }),
        );
        assert!(proposition_to_facts(&prop).is_empty());
    }

    #[test]
    fn test_len_gt_var_array_bounds() {
        // len(arr) > i  ->  ArrayBounds { index: i, array: arr }
        let prop = Proposition::Compare {
            lhs: Box::new(CirExpr::Len(Box::new(CirExpr::Var("arr".to_string())))),
            op: CompareOp::Gt,
            rhs: Box::new(CirExpr::Var("i".to_string())),
        };

        let facts = proposition_to_facts(&prop);
        assert_eq!(facts.len(), 1);
        assert!(matches!(
            &facts[0],
            ContractFact::ArrayBounds { index, array }
            if index == "i" && array == "arr"
        ));
    }

    #[test]
    fn test_flip_cmp_op_all_variants() {
        assert!(matches!(flip_cmp_op(CmpOp::Lt), CmpOp::Gt));
        assert!(matches!(flip_cmp_op(CmpOp::Le), CmpOp::Ge));
        assert!(matches!(flip_cmp_op(CmpOp::Gt), CmpOp::Lt));
        assert!(matches!(flip_cmp_op(CmpOp::Ge), CmpOp::Le));
        assert!(matches!(flip_cmp_op(CmpOp::Eq), CmpOp::Eq));
        assert!(matches!(flip_cmp_op(CmpOp::Ne), CmpOp::Ne));
    }

    #[test]
    fn test_cir_op_to_mir_op_all_variants() {
        assert!(matches!(cir_op_to_mir_op(CompareOp::Lt), CmpOp::Lt));
        assert!(matches!(cir_op_to_mir_op(CompareOp::Le), CmpOp::Le));
        assert!(matches!(cir_op_to_mir_op(CompareOp::Gt), CmpOp::Gt));
        assert!(matches!(cir_op_to_mir_op(CompareOp::Ge), CmpOp::Ge));
        assert!(matches!(cir_op_to_mir_op(CompareOp::Eq), CmpOp::Eq));
        assert!(matches!(cir_op_to_mir_op(CompareOp::Ne), CmpOp::Ne));
    }

    #[test]
    fn test_non_null_non_var_ignored() {
        // NonNull(IntLit) is not a Var - no fact
        let prop = Proposition::NonNull(Box::new(CirExpr::IntLit(42)));
        assert!(proposition_to_facts(&prop).is_empty());
    }

    #[test]
    fn test_in_bounds_non_var_ignored() {
        // InBounds with non-var args
        let prop = Proposition::InBounds {
            index: Box::new(CirExpr::IntLit(0)),
            array: Box::new(CirExpr::Var("arr".to_string())),
        };
        assert!(proposition_to_facts(&prop).is_empty());
    }

    #[test]
    fn test_not_complex_expr_ignored() {
        // !(pred) where pred is not Compare → no facts
        let prop = Proposition::Not(Box::new(Proposition::True));
        assert!(proposition_to_facts(&prop).is_empty());
    }

    #[test]
    fn test_extract_all_facts_empty_program() {
        let program = CirProgram {
            functions: vec![],
            extern_fns: vec![],
            structs: std::collections::HashMap::new(),
            type_invariants: std::collections::HashMap::new(),
        };
        let result = extract_all_facts(&program);
        assert!(result.is_empty());
    }

    #[test]
    fn test_extract_verified_facts_filters() {
        use crate::cir::{NamedProposition, EffectSet};
        let func = CirFunction {
            name: "foo".to_string(),
            type_params: vec![],
            params: vec![],
            ret_ty: crate::cir::CirType::I64,
            ret_name: "result".to_string(),
            body: CirExpr::IntLit(0),
            preconditions: vec![NamedProposition {
                name: None,
                proposition: Proposition::Compare {
                    lhs: Box::new(CirExpr::Var("x".to_string())),
                    op: CompareOp::Ge,
                    rhs: Box::new(CirExpr::IntLit(0)),
                },
            }],
            postconditions: vec![],
            loop_invariants: vec![],
            effects: EffectSet::pure(),
        };
        let program = CirProgram {
            functions: vec![func],
            extern_fns: vec![],
            structs: std::collections::HashMap::new(),
            type_invariants: std::collections::HashMap::new(),
        };

        // Without "foo" in verified set, no facts
        let empty_verified = std::collections::HashSet::new();
        let result = extract_verified_facts(&program, &empty_verified);
        assert!(result.is_empty());

        // With "foo" verified, facts are extracted
        let mut verified = std::collections::HashSet::new();
        verified.insert("foo".to_string());
        let result = extract_verified_facts(&program, &verified);
        assert_eq!(result.len(), 1);
        assert!(result.contains_key("foo"));
    }

    // ================================================================
    // Additional coverage tests (Cycle 1232)
    // ================================================================

    fn make_cir_function(name: &str) -> CirFunction {
        use crate::cir::EffectSet;
        CirFunction {
            name: name.to_string(),
            type_params: vec![],
            params: vec![],
            ret_ty: crate::cir::CirType::I64,
            ret_name: "result".to_string(),
            body: CirExpr::IntLit(0),
            preconditions: vec![],
            postconditions: vec![],
            loop_invariants: vec![],
            effects: EffectSet::pure(),
        }
    }

    #[test]
    fn test_extract_precondition_facts_with_param_constraints() {
        use crate::cir::{CirParam, EffectSet, NamedProposition};
        let func = CirFunction {
            name: "bounded".to_string(),
            type_params: vec![],
            params: vec![CirParam {
                name: "idx".to_string(),
                ty: crate::cir::CirType::I64,
                constraints: vec![Proposition::Compare {
                    lhs: Box::new(CirExpr::Var("idx".to_string())),
                    op: CompareOp::Ge,
                    rhs: Box::new(CirExpr::IntLit(0)),
                }],
            }],
            ret_ty: crate::cir::CirType::I64,
            ret_name: "result".to_string(),
            body: CirExpr::IntLit(0),
            preconditions: vec![NamedProposition {
                name: None,
                proposition: Proposition::Compare {
                    lhs: Box::new(CirExpr::Var("idx".to_string())),
                    op: CompareOp::Lt,
                    rhs: Box::new(CirExpr::IntLit(100)),
                },
            }],
            postconditions: vec![],
            loop_invariants: vec![],
            effects: EffectSet::pure(),
        };
        let facts = extract_precondition_facts(&func);
        // 1 from precondition + 1 from param constraint
        assert_eq!(facts.len(), 2);
    }

    #[test]
    fn test_extract_postcondition_facts_basic() {
        use crate::cir::NamedProposition;
        let mut func = make_cir_function("post_fn");
        func.postconditions = vec![NamedProposition {
            name: Some("positive_result".to_string()),
            proposition: Proposition::Compare {
                lhs: Box::new(CirExpr::Var("result".to_string())),
                op: CompareOp::Ge,
                rhs: Box::new(CirExpr::IntLit(0)),
            },
        }];
        let facts = extract_postcondition_facts(&func);
        assert_eq!(facts.len(), 1);
        assert!(matches!(
            &facts[0],
            ContractFact::VarCmp { var, op: CmpOp::Ge, value: 0 } if var == "result"
        ));
    }

    #[test]
    fn test_extract_all_facts_multiple_functions() {
        use crate::cir::NamedProposition;
        let mut func1 = make_cir_function("fn_a");
        func1.preconditions = vec![NamedProposition {
            name: None,
            proposition: Proposition::Compare {
                lhs: Box::new(CirExpr::Var("x".to_string())),
                op: CompareOp::Gt,
                rhs: Box::new(CirExpr::IntLit(0)),
            },
        }];
        let mut func2 = make_cir_function("fn_b");
        func2.postconditions = vec![NamedProposition {
            name: None,
            proposition: Proposition::NonNull(Box::new(CirExpr::Var("ptr".to_string()))),
        }];
        let program = CirProgram {
            functions: vec![func1, func2],
            extern_fns: vec![],
            structs: std::collections::HashMap::new(),
            type_invariants: std::collections::HashMap::new(),
        };
        let result = extract_all_facts(&program);
        assert_eq!(result.len(), 2);
        assert!(result.contains_key("fn_a"));
        assert!(result.contains_key("fn_b"));
        // fn_a has 1 precondition fact
        assert_eq!(result["fn_a"].0.len(), 1);
        // fn_b has 1 postcondition fact
        assert_eq!(result["fn_b"].1.len(), 1);
    }

    #[test]
    fn test_extract_verified_facts_partial() {
        use crate::cir::NamedProposition;
        let mut func1 = make_cir_function("verified_fn");
        func1.preconditions = vec![NamedProposition {
            name: None,
            proposition: Proposition::Compare {
                lhs: Box::new(CirExpr::Var("x".to_string())),
                op: CompareOp::Ge,
                rhs: Box::new(CirExpr::IntLit(0)),
            },
        }];
        let mut func2 = make_cir_function("unverified_fn");
        func2.preconditions = vec![NamedProposition {
            name: None,
            proposition: Proposition::Compare {
                lhs: Box::new(CirExpr::Var("y".to_string())),
                op: CompareOp::Gt,
                rhs: Box::new(CirExpr::IntLit(0)),
            },
        }];
        let program = CirProgram {
            functions: vec![func1, func2],
            extern_fns: vec![],
            structs: std::collections::HashMap::new(),
            type_invariants: std::collections::HashMap::new(),
        };
        let mut verified = std::collections::HashSet::new();
        verified.insert("verified_fn".to_string());
        let result = extract_verified_facts(&program, &verified);
        assert_eq!(result.len(), 1);
        assert!(result.contains_key("verified_fn"));
        assert!(!result.contains_key("unverified_fn"));
    }

    #[test]
    fn test_nested_and_not_combination() {
        // AND(x > 0, NOT(y == 5)) => x > 0 AND y != 5
        let prop = Proposition::And(vec![
            Proposition::Compare {
                lhs: Box::new(CirExpr::Var("x".to_string())),
                op: CompareOp::Gt,
                rhs: Box::new(CirExpr::IntLit(0)),
            },
            Proposition::Not(Box::new(Proposition::Compare {
                lhs: Box::new(CirExpr::Var("y".to_string())),
                op: CompareOp::Eq,
                rhs: Box::new(CirExpr::IntLit(5)),
            })),
        ]);
        let facts = proposition_to_facts(&prop);
        assert_eq!(facts.len(), 2);
        assert!(matches!(
            &facts[0],
            ContractFact::VarCmp { var, op: CmpOp::Gt, value: 0 } if var == "x"
        ));
        assert!(matches!(
            &facts[1],
            ContractFact::VarCmp { var, op: CmpOp::Ne, value: 5 } if var == "y"
        ));
    }

    #[test]
    fn test_flip_cmp_op_double_flip_identity() {
        // Flipping twice returns original
        for op in [CmpOp::Lt, CmpOp::Le, CmpOp::Gt, CmpOp::Ge, CmpOp::Eq, CmpOp::Ne] {
            assert_eq!(flip_cmp_op(flip_cmp_op(op)), op);
        }
    }

    #[test]
    fn test_forall_nested_and() {
        // forall i: i64. (i >= 0 AND i < 100)
        let prop = Proposition::Forall {
            var: "i".to_string(),
            ty: crate::cir::CirType::I64,
            body: Box::new(Proposition::And(vec![
                Proposition::Compare {
                    lhs: Box::new(CirExpr::Var("i".to_string())),
                    op: CompareOp::Ge,
                    rhs: Box::new(CirExpr::IntLit(0)),
                },
                Proposition::Compare {
                    lhs: Box::new(CirExpr::Var("i".to_string())),
                    op: CompareOp::Lt,
                    rhs: Box::new(CirExpr::IntLit(100)),
                },
            ])),
        };
        let facts = proposition_to_facts(&prop);
        assert_eq!(facts.len(), 2);
    }

    #[test]
    fn test_len_comparison_non_lt_no_bounds() {
        // i >= len(arr) does NOT produce ArrayBounds (only Lt does)
        let prop = Proposition::Compare {
            lhs: Box::new(CirExpr::Var("i".to_string())),
            op: CompareOp::Ge,
            rhs: Box::new(CirExpr::Len(Box::new(CirExpr::Var("arr".to_string())))),
        };
        let facts = proposition_to_facts(&prop);
        assert!(facts.is_empty());
    }

    #[test]
    fn test_len_non_var_array_ignored() {
        // i < len(IntLit) - array is not a Var
        let prop = Proposition::Compare {
            lhs: Box::new(CirExpr::Var("i".to_string())),
            op: CompareOp::Lt,
            rhs: Box::new(CirExpr::Len(Box::new(CirExpr::IntLit(10)))),
        };
        let facts = proposition_to_facts(&prop);
        assert!(facts.is_empty());
    }

    #[test]
    fn test_extract_all_facts_with_both_pre_post() {
        use crate::cir::NamedProposition;
        let mut func = make_cir_function("both_fn");
        func.preconditions = vec![NamedProposition {
            name: None,
            proposition: Proposition::Compare {
                lhs: Box::new(CirExpr::Var("n".to_string())),
                op: CompareOp::Gt,
                rhs: Box::new(CirExpr::IntLit(0)),
            },
        }];
        func.postconditions = vec![NamedProposition {
            name: None,
            proposition: Proposition::Compare {
                lhs: Box::new(CirExpr::Var("result".to_string())),
                op: CompareOp::Ge,
                rhs: Box::new(CirExpr::IntLit(0)),
            },
        }];
        let program = CirProgram {
            functions: vec![func],
            extern_fns: vec![],
            structs: std::collections::HashMap::new(),
            type_invariants: std::collections::HashMap::new(),
        };
        let result = extract_all_facts(&program);
        let (pre, post) = &result["both_fn"];
        assert_eq!(pre.len(), 1);
        assert_eq!(post.len(), 1);
    }

    // ================================================================
    // Additional CIR to_mir_facts tests (Cycle 1239)
    // ================================================================

    #[test]
    fn test_extract_precondition_facts_empty() {
        let func = make_cir_function("no_pre");
        let facts = extract_precondition_facts(&func);
        assert!(facts.is_empty());
    }

    #[test]
    fn test_extract_postcondition_facts_empty() {
        let func = make_cir_function("no_post");
        let facts = extract_postcondition_facts(&func);
        assert!(facts.is_empty());
    }

    #[test]
    fn test_not_of_gt() {
        // !(x > 5)  =>  x <= 5
        let prop = Proposition::Not(Box::new(Proposition::Compare {
            lhs: Box::new(CirExpr::Var("x".to_string())),
            op: CompareOp::Gt,
            rhs: Box::new(CirExpr::IntLit(5)),
        }));
        let facts = proposition_to_facts(&prop);
        assert_eq!(facts.len(), 1);
        assert!(matches!(
            &facts[0],
            ContractFact::VarCmp { var, op: CmpOp::Le, value: 5 } if var == "x"
        ));
    }

    #[test]
    fn test_var_eq_constant() {
        let prop = Proposition::Compare {
            lhs: Box::new(CirExpr::Var("status".to_string())),
            op: CompareOp::Eq,
            rhs: Box::new(CirExpr::IntLit(200)),
        };
        let facts = proposition_to_facts(&prop);
        assert_eq!(facts.len(), 1);
        assert!(matches!(
            &facts[0],
            ContractFact::VarCmp { var, op: CmpOp::Eq, value: 200 } if var == "status"
        ));
    }

    #[test]
    fn test_constant_ge_var_flip() {
        // 10 >= x  =>  x <= 10
        let prop = Proposition::Compare {
            lhs: Box::new(CirExpr::IntLit(10)),
            op: CompareOp::Ge,
            rhs: Box::new(CirExpr::Var("x".to_string())),
        };
        let facts = proposition_to_facts(&prop);
        assert_eq!(facts.len(), 1);
        assert!(matches!(
            &facts[0],
            ContractFact::VarCmp { var, op: CmpOp::Le, value: 10 } if var == "x"
        ));
    }

    #[test]
    fn test_and_with_nonnull() {
        let prop = Proposition::And(vec![
            Proposition::NonNull(Box::new(CirExpr::Var("ptr".to_string()))),
            Proposition::Compare {
                lhs: Box::new(CirExpr::Var("x".to_string())),
                op: CompareOp::Gt,
                rhs: Box::new(CirExpr::IntLit(0)),
            },
        ]);
        let facts = proposition_to_facts(&prop);
        assert_eq!(facts.len(), 2);
    }

    #[test]
    fn test_len_gt_non_var_ignored() {
        // len(IntLit) > x - array expr is not a Var
        let prop = Proposition::Compare {
            lhs: Box::new(CirExpr::Len(Box::new(CirExpr::IntLit(5)))),
            op: CompareOp::Gt,
            rhs: Box::new(CirExpr::Var("i".to_string())),
        };
        let facts = proposition_to_facts(&prop);
        assert!(facts.is_empty());
    }

    #[test]
    fn test_extract_all_facts_single_function() {
        let func = make_cir_function("single");
        let program = CirProgram {
            functions: vec![func],
            extern_fns: vec![],
            structs: std::collections::HashMap::new(),
            type_invariants: std::collections::HashMap::new(),
        };
        let result = extract_all_facts(&program);
        assert_eq!(result.len(), 1);
        assert!(result.contains_key("single"));
    }

    #[test]
    fn test_not_true_no_facts() {
        let prop = Proposition::Not(Box::new(Proposition::True));
        let facts = proposition_to_facts(&prop);
        assert!(facts.is_empty());
    }

    #[test]
    fn test_forall_with_nonnull() {
        let prop = Proposition::Forall {
            var: "p".to_string(),
            ty: crate::cir::CirType::I64,
            body: Box::new(Proposition::NonNull(
                Box::new(CirExpr::Var("p".to_string())),
            )),
        };
        let facts = proposition_to_facts(&prop);
        assert_eq!(facts.len(), 1);
        assert!(matches!(&facts[0], ContractFact::NonNull { var } if var == "p"));
    }
}
