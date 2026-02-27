//! PIR to MIR Lowering
//!
//! Phase 2.3: Convert PIR to MIR while preserving proof information.
//! The proof annotations are carried through to enable proof-guided optimization.
//!
//! # Architectural Decision (v0.60)
//!
//! **This module is intentionally minimal.** The BMB compiler uses a
//! **"fact-extraction" approach** rather than full PIR → MIR lowering.
//!
//! ## Current Architecture
//!
//! ```text
//! AST ──→ MIR (direct lowering for code generation)
//!  │
//!  └──→ CIR ──→ SMT Verification ──→ Verified CIR
//!                                       │
//!                                       ↓
//!                                     PIR (proof propagation)
//!                                       │
//!                                       ↓
//!                              Verified Facts ──→ MIR augmentation
//!                                                   │
//!                                                   ↓
//!                                          Proof-guided optimization
//!                                          (only verified facts = SOUND)
//! ```
//!
//! ## Why Not Full PIR → MIR Lowering?
//!
//! 1. **Fact-extraction is sufficient**: We only need contract facts to guide
//!    optimization. The MIR code generation doesn't change based on proofs.
//!
//! 2. **Avoids duplication**: Full lowering would duplicate AST → MIR logic,
//!    leading to potential inconsistencies.
//!
//! 3. **Verification at CIR level**: Soundness is guaranteed by verifying
//!    contracts at the CIR level before extracting facts.
//!
//! 4. **Simpler integration**: Adding facts to existing MIR is less invasive
//!    than replacing the entire MIR generation pipeline.
//!
//! ## Soundness Guarantee
//!
//! The key insight is that **proofs don't change what code does, only how
//! efficiently it runs**. The MIR from AST is always correct. The facts from
//! verified PIR just tell us which runtime checks can be safely eliminated.
//!
//! ## Usage in Build Pipeline
//!
//! The build pipeline (`bmb/src/build/mod.rs`) uses:
//! 1. `extract_all_facts()` / `extract_verified_facts()` from CIR
//! 2. `extract_all_pir_facts()` from PIR
//! 3. Direct augmentation of MIR function preconditions/postconditions
//!
//! The `lower_pir_to_mir()` function below is kept as a stub for potential
//! future use but is not used in the current architecture.

use super::{PirProgram, PirExpr, PirExprKind};
use crate::cir::Proposition;

/// Lower PIR program to MIR (stub implementation)
///
/// # Deprecation Notice
///
/// This function is **intentionally minimal** per the architectural decision above.
/// The BMB compiler uses fact-extraction from PIR rather than full lowering.
///
/// See `bmb/src/pir/to_mir_facts.rs` for the actual PIR → MIR fact extraction.
/// See `bmb/src/build/mod.rs` for how these facts are integrated into the pipeline.
#[deprecated(
    since = "0.60.0",
    note = "Use fact-extraction approach via to_mir_facts.rs instead of full lowering"
)]
pub fn lower_pir_to_mir(_pir: &PirProgram) -> MirWithProofs {
    // This is a stub implementation kept for potential future use.
    // The current architecture uses fact-extraction instead.

    MirWithProofs {
        functions: vec![],
        proof_annotations: vec![],
    }
}

/// MIR program with proof annotations
///
/// This is a simplified representation. The actual implementation would
/// extend the existing MIR data structures with proof fields.
#[derive(Debug)]
pub struct MirWithProofs {
    /// MIR functions
    pub functions: Vec<MirFunctionWithProofs>,

    /// Global proof annotations
    pub proof_annotations: Vec<ProofAnnotation>,
}

/// MIR function with proof information
#[derive(Debug)]
pub struct MirFunctionWithProofs {
    pub name: String,
    pub blocks: Vec<MirBlockWithProofs>,

    /// Entry facts (from preconditions)
    pub entry_proofs: Vec<ProofAnnotation>,

    /// Exit facts (for postconditions)
    pub exit_proofs: Vec<ProofAnnotation>,
}

