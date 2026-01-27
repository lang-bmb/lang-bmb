//! Proof-Indexed IR (PIR)
//!
//! Phase 2: IR where every expression carries proven facts.
//! PIR enables proof-guided optimizations by tracking what facts are
//! known at each program point.
//!
//! # Design Philosophy
//!
//! > "Proofs are not for safety. Proofs are for speed."
//!
//! PIR makes proven facts explicit at every expression, allowing
//! optimization passes to:
//! - Eliminate bounds checks when array access is proven safe
//! - Remove null checks when non-null is proven
//! - Skip division-by-zero checks when divisor is proven non-zero
//! - Remove unreachable code when condition contradicts known facts
//!
//! # v0.54: Initial Implementation

mod propagate;
mod lower_to_mir;
mod to_mir_facts;

pub use propagate::{propagate_proofs, PropagationRule};
pub use lower_to_mir::lower_pir_to_mir;
pub use to_mir_facts::{
    extract_function_facts, extract_all_pir_facts, FunctionFacts,
    proven_fact_to_contract_facts,
};

use std::collections::HashMap;

use crate::cir::Proposition;
use crate::verify::{ProofDatabase, ProofFact, ProofScope, ProofEvidence, FunctionId};

/// A PIR program with proof database
#[derive(Debug)]
pub struct PirProgram {
    /// Functions with attached proofs
    pub functions: Vec<PirFunction>,

    /// Proof database for lookup
    pub proof_db: ProofDatabase,

    /// Type invariants
    pub type_invariants: HashMap<String, Vec<Proposition>>,
}

impl PirProgram {
    pub fn new() -> Self {
        Self {
            functions: Vec::new(),
            proof_db: ProofDatabase::new(),
            type_invariants: HashMap::new(),
        }
    }
}

impl Default for PirProgram {
    fn default() -> Self {
        Self::new()
    }
}

/// A function with proof annotations
#[derive(Debug, Clone)]
pub struct PirFunction {
    /// Function name
    pub name: String,

    /// Parameters with types
    pub params: Vec<PirParam>,

    /// Return type
    pub ret_ty: PirType,

    /// Function body
    pub body: PirExpr,

    /// Facts proven at function entry (from preconditions)
    pub entry_facts: Vec<ProvenFact>,

    /// Facts guaranteed at function exit (postconditions)
    pub exit_facts: Vec<ProvenFact>,
}

/// A parameter with constraints
#[derive(Debug, Clone)]
pub struct PirParam {
    pub name: String,
    pub ty: PirType,
    /// Constraints on this parameter
    pub constraints: Vec<ProvenFact>,
}

/// A proven fact with evidence
#[derive(Debug, Clone)]
pub struct ProvenFact {
    /// The proposition that is proven
    pub proposition: Proposition,

    /// How it was proven
    pub evidence: ProofEvidence,

    /// Unique identifier for this fact
    pub id: u32,
}

impl ProvenFact {
    pub fn from_precondition(prop: Proposition, id: u32) -> Self {
        Self {
            proposition: prop,
            evidence: ProofEvidence::Precondition,
            id,
        }
    }

    pub fn from_control_flow(prop: Proposition, id: u32) -> Self {
        Self {
            proposition: prop,
            evidence: ProofEvidence::ControlFlow,
            id,
        }
    }

    pub fn from_smt(prop: Proposition, query_hash: u64, id: u32) -> Self {
        Self {
            proposition: prop,
            evidence: ProofEvidence::SmtProof {
                query_hash,
                solver: "z3".to_string(),
            },
            id,
        }
    }
}

/// PIR expression with attached proofs
#[derive(Debug, Clone)]
pub struct PirExpr {
    /// Expression kind
    pub kind: PirExprKind,

    /// Facts proven at this expression (available from context)
    pub proven: Vec<ProvenFact>,

    /// Facts about the result of this expression
    pub result_facts: Vec<ProvenFact>,

    /// Expression type
    pub ty: PirType,

    /// Source location
    pub span: Option<crate::ast::Span>,
}

impl PirExpr {
    pub fn new(kind: PirExprKind, ty: PirType) -> Self {
        Self {
            kind,
            proven: Vec::new(),
            result_facts: Vec::new(),
            ty,
            span: None,
        }
    }

    pub fn with_proven(mut self, facts: Vec<ProvenFact>) -> Self {
        self.proven = facts;
        self
    }

    pub fn with_result_facts(mut self, facts: Vec<ProvenFact>) -> Self {
        self.result_facts = facts;
        self
    }

    /// Check if a bounds check can be eliminated
    pub fn has_bounds_proof(&self) -> bool {
        self.proven.iter().any(|f| {
            matches!(&f.proposition, Proposition::InBounds { .. })
        })
    }

