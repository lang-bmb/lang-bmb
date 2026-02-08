//! Function Summary Extraction
//!
//! Phase 1.2: Extract function summaries for inter-module verification
//! and incremental compilation support.

use std::collections::HashMap;

use crate::cir::{CirProgram, CirFunction, Proposition, EffectSet};
use super::proof_db::FunctionId;

/// Function summary - caller's view of the contract
#[derive(Debug, Clone)]
pub struct FunctionSummary {
    /// Preconditions (caller must guarantee)
    pub requires: Vec<Proposition>,

    /// Postconditions (callee guarantees)
    pub ensures: Vec<Proposition>,

    /// Effect summary
    pub effects: EffectSet,

    /// Termination status
    pub termination: TerminationStatus,

    /// Whether this function has been verified
    pub verified: bool,

    /// Hash of function body (for change detection)
    pub body_hash: u64,
}

impl Default for FunctionSummary {
    fn default() -> Self {
        Self {
            requires: Vec::new(),
            ensures: Vec::new(),
            effects: EffectSet::impure(),
            termination: TerminationStatus::Unknown,
            verified: false,
            body_hash: 0,
        }
    }
}

/// Termination status of a function
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerminationStatus {
    /// Proven to terminate
    Terminating,
    /// May not terminate (e.g., has unbounded loops)
    MayDiverge,
    /// Intentionally non-terminating (e.g., event loop)
    NonTerminating,
    /// Termination status unknown
    Unknown,
}

/// Extract summaries from all functions in a CIR program
pub fn extract_summaries(cir: &CirProgram) -> HashMap<FunctionId, FunctionSummary> {
    let mut summaries = HashMap::new();

    for func in &cir.functions {
        let id = FunctionId::simple(&func.name);
        let summary = extract_function_summary(func);
        summaries.insert(id.key(), (id, summary));
    }

    summaries.into_iter().map(|(_, (id, s))| (id, s)).collect()
}

/// Extract summary from a single function
pub fn extract_function_summary(func: &CirFunction) -> FunctionSummary {
    FunctionSummary {
        requires: func.preconditions
            .iter()
            .map(|p| p.proposition.clone())
            .collect(),
        ensures: func.postconditions
            .iter()
            .map(|p| p.proposition.clone())
            .collect(),
        effects: func.effects.clone(),
        termination: infer_termination(func),
        verified: false, // Will be set by verifier
        body_hash: compute_body_hash(func),
    }
}

/// Infer termination status from function structure
fn infer_termination(func: &CirFunction) -> TerminationStatus {
    // Simple heuristic: if function has no loops, it terminates
    // More sophisticated analysis would require loop variant analysis
    if func.effects.diverges || has_unbounded_loop(&func.body) {
        TerminationStatus::MayDiverge
    } else {
        TerminationStatus::Terminating
    }
}

/// Check if expression contains unbounded loops
fn has_unbounded_loop(expr: &crate::cir::CirExpr) -> bool {
    use crate::cir::CirExpr;

    match expr {
        CirExpr::Loop { .. } => true, // Simple check: any loop is potentially unbounded
        CirExpr::While { .. } => true,
        CirExpr::For { .. } => false, // For loops have bounded iteration
        CirExpr::If { then_branch, else_branch, .. } => {
            has_unbounded_loop(then_branch) || has_unbounded_loop(else_branch)
        }
        CirExpr::Let { body, .. } |
        CirExpr::LetMut { body, .. } => {
            has_unbounded_loop(body)
        }
        CirExpr::Block(exprs) => {
            exprs.iter().any(has_unbounded_loop)
        }
        _ => false,
    }
}

/// Compute a hash of the function body for change detection
fn compute_body_hash(func: &CirFunction) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();

    // Hash function name
    func.name.hash(&mut hasher);

    // Hash parameter names and types
    for param in &func.params {
        param.name.hash(&mut hasher);
        format!("{:?}", param.ty).hash(&mut hasher);
    }

    // Hash return type
    format!("{:?}", func.ret_ty).hash(&mut hasher);

    // Hash preconditions
    for pre in &func.preconditions {
        format!("{:?}", pre.proposition).hash(&mut hasher);
    }

    // Hash postconditions
    for post in &func.postconditions {
        format!("{:?}", post.proposition).hash(&mut hasher);
    }

    // Hash body structure (simplified - just format string)
    format!("{:?}", func.body).hash(&mut hasher);

    hasher.finish()
}