/// MIR basic block with proofs
#[derive(Debug)]
pub struct MirBlockWithProofs {
    pub id: u32,
    pub statements: Vec<MirStmtWithProofs>,
    pub terminator: MirTerminator,

    /// Proofs valid at block entry
    pub entry_proofs: Vec<ProofAnnotation>,
}

/// MIR statement with proof annotation
#[derive(Debug)]
pub struct MirStmtWithProofs {
    pub kind: MirStmtKind,

    /// Proofs that enable optimizations on this statement
    pub available_proofs: Vec<ProofAnnotation>,
}

/// MIR statement kinds
#[derive(Debug, Clone)]
pub enum MirStmtKind {
    /// Assignment: local = rvalue
    Assign(u32, MirRvalue),

    /// Function call
    Call {
        dest: u32,
        func: String,
        args: Vec<u32>,
    },

    /// No-op (placeholder)
    Nop,
}

/// MIR rvalue (right-hand side of assignment)
#[derive(Debug, Clone)]
pub enum MirRvalue {
    /// Use a local
    Use(u32),

    /// Literal constant
    Constant(MirConstant),

    /// Binary operation
    BinaryOp(MirBinOp, u32, u32),

    /// Unary operation
    UnaryOp(MirUnaryOp, u32),

    /// Array/slice indexing
    Index(u32, u32),

    /// Field access
    Field(u32, String),

    /// Length
    Len(u32),
}

#[derive(Debug, Clone)]
pub enum MirConstant {
    Int(i64),
    Float(f64),
    Bool(bool),
    String(String),
    Unit,
}

#[derive(Debug, Clone, Copy)]
pub enum MirBinOp {
    Add, Sub, Mul, Div, Mod,
    Lt, Le, Gt, Ge, Eq, Ne,
    And, Or,
    BitAnd, BitOr, BitXor, Shl, Shr,
}

#[derive(Debug, Clone, Copy)]
pub enum MirUnaryOp {
    Neg, Not, BitNot,
}

/// MIR terminator
#[derive(Debug, Clone)]
pub enum MirTerminator {
    /// Return from function
    Return(u32),

    /// Unconditional goto
    Goto(u32),

    /// Conditional branch
    SwitchInt {
        discr: u32,
        targets: Vec<(i64, u32)>,
        default: u32,
    },

    /// Call with continuation
    Call {
        func: String,
        args: Vec<u32>,
        dest: u32,
        next: u32,
    },

    /// Unreachable
    Unreachable,
}

/// Proof annotation for MIR
#[derive(Debug, Clone)]
pub struct ProofAnnotation {
    /// The proven proposition
    pub proposition: Proposition,

    /// Kind of proof
    pub kind: ProofAnnotationKind,

    /// Proof ID for tracking
    pub id: u32,
}

/// Kind of proof annotation
#[derive(Debug, Clone)]
pub enum ProofAnnotationKind {
    /// Bounds check can be eliminated
    BoundsCheckEliminated,

    /// Null check can be eliminated
    NullCheckEliminated,

    /// Division zero check can be eliminated
    DivZeroCheckEliminated,

    /// Code is unreachable
    Unreachable,

    /// General fact (for propagation)
    Fact,
}

impl ProofAnnotation {
    pub fn bounds_check(prop: Proposition, id: u32) -> Self {
        Self {
            proposition: prop,
            kind: ProofAnnotationKind::BoundsCheckEliminated,
            id,
        }
    }

    pub fn null_check(prop: Proposition, id: u32) -> Self {
        Self {
            proposition: prop,
            kind: ProofAnnotationKind::NullCheckEliminated,
            id,
        }
    }

    pub fn div_zero(prop: Proposition, id: u32) -> Self {
        Self {
            proposition: prop,
            kind: ProofAnnotationKind::DivZeroCheckEliminated,
            id,
        }
    }