    /// Check if a null check can be eliminated
    pub fn has_null_proof(&self) -> bool {
        self.proven.iter().any(|f| {
            matches!(&f.proposition, Proposition::NonNull(_))
        })
    }

    /// Check if a division check can be eliminated
    pub fn has_nonzero_proof(&self) -> bool {
        self.proven.iter().any(|f| {
            match &f.proposition {
                Proposition::Compare { op, rhs, .. } => {
                    matches!(op, crate::cir::CompareOp::Ne) &&
                    matches!(rhs.as_ref(), crate::cir::CirExpr::IntLit(0))
                }
                _ => false,
            }
        })
    }
}

/// PIR expression kinds
#[derive(Debug, Clone)]
pub enum PirExprKind {
    // === Literals ===
    IntLit(i64),
    FloatLit(u64),
    BoolLit(bool),
    StringLit(String),
    Unit,

    // === Variables ===
    Var(String),

    // === Binary operations ===
    BinOp {
        op: PirBinOp,
        lhs: Box<PirExpr>,
        rhs: Box<PirExpr>,
    },

    // === Unary operations ===
    UnaryOp {
        op: PirUnaryOp,
        operand: Box<PirExpr>,
    },

    // === Array indexing with bounds proof ===
    Index {
        array: Box<PirExpr>,
        index: Box<PirExpr>,
        /// If Some, bounds check can be eliminated
        bounds_proof: Option<ProvenFact>,
    },

    // === Field access with null proof ===
    Field {
        base: Box<PirExpr>,
        field: String,
        /// If Some, null check can be eliminated
        null_proof: Option<ProvenFact>,
    },

    // === Division with non-zero proof ===
    Div {
        lhs: Box<PirExpr>,
        rhs: Box<PirExpr>,
        /// If Some, zero check can be eliminated
        nonzero_proof: Option<ProvenFact>,
    },

    // === Control flow ===
    If {
        cond: Box<PirExpr>,
        then_branch: Box<PirExpr>,
        else_branch: Box<PirExpr>,
        /// Facts added in then branch (condition is true)
        then_facts: Vec<ProvenFact>,
        /// Facts added in else branch (condition is false)
        else_facts: Vec<ProvenFact>,
    },

    While {
        cond: Box<PirExpr>,
        body: Box<PirExpr>,
        /// Loop invariant facts
        invariant_facts: Vec<ProvenFact>,
    },

    Loop {
        body: Box<PirExpr>,
        /// Loop invariant facts
        invariant_facts: Vec<ProvenFact>,
    },

    For {
        var: String,
        iter: Box<PirExpr>,
        body: Box<PirExpr>,
        /// Facts about iteration variable
        iter_facts: Vec<ProvenFact>,
    },

    Break(Box<PirExpr>),
    Continue,

    // === Bindings ===
    Let {
        name: String,
        ty: PirType,
        value: Box<PirExpr>,
        body: Box<PirExpr>,
        /// Facts inherited by the variable
        value_facts: Vec<ProvenFact>,
    },

    LetMut {
        name: String,
        ty: PirType,
        value: Box<PirExpr>,
        body: Box<PirExpr>,
    },

    Assign {
        target: String,
        value: Box<PirExpr>,
    },

    // === Function call ===
    Call {
        func: String,
        args: Vec<PirExpr>,
        /// Facts from callee's postconditions
        postcondition_facts: Vec<ProvenFact>,
    },

    // === Compound expressions ===
    Block(Vec<PirExpr>),

    Struct {
        name: String,
        fields: Vec<(String, PirExpr)>,
    },

    Array(Vec<PirExpr>),

    Tuple(Vec<PirExpr>),

    // === References ===
    Ref(Box<PirExpr>),
    RefMut(Box<PirExpr>),
    Deref(Box<PirExpr>),

    // === Other ===
    Cast {
        expr: Box<PirExpr>,
        ty: PirType,
    },

    Len(Box<PirExpr>),
}

/// PIR binary operators
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PirBinOp {
    Add,
    Sub,
    Mul,
    Mod,
    // Division is separate because it needs non-zero proof
    Lt,
    Le,
    Gt,
    Ge,
    Eq,
    Ne,
    And,
    Or,
    BitAnd,
    BitOr,
    BitXor,
    Shl,
    Shr,
}

/// PIR unary operators
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PirUnaryOp {
    Neg,
    Not,
    BitNot,
}

/// PIR types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PirType {
    Bool,
    I8, I16, I32, I64, I128,
    U8, U16, U32, U64, U128,
    F32, F64,
    Char,
    String,
    Unit,
    Array(Box<PirType>, Option<usize>),
    Slice(Box<PirType>),
    Ref(Box<PirType>),
    RefMut(Box<PirType>),
    Option(Box<PirType>),
    Tuple(Vec<PirType>),
    Struct(String),
    Enum(String),
    Fn {
        params: Vec<PirType>,
        ret: Box<PirType>,
    },
    Never,
    Infer,
}