/// Summary comparison result
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SummaryChange {
    /// No change
    Unchanged,
    /// Contract changed (requires re-verification of callers)
    ContractChanged,
    /// Implementation changed but contract same (only re-verify this function)
    ImplementationChanged,
    /// New function
    Added,
    /// Function removed
    Removed,
}

/// Compare two summaries to detect changes
pub fn compare_summaries(
    old: Option<&FunctionSummary>,
    new: Option<&FunctionSummary>,
) -> SummaryChange {
    match (old, new) {
        (None, None) => SummaryChange::Unchanged,
        (None, Some(_)) => SummaryChange::Added,
        (Some(_), None) => SummaryChange::Removed,
        (Some(old), Some(new)) => {
            // Check if contracts changed
            if old.requires.len() != new.requires.len()
                || old.ensures.len() != new.ensures.len()
            {
                return SummaryChange::ContractChanged;
            }

            // Deep comparison would require proposition equality
            // For now, compare hashes
            if format!("{:?}", old.requires) != format!("{:?}", new.requires)
                || format!("{:?}", old.ensures) != format!("{:?}", new.ensures)
            {
                return SummaryChange::ContractChanged;
            }

            // Check effects
            if old.effects != new.effects {
                return SummaryChange::ContractChanged;
            }

            // Check body hash
            if old.body_hash != new.body_hash {
                return SummaryChange::ImplementationChanged;
            }

            SummaryChange::Unchanged
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cir::{CirParam, CirType, CirExpr, NamedProposition, CompareOp};

    fn make_test_function(name: &str) -> CirFunction {
        CirFunction {
            name: name.to_string(),
            type_params: vec![],
            params: vec![
                CirParam {
                    name: "x".to_string(),
                    ty: CirType::I64,
                    constraints: vec![],
                },
            ],
            ret_name: "result".to_string(),
            ret_ty: CirType::I64,
            preconditions: vec![
                NamedProposition {
                    name: None,
                    proposition: Proposition::Compare {
                        lhs: Box::new(CirExpr::Var("x".to_string())),
                        op: CompareOp::Gt,
                        rhs: Box::new(CirExpr::IntLit(0)),
                    },
                },
            ],
            postconditions: vec![
                NamedProposition {
                    name: None,
                    proposition: Proposition::Compare {
                        lhs: Box::new(CirExpr::Var("result".to_string())),
                        op: CompareOp::Gt,
                        rhs: Box::new(CirExpr::IntLit(0)),
                    },
                },
            ],
            loop_invariants: vec![],
            effects: EffectSet::pure(),
            body: CirExpr::Var("x".to_string()),
        }
    }

    #[test]
    fn test_extract_function_summary() {
        let func = make_test_function("foo");
        let summary = extract_function_summary(&func);

        assert_eq!(summary.requires.len(), 1);
        assert_eq!(summary.ensures.len(), 1);
        assert!(summary.effects.is_pure);
        assert_eq!(summary.termination, TerminationStatus::Terminating);
    }

    #[test]
    fn test_termination_detection() {
        let mut func = make_test_function("loop_fn");
        assert_eq!(infer_termination(&func), TerminationStatus::Terminating);

        // Add a loop
        func.body = CirExpr::Loop {
            body: Box::new(CirExpr::Break(Box::new(CirExpr::Unit))),
        };
        assert_eq!(infer_termination(&func), TerminationStatus::MayDiverge);
    }

    #[test]
    fn test_summary_comparison() {
        let func1 = make_test_function("foo");
        let func2 = make_test_function("foo");
        let mut func3 = make_test_function("foo");
        func3.body = CirExpr::IntLit(42); // Different implementation

        let summary1 = extract_function_summary(&func1);
        let summary2 = extract_function_summary(&func2);
        let summary3 = extract_function_summary(&func3);

        assert_eq!(
            compare_summaries(Some(&summary1), Some(&summary2)),
            SummaryChange::Unchanged
        );
        assert_eq!(
            compare_summaries(Some(&summary1), Some(&summary3)),
            SummaryChange::ImplementationChanged
        );
        assert_eq!(
            compare_summaries(None, Some(&summary1)),
            SummaryChange::Added
        );
        assert_eq!(
            compare_summaries(Some(&summary1), None),
            SummaryChange::Removed
        );
    }

    #[test]
    fn test_body_hash_consistency() {
        let func1 = make_test_function("foo");
        let func2 = make_test_function("foo");

        let hash1 = compute_body_hash(&func1);
        let hash2 = compute_body_hash(&func2);

        assert_eq!(hash1, hash2, "Same functions should have same hash");
    }

    // ---- Cycle 73: Additional summary tests ----

    #[test]
    fn test_summary_change_eq() {
        assert_eq!(SummaryChange::Unchanged, SummaryChange::Unchanged);
        assert_eq!(SummaryChange::ContractChanged, SummaryChange::ContractChanged);
        assert_eq!(SummaryChange::ImplementationChanged, SummaryChange::ImplementationChanged);
        assert_eq!(SummaryChange::Added, SummaryChange::Added);
        assert_eq!(SummaryChange::Removed, SummaryChange::Removed);
        assert_ne!(SummaryChange::Unchanged, SummaryChange::Added);
    }

    #[test]
    fn test_termination_status_all_variants() {
        assert_eq!(TerminationStatus::Terminating, TerminationStatus::Terminating);
        assert_eq!(TerminationStatus::MayDiverge, TerminationStatus::MayDiverge);
        assert_eq!(TerminationStatus::NonTerminating, TerminationStatus::NonTerminating);
        assert_eq!(TerminationStatus::Unknown, TerminationStatus::Unknown);
        assert_ne!(TerminationStatus::Terminating, TerminationStatus::MayDiverge);
    }

    #[test]
    fn test_function_summary_default() {
        let summary = FunctionSummary::default();
        assert!(summary.requires.is_empty());
        assert!(summary.ensures.is_empty());
        assert!(!summary.effects.is_pure);  // impure by default
        assert_eq!(summary.termination, TerminationStatus::Unknown);
        assert!(!summary.verified);
        assert_eq!(summary.body_hash, 0);
    }

    #[test]
    fn test_extract_summaries_multiple_functions() {
        let program = CirProgram {
            functions: vec![
                make_test_function("alpha"),
                make_test_function("beta"),
                make_test_function("gamma"),
            ],
            extern_fns: vec![],
            structs: std::collections::HashMap::new(),
            type_invariants: std::collections::HashMap::new(),
        };

        let summaries = extract_summaries(&program);
        assert_eq!(summaries.len(), 3);

        // Check all functions have summaries
        let names: Vec<String> = summaries.keys().map(|id| id.name.clone()).collect();
        assert!(names.contains(&"alpha".to_string()));
        assert!(names.contains(&"beta".to_string()));
        assert!(names.contains(&"gamma".to_string()));
    }

    #[test]
    fn test_extract_summaries_empty_program() {
        let program = CirProgram {
            functions: vec![],
            extern_fns: vec![],
            structs: std::collections::HashMap::new(),
            type_invariants: std::collections::HashMap::new(),
        };

        let summaries = extract_summaries(&program);
        assert!(summaries.is_empty());
    }

    #[test]
    fn test_compare_summaries_none_none() {
        assert_eq!(compare_summaries(None, None), SummaryChange::Unchanged);
    }

    #[test]
    fn test_compare_summaries_contract_changed() {
        let func1 = make_test_function("foo");
        let summary1 = extract_function_summary(&func1);

        // Remove preconditions to change contract
        let mut func2 = make_test_function("foo");
        func2.preconditions = vec![];
        let summary2 = extract_function_summary(&func2);

        assert_eq!(
            compare_summaries(Some(&summary1), Some(&summary2)),
            SummaryChange::ContractChanged
        );
    }

    #[test]
    fn test_compare_summaries_effects_changed() {
        let func1 = make_test_function("foo");
        let summary1 = extract_function_summary(&func1);

        let mut func2 = make_test_function("foo");
        func2.effects = EffectSet::impure();
        let summary2 = extract_function_summary(&func2);

        assert_eq!(
            compare_summaries(Some(&summary1), Some(&summary2)),
            SummaryChange::ContractChanged
        );
    }

    #[test]
    fn test_has_unbounded_loop_while() {
        let expr = CirExpr::While {
            cond: Box::new(CirExpr::BoolLit(true)),
            body: Box::new(CirExpr::Unit),
            invariant: None,
        };
        assert!(has_unbounded_loop(&expr));
    }

    #[test]
    fn test_has_unbounded_loop_for_is_bounded() {
        let expr = CirExpr::For {
            var: "i".to_string(),
            iter: Box::new(CirExpr::IntLit(10)),
            body: Box::new(CirExpr::Unit),
        };
        assert!(!has_unbounded_loop(&expr));
    }

    #[test]
    fn test_has_unbounded_loop_nested_in_if() {
        let expr = CirExpr::If {
            cond: Box::new(CirExpr::BoolLit(true)),
            then_branch: Box::new(CirExpr::Loop {
                body: Box::new(CirExpr::Break(Box::new(CirExpr::Unit))),
            }),
            else_branch: Box::new(CirExpr::Unit),
        };
        assert!(has_unbounded_loop(&expr));
    }

    #[test]
    fn test_has_unbounded_loop_nested_in_let() {
        let expr = CirExpr::Let {
            name: "x".to_string(),
            ty: CirType::I64,
            value: Box::new(CirExpr::IntLit(1)),
            body: Box::new(CirExpr::While {
                cond: Box::new(CirExpr::BoolLit(true)),
                body: Box::new(CirExpr::Unit),
                invariant: None,
            }),
        };
        assert!(has_unbounded_loop(&expr));
    }

    #[test]
    fn test_has_unbounded_loop_nested_in_block() {
        let expr = CirExpr::Block(vec![
            CirExpr::IntLit(1),
            CirExpr::Loop {
                body: Box::new(CirExpr::Break(Box::new(CirExpr::Unit))),
            },
        ]);
        assert!(has_unbounded_loop(&expr));
    }

    #[test]
    fn test_has_unbounded_loop_simple_expr() {
        assert!(!has_unbounded_loop(&CirExpr::IntLit(42)));
        assert!(!has_unbounded_loop(&CirExpr::BoolLit(true)));
        assert!(!has_unbounded_loop(&CirExpr::Var("x".to_string())));
        assert!(!has_unbounded_loop(&CirExpr::Unit));
    }

    #[test]
    fn test_body_hash_different_functions() {
        let func1 = make_test_function("foo");
        let func2 = make_test_function("bar");

        let hash1 = compute_body_hash(&func1);
        let hash2 = compute_body_hash(&func2);

        assert_ne!(hash1, hash2, "Different function names should produce different hashes");
    }

    #[test]
    fn test_body_hash_different_body() {
        let func1 = make_test_function("foo");
        let mut func2 = make_test_function("foo");
        func2.body = CirExpr::IntLit(99);

        let hash1 = compute_body_hash(&func1);
        let hash2 = compute_body_hash(&func2);

        assert_ne!(hash1, hash2, "Different bodies should produce different hashes");
    }

    #[test]
    fn test_infer_termination_diverges_flag() {
        let mut func = make_test_function("diverge_fn");
        func.effects.diverges = true;
        assert_eq!(infer_termination(&func), TerminationStatus::MayDiverge);
    }

    #[test]
    fn test_extract_function_summary_preserves_effects() {
        let mut func = make_test_function("io_fn");
        func.effects = EffectSet {
            is_pure: false,
            is_const: false,
            reads: true,
            writes: true,
            allocates: false,
            diverges: false,
            io: true,
        };

        let summary = extract_function_summary(&func);
        assert!(!summary.effects.is_pure);
        assert!(summary.effects.io);
        assert!(summary.effects.reads);
        assert!(summary.effects.writes);
    }
}