    pub fn unreachable(prop: Proposition, id: u32) -> Self {
        Self {
            proposition: prop,
            kind: ProofAnnotationKind::Unreachable,
            id,
        }
    }

    pub fn fact(prop: Proposition, id: u32) -> Self {
        Self {
            proposition: prop,
            kind: ProofAnnotationKind::Fact,
            id,
        }
    }
}

/// Convert PIR expression to MIR statements (stub)
#[allow(dead_code)]
fn lower_pir_expr_to_mir(
    expr: &PirExpr,
    dest: u32,
    next_local: &mut u32,
    stmts: &mut Vec<MirStmtWithProofs>,
) {
    // Collect proof annotations from PIR
    let available_proofs: Vec<ProofAnnotation> = expr.proven.iter()
        .map(|f| ProofAnnotation::fact(f.proposition.clone(), f.id))
        .collect();

    match &expr.kind {
        PirExprKind::IntLit(n) => {
            stmts.push(MirStmtWithProofs {
                kind: MirStmtKind::Assign(dest, MirRvalue::Constant(MirConstant::Int(*n))),
                available_proofs,
            });
        }

        PirExprKind::BoolLit(b) => {
            stmts.push(MirStmtWithProofs {
                kind: MirStmtKind::Assign(dest, MirRvalue::Constant(MirConstant::Bool(*b))),
                available_proofs,
            });
        }

        PirExprKind::Var(_name) => {
            // Would map name to local
            stmts.push(MirStmtWithProofs {
                kind: MirStmtKind::Assign(dest, MirRvalue::Use(0)),
                available_proofs,
            });
        }

        PirExprKind::Index { array, index, bounds_proof } => {
            // Lower array and index
            let arr_local = *next_local;
            *next_local += 1;
            lower_pir_expr_to_mir(array, arr_local, next_local, stmts);

            let idx_local = *next_local;
            *next_local += 1;
            lower_pir_expr_to_mir(index, idx_local, next_local, stmts);

            // Add bounds check annotation if proven
            let mut proofs = available_proofs;
            if let Some(bp) = bounds_proof {
                proofs.push(ProofAnnotation::bounds_check(bp.proposition.clone(), bp.id));
            }

            stmts.push(MirStmtWithProofs {
                kind: MirStmtKind::Assign(dest, MirRvalue::Index(arr_local, idx_local)),
                available_proofs: proofs,
            });
        }

        PirExprKind::Div { lhs, rhs, nonzero_proof } => {
            let lhs_local = *next_local;
            *next_local += 1;
            lower_pir_expr_to_mir(lhs, lhs_local, next_local, stmts);

            let rhs_local = *next_local;
            *next_local += 1;
            lower_pir_expr_to_mir(rhs, rhs_local, next_local, stmts);

            // Add zero check annotation if proven
            let mut proofs = available_proofs;
            if let Some(np) = nonzero_proof {
                proofs.push(ProofAnnotation::div_zero(np.proposition.clone(), np.id));
            }

            stmts.push(MirStmtWithProofs {
                kind: MirStmtKind::Assign(dest, MirRvalue::BinaryOp(MirBinOp::Div, lhs_local, rhs_local)),
                available_proofs: proofs,
            });
        }

        _ => {
            // Default: emit nop
            stmts.push(MirStmtWithProofs {
                kind: MirStmtKind::Nop,
                available_proofs,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_proof_annotation_creation() {
        let prop = Proposition::True;
        let ann = ProofAnnotation::bounds_check(prop.clone(), 1);

        assert!(matches!(ann.kind, ProofAnnotationKind::BoundsCheckEliminated));
        assert_eq!(ann.id, 1);
    }

    #[test]
    fn test_mir_with_proofs_creation() {
        let mir = MirWithProofs {
            functions: vec![],
            proof_annotations: vec![],
        };

        assert!(mir.functions.is_empty());
    }

    #[test]
    #[allow(deprecated)]
    fn test_lower_pir_to_mir() {
        let pir = PirProgram::new();
        let mir = lower_pir_to_mir(&pir);

        assert!(mir.functions.is_empty());
    }

    // ---- Cycle 70: Additional PIR lowering tests ----

    #[test]
    fn test_proof_annotation_null_check() {
        let ann = ProofAnnotation::null_check(Proposition::True, 2);
        assert!(matches!(ann.kind, ProofAnnotationKind::NullCheckEliminated));
        assert_eq!(ann.id, 2);
    }

    #[test]
    fn test_proof_annotation_div_zero() {
        let ann = ProofAnnotation::div_zero(Proposition::False, 3);
        assert!(matches!(ann.kind, ProofAnnotationKind::DivZeroCheckEliminated));
        assert_eq!(ann.id, 3);
    }

    #[test]
    fn test_proof_annotation_unreachable() {
        let ann = ProofAnnotation::unreachable(Proposition::True, 4);
        assert!(matches!(ann.kind, ProofAnnotationKind::Unreachable));
    }

    #[test]
    fn test_proof_annotation_fact() {
        let ann = ProofAnnotation::fact(Proposition::True, 5);
        assert!(matches!(ann.kind, ProofAnnotationKind::Fact));
    }

    #[test]
    fn test_mir_stmt_kind_nop() {
        let stmt = MirStmtWithProofs {
            kind: MirStmtKind::Nop,
            available_proofs: vec![],
        };
        assert!(matches!(stmt.kind, MirStmtKind::Nop));
    }

    #[test]
    fn test_mir_stmt_kind_assign() {
        let stmt = MirStmtWithProofs {
            kind: MirStmtKind::Assign(0, MirRvalue::Constant(MirConstant::Int(42))),
            available_proofs: vec![],
        };
        match &stmt.kind {
            MirStmtKind::Assign(dest, MirRvalue::Constant(MirConstant::Int(v))) => {
                assert_eq!(*dest, 0);
                assert_eq!(*v, 42);
            }
            _ => panic!("Expected Assign with Int constant"),
        }
    }

    #[test]
    fn test_mir_constant_variants() {
        let _int = MirConstant::Int(0);
        let _float = MirConstant::Float(1.0);
        let _bool = MirConstant::Bool(true);
        let _string = MirConstant::String("hello".to_string());
        let _unit = MirConstant::Unit;
    }

    #[test]
    fn test_mir_terminator_variants() {
        let _ret = MirTerminator::Return(0);
        let _goto = MirTerminator::Goto(1);
        let _unreachable = MirTerminator::Unreachable;
        let _switch = MirTerminator::SwitchInt {
            discr: 0,
            targets: vec![(0, 1), (1, 2)],
            default: 3,
        };
    }

    #[test]
    fn test_mir_block_with_proofs() {
        let block = MirBlockWithProofs {
            id: 0,
            statements: vec![],
            terminator: MirTerminator::Return(0),
            entry_proofs: vec![
                ProofAnnotation::fact(Proposition::True, 1),
            ],
        };
        assert_eq!(block.id, 0);
        assert!(block.statements.is_empty());
        assert_eq!(block.entry_proofs.len(), 1);
    }

    #[test]
    fn test_mir_function_with_proofs() {
        let func = MirFunctionWithProofs {
            name: "test".to_string(),
            blocks: vec![],
            entry_proofs: vec![],
            exit_proofs: vec![],
        };
        assert_eq!(func.name, "test");
        assert!(func.blocks.is_empty());
    }

    #[test]
    fn test_mir_rvalue_variants() {
        let _use = MirRvalue::Use(0);
        let _binop = MirRvalue::BinaryOp(MirBinOp::Add, 0, 1);
        let _unaryop = MirRvalue::UnaryOp(MirUnaryOp::Neg, 0);
        let _index = MirRvalue::Index(0, 1);
        let _field = MirRvalue::Field(0, "x".to_string());
        let _len = MirRvalue::Len(0);
    }

    // --- Cycle 1230: Additional PIR lower_to_mir Tests ---

    #[test]
    fn test_proof_annotation_stores_proposition() {
        let prop = Proposition::compare(
            crate::cir::CirExpr::var("x"),
            crate::cir::CompareOp::Gt,
            crate::cir::CirExpr::int(0),
        );
        let ann = ProofAnnotation::fact(prop.clone(), 10);
        assert_eq!(ann.proposition, prop);
        assert_eq!(ann.id, 10);
    }

    #[test]
    fn test_mir_binop_all_variants() {
        let ops = [
            MirBinOp::Add, MirBinOp::Sub, MirBinOp::Mul, MirBinOp::Div, MirBinOp::Mod,
            MirBinOp::Lt, MirBinOp::Le, MirBinOp::Gt, MirBinOp::Ge, MirBinOp::Eq, MirBinOp::Ne,
            MirBinOp::And, MirBinOp::Or,
            MirBinOp::BitAnd, MirBinOp::BitOr, MirBinOp::BitXor, MirBinOp::Shl, MirBinOp::Shr,
        ];
        for op in ops {
            let rv = MirRvalue::BinaryOp(op, 0, 1);
            assert!(matches!(rv, MirRvalue::BinaryOp(_, 0, 1)));
        }
    }

    #[test]
    fn test_mir_unaryop_all_variants() {
        for op in [MirUnaryOp::Neg, MirUnaryOp::Not, MirUnaryOp::BitNot] {
            let rv = MirRvalue::UnaryOp(op, 0);
            assert!(matches!(rv, MirRvalue::UnaryOp(_, 0)));
        }
    }

    #[test]
    fn test_mir_terminator_call() {
        let term = MirTerminator::Call {
            func: "add".to_string(),
            args: vec![0, 1],
            dest: 2,
            next: 1,
        };
        match term {
            MirTerminator::Call { func, args, dest, next } => {
                assert_eq!(func, "add");
                assert_eq!(args.len(), 2);
                assert_eq!(dest, 2);
                assert_eq!(next, 1);
            }
            _ => panic!("Expected Call"),
        }
    }

    #[test]
    fn test_mir_stmt_kind_call() {
        let stmt = MirStmtWithProofs {
            kind: MirStmtKind::Call {
                dest: 0,
                func: "foo".to_string(),
                args: vec![1, 2],
            },
            available_proofs: vec![ProofAnnotation::fact(Proposition::True, 1)],
        };
        match &stmt.kind {
            MirStmtKind::Call { dest, func, args } => {
                assert_eq!(*dest, 0);
                assert_eq!(func, "foo");
                assert_eq!(args.len(), 2);
            }
            _ => panic!("Expected Call"),
        }
        assert_eq!(stmt.available_proofs.len(), 1);
    }

    #[test]
    fn test_mir_with_proofs_with_content() {
        let mir = MirWithProofs {
            functions: vec![MirFunctionWithProofs {
                name: "main".to_string(),
                blocks: vec![MirBlockWithProofs {
                    id: 0,
                    statements: vec![MirStmtWithProofs {
                        kind: MirStmtKind::Assign(0, MirRvalue::Constant(MirConstant::Int(0))),
                        available_proofs: vec![],
                    }],
                    terminator: MirTerminator::Return(0),
                    entry_proofs: vec![],
                }],
                entry_proofs: vec![],
                exit_proofs: vec![],
            }],
            proof_annotations: vec![ProofAnnotation::fact(Proposition::True, 0)],
        };
        assert_eq!(mir.functions.len(), 1);
        assert_eq!(mir.functions[0].blocks.len(), 1);
        assert_eq!(mir.functions[0].blocks[0].statements.len(), 1);
        assert_eq!(mir.proof_annotations.len(), 1);
    }

    #[test]
    fn test_mir_function_with_entry_exit_proofs() {
        let func = MirFunctionWithProofs {
            name: "checked".to_string(),
            blocks: vec![],
            entry_proofs: vec![
                ProofAnnotation::bounds_check(Proposition::True, 1),
                ProofAnnotation::null_check(Proposition::True, 2),
            ],
            exit_proofs: vec![
                ProofAnnotation::fact(Proposition::True, 3),
            ],
        };
        assert_eq!(func.entry_proofs.len(), 2);
        assert_eq!(func.exit_proofs.len(), 1);
    }

    #[test]
    fn test_proof_annotation_kind_debug() {
        let kinds = [
            ProofAnnotationKind::BoundsCheckEliminated,
            ProofAnnotationKind::NullCheckEliminated,
            ProofAnnotationKind::DivZeroCheckEliminated,
            ProofAnnotationKind::Unreachable,
            ProofAnnotationKind::Fact,
        ];
        for kind in kinds {
            let debug = format!("{:?}", kind);
            assert!(!debug.is_empty());
        }
    }

    #[test]
    fn test_mir_constant_clone() {
        let c = MirConstant::String("hello".to_string());
        let c2 = c.clone();
        assert!(matches!(c2, MirConstant::String(s) if s == "hello"));
    }

    #[test]
    fn test_mir_terminator_switch_int() {
        let term = MirTerminator::SwitchInt {
            discr: 0,
            targets: vec![(0, 1), (1, 2), (2, 3)],
            default: 4,
        };
        match term {
            MirTerminator::SwitchInt { discr, targets, default } => {
                assert_eq!(discr, 0);
                assert_eq!(targets.len(), 3);
                assert_eq!(default, 4);
            }
            _ => panic!("Expected SwitchInt"),
        }
    }

    #[test]
    fn test_lower_pir_expr_int_lit() {
        use super::super::PirType;
        let expr = PirExpr {
            kind: PirExprKind::IntLit(42),
            proven: vec![],
            result_facts: vec![],
            ty: PirType::I64,
            span: None,
        };
        let mut stmts = vec![];
        let mut next_local = 1;
        lower_pir_expr_to_mir(&expr, 0, &mut next_local, &mut stmts);
        assert_eq!(stmts.len(), 1);
        match &stmts[0].kind {
            MirStmtKind::Assign(dest, MirRvalue::Constant(MirConstant::Int(v))) => {
                assert_eq!(*dest, 0);
                assert_eq!(*v, 42);
            }
            _ => panic!("Expected Assign with Int"),
        }
    }

    #[test]
    fn test_lower_pir_expr_bool_lit() {
        use super::super::PirType;
        let expr = PirExpr {
            kind: PirExprKind::BoolLit(true),
            proven: vec![],
            result_facts: vec![],
            ty: PirType::Bool,
            span: None,
        };
        let mut stmts = vec![];
        let mut next_local = 1;
        lower_pir_expr_to_mir(&expr, 0, &mut next_local, &mut stmts);
        assert_eq!(stmts.len(), 1);
        match &stmts[0].kind {
            MirStmtKind::Assign(_, MirRvalue::Constant(MirConstant::Bool(b))) => {
                assert!(*b);
            }
            _ => panic!("Expected Assign with Bool"),
        }
    }

    #[test]
    fn test_lower_pir_expr_var() {
        use super::super::PirType;
        let expr = PirExpr {
            kind: PirExprKind::Var("x".to_string()),
            proven: vec![],
            result_facts: vec![],
            ty: PirType::I64,
            span: None,
        };
        let mut stmts = vec![];
        let mut next_local = 1;
        lower_pir_expr_to_mir(&expr, 0, &mut next_local, &mut stmts);
        assert_eq!(stmts.len(), 1);
        assert!(matches!(&stmts[0].kind, MirStmtKind::Assign(0, MirRvalue::Use(0))));
    }

    // ================================================================
    // Additional PIR lower_to_mir tests (Cycle 1236)
    // ================================================================

    #[test]
    fn test_lower_pir_expr_nop_fallback() {
        use super::super::PirType;
        // Unknown/default expression kind falls back to Nop
        let expr = PirExpr {
            kind: PirExprKind::FloatLit(3_14u64),
            proven: vec![],
            result_facts: vec![],
            ty: PirType::F64,
            span: None,
        };
        let mut stmts = vec![];
        let mut next_local = 1;
        lower_pir_expr_to_mir(&expr, 0, &mut next_local, &mut stmts);
        assert_eq!(stmts.len(), 1);
        assert!(matches!(&stmts[0].kind, MirStmtKind::Nop));
    }

    #[test]
    fn test_lower_pir_expr_with_proven_facts() {
        use super::super::{PirType, ProvenFact};
        use crate::verify::ProofEvidence;
        let expr = PirExpr {
            kind: PirExprKind::IntLit(10),
            proven: vec![ProvenFact {
                proposition: Proposition::True,
                evidence: ProofEvidence::Precondition,
                id: 42,
            }],
            result_facts: vec![],
            ty: PirType::I64,
            span: None,
        };
        let mut stmts = vec![];
        let mut next_local = 1;
        lower_pir_expr_to_mir(&expr, 0, &mut next_local, &mut stmts);
        assert_eq!(stmts.len(), 1);
        assert_eq!(stmts[0].available_proofs.len(), 1);
        assert_eq!(stmts[0].available_proofs[0].id, 42);
    }

    #[test]
    fn test_lower_pir_expr_index_no_bounds_proof() {
        use super::super::PirType;
        let array_expr = PirExpr {
            kind: PirExprKind::Var("arr".to_string()),
            proven: vec![],
            result_facts: vec![],
            ty: PirType::I64,
            span: None,
        };
        let index_expr = PirExpr {
            kind: PirExprKind::IntLit(0),
            proven: vec![],
            result_facts: vec![],
            ty: PirType::I64,
            span: None,
        };
        let expr = PirExpr {
            kind: PirExprKind::Index {
                array: Box::new(array_expr),
                index: Box::new(index_expr),
                bounds_proof: None,
            },
            proven: vec![],
            result_facts: vec![],
            ty: PirType::I64,
            span: None,
        };
        let mut stmts = vec![];
        let mut next_local = 1;
        lower_pir_expr_to_mir(&expr, 0, &mut next_local, &mut stmts);
        // array var + index int + index assign = 3 statements
        assert_eq!(stmts.len(), 3);
        // Last statement should be the Index
        assert!(matches!(&stmts[2].kind, MirStmtKind::Assign(0, MirRvalue::Index(_, _))));
        // No bounds proof → only available_proofs from expr.proven (empty)
        assert!(stmts[2].available_proofs.is_empty());
    }

    #[test]
    fn test_lower_pir_expr_index_with_bounds_proof() {
        use super::super::{PirType, ProvenFact};
        let array_expr = PirExpr {
            kind: PirExprKind::Var("arr".to_string()),
            proven: vec![],
            result_facts: vec![],
            ty: PirType::I64,
            span: None,
        };
        let index_expr = PirExpr {
            kind: PirExprKind::IntLit(0),
            proven: vec![],
            result_facts: vec![],
            ty: PirType::I64,
            span: None,
        };
        let bounds_fact = ProvenFact {
            proposition: Proposition::True,
            evidence: crate::verify::ProofEvidence::Precondition,
            id: 99,
        };
        let expr = PirExpr {
            kind: PirExprKind::Index {
                array: Box::new(array_expr),
                index: Box::new(index_expr),
                bounds_proof: Some(bounds_fact),
            },
            proven: vec![],
            result_facts: vec![],
            ty: PirType::I64,
            span: None,
        };
        let mut stmts = vec![];
        let mut next_local = 1;
        lower_pir_expr_to_mir(&expr, 0, &mut next_local, &mut stmts);
        assert_eq!(stmts.len(), 3);
        // Last statement should have bounds check proof
        assert_eq!(stmts[2].available_proofs.len(), 1);
        assert!(matches!(stmts[2].available_proofs[0].kind, ProofAnnotationKind::BoundsCheckEliminated));
    }

    #[test]
    fn test_lower_pir_expr_div_no_proof() {
        use super::super::PirType;
        let lhs = PirExpr {
            kind: PirExprKind::IntLit(10),
            proven: vec![],
            result_facts: vec![],
            ty: PirType::I64,
            span: None,
        };
        let rhs = PirExpr {
            kind: PirExprKind::IntLit(2),
            proven: vec![],
            result_facts: vec![],
            ty: PirType::I64,
            span: None,
        };
        let expr = PirExpr {
            kind: PirExprKind::Div {
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
                nonzero_proof: None,
            },
            proven: vec![],
            result_facts: vec![],
            ty: PirType::I64,
            span: None,
        };
        let mut stmts = vec![];
        let mut next_local = 1;
        lower_pir_expr_to_mir(&expr, 0, &mut next_local, &mut stmts);
        assert_eq!(stmts.len(), 3);
        assert!(matches!(&stmts[2].kind, MirStmtKind::Assign(0, MirRvalue::BinaryOp(MirBinOp::Div, _, _))));
        assert!(stmts[2].available_proofs.is_empty());
    }

    #[test]
    fn test_lower_pir_expr_div_with_nonzero_proof() {
        use super::super::{PirType, ProvenFact};
        let lhs = PirExpr {
            kind: PirExprKind::IntLit(10),
            proven: vec![],
            result_facts: vec![],
            ty: PirType::I64,
            span: None,
        };
        let rhs = PirExpr {
            kind: PirExprKind::IntLit(2),
            proven: vec![],
            result_facts: vec![],
            ty: PirType::I64,
            span: None,
        };
        let nonzero_fact = ProvenFact {
            proposition: Proposition::True,
            evidence: crate::verify::ProofEvidence::Precondition,
            id: 77,
        };
        let expr = PirExpr {
            kind: PirExprKind::Div {
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
                nonzero_proof: Some(nonzero_fact),
            },
            proven: vec![],
            result_facts: vec![],
            ty: PirType::I64,
            span: None,
        };
        let mut stmts = vec![];
        let mut next_local = 1;
        lower_pir_expr_to_mir(&expr, 0, &mut next_local, &mut stmts);
        assert_eq!(stmts.len(), 3);
        assert_eq!(stmts[2].available_proofs.len(), 1);
        assert!(matches!(stmts[2].available_proofs[0].kind, ProofAnnotationKind::DivZeroCheckEliminated));
    }

    #[test]
    fn test_proof_annotation_clone() {
        let ann = ProofAnnotation::fact(Proposition::True, 1);
        let cloned = ann.clone();
        assert_eq!(cloned.id, 1);
        assert!(matches!(cloned.kind, ProofAnnotationKind::Fact));
    }

    #[test]
    fn test_mir_stmt_kind_clone() {
        let kind = MirStmtKind::Assign(0, MirRvalue::Constant(MirConstant::Bool(false)));
        let cloned = kind.clone();
        assert!(matches!(cloned, MirStmtKind::Assign(0, MirRvalue::Constant(MirConstant::Bool(false)))));
    }

    #[test]
    fn test_mir_rvalue_field_access() {
        let rv = MirRvalue::Field(0, "name".to_string());
        let cloned = rv.clone();
        match cloned {
            MirRvalue::Field(local, field) => {
                assert_eq!(local, 0);
                assert_eq!(field, "name");
            }
            _ => panic!("Expected Field"),
        }
    }

    #[test]
    fn test_mir_rvalue_len() {
        let rv = MirRvalue::Len(5);
        let cloned = rv.clone();
        assert!(matches!(cloned, MirRvalue::Len(5)));
    }
}
