//! PIR to MIR ContractFact Conversion
//!
//! Phase 2.4: Extract ContractFacts from PIR's rich proof information.
//! PIR carries proof annotations through the entire program, including:
//! - Branch condition facts (if/else)
//! - Loop invariant facts (while, for)
//! - Postcondition facts from function calls
//! - Let binding facts
//!
//! This module extracts these facts for use by MIR's proof-guided optimizations.

use std::collections::HashMap;

use super::{PirProgram, PirFunction, PirExpr, PirExprKind, ProvenFact};
use crate::cir::{CirExpr, CompareOp, Proposition};
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

/// Flip a comparison operator
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

/// Extract ContractFacts from a ProvenFact
pub fn proven_fact_to_contract_facts(fact: &ProvenFact) -> Vec<ContractFact> {
    proposition_to_facts(&fact.proposition)
}

/// Extract ContractFacts from a Proposition
fn proposition_to_facts(prop: &Proposition) -> Vec<ContractFact> {
    let mut facts = Vec::new();
    extract_facts_recursive(prop, &mut facts);
    facts
}

/// Recursively extract facts
fn extract_facts_recursive(prop: &Proposition, facts: &mut Vec<ContractFact>) {
    match prop {
        Proposition::True | Proposition::False => {}

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
                // var < len(arr)
                (CirExpr::Var(var), CirExpr::Len(arr_expr)) => {
                    if let CirExpr::Var(arr_var) = arr_expr.as_ref()
                        && matches!(mir_op, CmpOp::Lt) {
                            facts.push(ContractFact::ArrayBounds {
                                index: var.clone(),
                                array: arr_var.clone(),
                            });
                        }
                }
                _ => {}
            }
        }

        Proposition::Not(inner) => {
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

        Proposition::And(props) => {
            for p in props {
                extract_facts_recursive(p, facts);
            }
        }

        Proposition::Or(_) => {
            // Can't extract universal facts from OR
        }

        Proposition::Implies(_, _) => {
            // Need more context to extract from implications
        }

        Proposition::Forall { body, .. } => {
            extract_facts_recursive(body, facts);
        }

        Proposition::Exists { .. } => {}

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

        Proposition::NonNull(expr) => {
            if let CirExpr::Var(var) = expr.as_ref() {
                facts.push(ContractFact::NonNull { var: var.clone() });
            }
        }

        Proposition::Predicate { .. } | Proposition::Old(_, _) => {}
    }
}

/// Extract all ContractFacts from a PIR function
///
/// This extracts facts from:
/// - Entry facts (preconditions)
/// - Exit facts (postconditions)
/// - Branch condition facts
/// - Loop invariant facts
/// - Postcondition facts from function calls
pub fn extract_function_facts(func: &PirFunction) -> FunctionFacts {
    let mut preconditions = Vec::new();
    let mut postconditions = Vec::new();
    let mut all_facts = Vec::new();

    // Extract from entry facts (preconditions)
    for fact in &func.entry_facts {
        let extracted = proven_fact_to_contract_facts(fact);
        preconditions.extend(extracted.clone());
        all_facts.extend(extracted);
    }

    // Extract from exit facts (postconditions)
    for fact in &func.exit_facts {
        let extracted = proven_fact_to_contract_facts(fact);
        postconditions.extend(extracted.clone());
    }

    // Extract facts from expression tree
    extract_expr_facts(&func.body, &mut all_facts);

    FunctionFacts {
        preconditions,
        postconditions,
        all_facts,
    }
}

/// Facts extracted from a PIR function
#[derive(Debug, Clone)]
pub struct FunctionFacts {
    /// Precondition-derived facts
    pub preconditions: Vec<ContractFact>,
    /// Postcondition-derived facts
    pub postconditions: Vec<ContractFact>,
    /// All facts (including branch conditions, loop invariants, etc.)
    pub all_facts: Vec<ContractFact>,
}

