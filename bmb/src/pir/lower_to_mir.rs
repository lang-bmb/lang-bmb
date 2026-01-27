//! PIR to MIR Lowering
//!
//! Phase 2.3: Convert PIR to MIR while preserving proof information.
//! The proof annotations are carried through to enable proof-guided optimization.

use super::{PirProgram, PirExpr, PirExprKind};
use crate::cir::Proposition;

/// Lower PIR program to MIR (stub implementation)
///
/// This function converts PIR to MIR while preserving proof annotations.
/// The actual MIR representation depends on the existing MIR infrastructure.
pub fn lower_pir_to_mir(_pir: &PirProgram) -> MirWithProofs {
    // This is a stub implementation
    // Full implementation would integrate with existing MIR infrastructure

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
    fn test_lower_pir_to_mir() {
        let pir = PirProgram::new();
        let mir = lower_pir_to_mir(&pir);

        assert!(mir.functions.is_empty());
    }
}