impl PirType {
    /// Convert from CIR type
    pub fn from_cir(cir_ty: &crate::cir::CirType) -> Self {
        use crate::cir::CirType;

        match cir_ty {
            CirType::Bool => PirType::Bool,
            CirType::I8 => PirType::I8,
            CirType::I16 => PirType::I16,
            CirType::I32 => PirType::I32,
            CirType::I64 => PirType::I64,
            CirType::I128 => PirType::I128,
            CirType::U8 => PirType::U8,
            CirType::U16 => PirType::U16,
            CirType::U32 => PirType::U32,
            CirType::U64 => PirType::U64,
            CirType::U128 => PirType::U128,
            CirType::F32 => PirType::F32,
            CirType::F64 => PirType::F64,
            CirType::Char => PirType::Char,
            CirType::String => PirType::String,
            CirType::Unit => PirType::Unit,
            CirType::Array(elem, size) => {
                PirType::Array(Box::new(PirType::from_cir(elem)), Some(*size))
            }
            CirType::Slice(elem) => PirType::Slice(Box::new(PirType::from_cir(elem))),
            CirType::Ref(inner) => PirType::Ref(Box::new(PirType::from_cir(inner))),
            CirType::RefMut(inner) => PirType::RefMut(Box::new(PirType::from_cir(inner))),
            CirType::Option(inner) => PirType::Option(Box::new(PirType::from_cir(inner))),
            CirType::Tuple(elems) => {
                PirType::Tuple(elems.iter().map(PirType::from_cir).collect())
            }
            CirType::Struct(name) => PirType::Struct(name.clone()),
            CirType::Enum(name) => PirType::Enum(name.clone()),
            CirType::Fn { params, ret } => PirType::Fn {
                params: params.iter().map(PirType::from_cir).collect(),
                ret: Box::new(PirType::from_cir(ret)),
            },
            CirType::Never => PirType::Never,
            CirType::Infer => PirType::Infer,
            CirType::TypeParam(name) => PirType::Struct(name.clone()), // Simplified
            CirType::Generic(name, _) => PirType::Struct(name.clone()),
            CirType::Ptr(inner) => PirType::Ref(Box::new(PirType::from_cir(inner))),
            CirType::Range(inner) => PirType::Tuple(vec![
                PirType::from_cir(inner),
                PirType::from_cir(inner),
            ]),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cir::{CirExpr, CompareOp};

    #[test]
    fn test_pir_program_creation() {
        let program = PirProgram::new();
        assert!(program.functions.is_empty());
    }

    #[test]
    fn test_pir_expr_with_proofs() {
        let fact = ProvenFact::from_precondition(Proposition::True, 1);
        let expr = PirExpr::new(PirExprKind::IntLit(42), PirType::I64)
            .with_proven(vec![fact.clone()])
            .with_result_facts(vec![fact]);

        assert_eq!(expr.proven.len(), 1);
        assert_eq!(expr.result_facts.len(), 1);
    }

    #[test]
    fn test_bounds_proof_detection() {
        let bounds_fact = ProvenFact::from_control_flow(
            Proposition::InBounds {
                index: Box::new(CirExpr::Var("i".to_string())),
                array: Box::new(CirExpr::Var("arr".to_string())),
            },
            1,
        );

        let expr = PirExpr::new(
            PirExprKind::Index {
                array: Box::new(PirExpr::new(PirExprKind::Var("arr".to_string()), PirType::Slice(Box::new(PirType::I64)))),
                index: Box::new(PirExpr::new(PirExprKind::Var("i".to_string()), PirType::I64)),
                bounds_proof: Some(bounds_fact.clone()),
            },
            PirType::I64,
        ).with_proven(vec![bounds_fact]);

        assert!(expr.has_bounds_proof());
    }

    #[test]
    fn test_null_proof_detection() {
        let null_fact = ProvenFact::from_precondition(
            Proposition::NonNull(Box::new(CirExpr::Var("ptr".to_string()))),
            1,
        );

        let expr = PirExpr::new(
            PirExprKind::Var("ptr".to_string()),
            PirType::Ref(Box::new(PirType::I64)),
        ).with_proven(vec![null_fact]);

        assert!(expr.has_null_proof());
    }

    #[test]
    fn test_pir_type_from_cir() {
        use crate::cir::CirType;

        assert_eq!(PirType::from_cir(&CirType::I64), PirType::I64);
        assert_eq!(PirType::from_cir(&CirType::Bool), PirType::Bool);

        let arr_ty = CirType::Array(Box::new(CirType::I64), 10);
        assert_eq!(
            PirType::from_cir(&arr_ty),
            PirType::Array(Box::new(PirType::I64), Some(10))
        );
    }
}