/// Recursively extract facts from PIR expression tree
fn extract_expr_facts(expr: &PirExpr, facts: &mut Vec<ContractFact>) {
    // Extract facts from this expression's proven facts
    for fact in &expr.proven {
        facts.extend(proven_fact_to_contract_facts(fact));
    }

    // Recursively visit children
    match &expr.kind {
        PirExprKind::If { cond, then_branch, else_branch, then_facts, else_facts } => {
            extract_expr_facts(cond, facts);

            // Add branch-specific facts
            for fact in then_facts {
                facts.extend(proven_fact_to_contract_facts(fact));
            }
            extract_expr_facts(then_branch, facts);

            for fact in else_facts {
                facts.extend(proven_fact_to_contract_facts(fact));
            }
            extract_expr_facts(else_branch, facts);
        }

        PirExprKind::While { cond, body, invariant_facts } => {
            extract_expr_facts(cond, facts);

            // Add loop invariant facts
            for fact in invariant_facts {
                facts.extend(proven_fact_to_contract_facts(fact));
            }
            extract_expr_facts(body, facts);
        }

        PirExprKind::Loop { body, invariant_facts } => {
            for fact in invariant_facts {
                facts.extend(proven_fact_to_contract_facts(fact));
            }
            extract_expr_facts(body, facts);
        }

        PirExprKind::For { iter, body, iter_facts, .. } => {
            extract_expr_facts(iter, facts);

            // Add iterator facts (bounds)
            for fact in iter_facts {
                facts.extend(proven_fact_to_contract_facts(fact));
            }
            extract_expr_facts(body, facts);
        }

        PirExprKind::Let { value, body, value_facts, .. } => {
            // Add facts from let binding
            for fact in value_facts {
                facts.extend(proven_fact_to_contract_facts(fact));
            }
            extract_expr_facts(value, facts);
            extract_expr_facts(body, facts);
        }

        PirExprKind::LetMut { value, body, .. } => {
            extract_expr_facts(value, facts);
            extract_expr_facts(body, facts);
        }

        PirExprKind::Call { args, postcondition_facts, .. } => {
            for arg in args {
                extract_expr_facts(arg, facts);
            }
            // Add postcondition facts from callee
            for fact in postcondition_facts {
                facts.extend(proven_fact_to_contract_facts(fact));
            }
        }

        PirExprKind::BinOp { lhs, rhs, .. } => {
            extract_expr_facts(lhs, facts);
            extract_expr_facts(rhs, facts);
        }

        PirExprKind::Div { lhs, rhs, nonzero_proof } => {
            extract_expr_facts(lhs, facts);
            extract_expr_facts(rhs, facts);
            if let Some(proof) = nonzero_proof {
                facts.extend(proven_fact_to_contract_facts(proof));
            }
        }

        PirExprKind::UnaryOp { operand, .. } => {
            extract_expr_facts(operand, facts);
        }

        PirExprKind::Index { array, index, bounds_proof } => {
            extract_expr_facts(array, facts);
            extract_expr_facts(index, facts);
            if let Some(proof) = bounds_proof {
                facts.extend(proven_fact_to_contract_facts(proof));
            }
        }

        PirExprKind::Field { base, null_proof, .. } => {
            extract_expr_facts(base, facts);
            if let Some(proof) = null_proof {
                facts.extend(proven_fact_to_contract_facts(proof));
            }
        }

        PirExprKind::Block(exprs) => {
            for e in exprs {
                extract_expr_facts(e, facts);
            }
        }

        PirExprKind::Array(exprs) | PirExprKind::Tuple(exprs) => {
            for e in exprs {
                extract_expr_facts(e, facts);
            }
        }

        PirExprKind::Struct { fields, .. } => {
            for (_, e) in fields {
                extract_expr_facts(e, facts);
            }
        }

        PirExprKind::Assign { value, .. } => {
            extract_expr_facts(value, facts);
        }

        PirExprKind::Ref(e) | PirExprKind::RefMut(e) | PirExprKind::Deref(e) |
        PirExprKind::Len(e) | PirExprKind::Break(e) => {
            extract_expr_facts(e, facts);
        }

        PirExprKind::Cast { expr, .. } => {
            extract_expr_facts(expr, facts);
        }

        // Leaves
        PirExprKind::IntLit(_) | PirExprKind::FloatLit(_) | PirExprKind::BoolLit(_) |
        PirExprKind::StringLit(_) | PirExprKind::Unit | PirExprKind::Var(_) |
        PirExprKind::Continue => {}
    }
}

/// Extract ContractFacts for all functions in a PIR program
pub fn extract_all_pir_facts(program: &PirProgram) -> HashMap<String, FunctionFacts> {
    let mut result = HashMap::new();

    for func in &program.functions {
        let facts = extract_function_facts(func);
        result.insert(func.name.clone(), facts);
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pir::{PirType, PirParam};
    use crate::verify::ProofEvidence;

    fn make_proven_fact(prop: Proposition) -> ProvenFact {
        ProvenFact {
            proposition: prop,
            evidence: ProofEvidence::Precondition,
            id: 1,
        }
    }

    #[test]
    fn test_var_cmp_extraction() {
        let prop = Proposition::Compare {
            lhs: Box::new(CirExpr::Var("x".to_string())),
            op: CompareOp::Ge,
            rhs: Box::new(CirExpr::IntLit(0)),
        };

        let fact = make_proven_fact(prop);
        let facts = proven_fact_to_contract_facts(&fact);

        assert_eq!(facts.len(), 1);
        assert!(matches!(
            &facts[0],
            ContractFact::VarCmp { var, op: CmpOp::Ge, value: 0 } if var == "x"
        ));
    }

    #[test]
    fn test_in_bounds_extraction() {
        let prop = Proposition::InBounds {
            index: Box::new(CirExpr::Var("i".to_string())),
            array: Box::new(CirExpr::Var("arr".to_string())),
        };

        let fact = make_proven_fact(prop);
        let facts = proven_fact_to_contract_facts(&fact);

        assert_eq!(facts.len(), 1);
        assert!(matches!(
            &facts[0],
            ContractFact::ArrayBounds { index, array }
            if index == "i" && array == "arr"
        ));
    }

    #[test]
    fn test_function_facts_extraction() {
        let func = PirFunction {
            name: "test".to_string(),
            params: vec![PirParam {
                name: "x".to_string(),
                ty: PirType::I64,
                constraints: vec![],
            }],
            ret_ty: PirType::I64,
            body: PirExpr::new(PirExprKind::Var("x".to_string()), PirType::I64),
            entry_facts: vec![make_proven_fact(Proposition::Compare {
                lhs: Box::new(CirExpr::Var("x".to_string())),
                op: CompareOp::Gt,
                rhs: Box::new(CirExpr::IntLit(0)),
            })],
            exit_facts: vec![],
        };

        let facts = extract_function_facts(&func);
        assert!(!facts.preconditions.is_empty());
    }
}
