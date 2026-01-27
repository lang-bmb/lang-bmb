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
}
