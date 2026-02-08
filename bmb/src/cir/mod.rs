//! Contract Intermediate Representation (CIR)
//!
//! CIR is a semantic IR where contracts are first-class logical propositions.
//! It sits between Typed AST and PIR/MIR, providing a normalized representation
//! of contracts that can be:
//! 1. Verified by SMT solvers (Z3)
//! 2. Propagated through the program
//! 3. Used to guide optimizations
//!
//! # Design Philosophy
//!
//! > "Proofs are not for safety. Proofs are for speed."
//!
//! CIR captures all semantic information needed for contract verification,
//! including:
//! - Preconditions (caller guarantees)
//! - Postconditions (callee guarantees)
//! - Loop invariants
//! - Type invariants
//! - Effect annotations
//!
//! # v0.52: Initial Implementation

mod lower;
mod output;
pub mod smt;
pub mod verify;
pub mod to_mir_facts;

pub use lower::lower_to_cir;
pub use output::CirOutput;
pub use smt::{CirSmtGenerator, SmtSort, SmtError};
pub use verify::{CirVerifier, CirVerificationReport, ProofWitness, ProofOutcome};
pub use to_mir_facts::{
    proposition_to_facts, extract_precondition_facts, extract_postcondition_facts,
    extract_all_facts, extract_verified_facts,
};

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// A CIR program containing all functions with their contracts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CirProgram {
    /// Functions with extracted contracts
    pub functions: Vec<CirFunction>,

    /// External function declarations
    pub extern_fns: Vec<CirExternFn>,

    /// Struct definitions with type invariants
    pub structs: HashMap<String, CirStruct>,

    /// Type invariants (constraints that always hold for a type)
    pub type_invariants: HashMap<String, Vec<Proposition>>,
}

/// External function declaration with effect information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CirExternFn {
    /// Module name (e.g., "wasi_snapshot_preview1")
    pub module: String,
    /// Function name
    pub name: String,
    /// Parameter types
    pub params: Vec<CirType>,
    /// Return type
    pub ret_ty: CirType,
    /// Effect classification
    pub effects: EffectSet,
}

/// Struct definition with field types and invariants
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CirStruct {
    pub name: String,
    pub fields: Vec<(String, CirType)>,
    /// Structural invariants (e.g., start <= end for Range)
    pub invariants: Vec<Proposition>,
}

/// A CIR function with explicit contract information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CirFunction {
    /// Function name
    pub name: String,

    /// Type parameters (generics)
    pub type_params: Vec<String>,

    /// Function parameters with types
    pub params: Vec<CirParam>,

    /// Return type
    pub ret_ty: CirType,

    /// Return value binding name (for postconditions)
    pub ret_name: String,

    /// Preconditions - what the caller must guarantee
    pub preconditions: Vec<NamedProposition>,

    /// Postconditions - what the function guarantees
    pub postconditions: Vec<NamedProposition>,

    /// Loop invariants extracted from the body
    pub loop_invariants: Vec<LoopInvariant>,

    /// Effect classification
    pub effects: EffectSet,

    /// Function body as CIR expression
    pub body: CirExpr,
}

/// Function parameter with type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CirParam {
    pub name: String,
    pub ty: CirType,
    /// Parameter-specific constraints (e.g., non-null)
    pub constraints: Vec<Proposition>,
}

/// Named proposition for better error messages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NamedProposition {
    /// Optional name (e.g., "sorted_input", "valid_index")
    pub name: Option<String>,
    /// The logical proposition
    pub proposition: Proposition,
}

/// Loop invariant with location information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoopInvariant {
    /// Loop identifier (for nested loops)
    pub loop_id: usize,
    /// The invariant proposition
    pub invariant: Proposition,
}

// ============================================================================
// Propositions - First-Order Logic Representation
// ============================================================================

/// Logical proposition in first-order logic
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Proposition {
    /// Boolean constants
    True,
    False,

    /// Comparison: expr1 op expr2
    Compare {
        lhs: Box<CirExpr>,
        op: CompareOp,
        rhs: Box<CirExpr>,
    },

    /// Logical NOT: ¬P
    Not(Box<Proposition>),

    /// Logical AND: P₁ ∧ P₂ ∧ ... ∧ Pₙ
    And(Vec<Proposition>),

    /// Logical OR: P₁ ∨ P₂ ∨ ... ∨ Pₙ
    Or(Vec<Proposition>),

    /// Implication: P → Q
    Implies(Box<Proposition>, Box<Proposition>),

    /// Universal quantification: ∀x:T. P
    Forall {
        var: String,
        ty: CirType,
        body: Box<Proposition>,
    },

    /// Existential quantification: ∃x:T. P
    Exists {
        var: String,
        ty: CirType,
        body: Box<Proposition>,
    },

    /// Pure function call as predicate (e.g., is_sorted(arr))
    Predicate {
        name: String,
        args: Vec<CirExpr>,
    },

    /// Array bounds check: 0 <= index < len(array)
    InBounds {
        index: Box<CirExpr>,
        array: Box<CirExpr>,
    },

    /// Non-null check: ptr != null
    NonNull(Box<CirExpr>),

    /// Old value reference (for postconditions): old(expr)
    Old(Box<CirExpr>, Box<Proposition>),
}

impl Proposition {
    /// Create a simple comparison proposition
    pub fn compare(lhs: CirExpr, op: CompareOp, rhs: CirExpr) -> Self {
        Proposition::Compare {
            lhs: Box::new(lhs),
            op,
            rhs: Box::new(rhs),
        }
    }

    /// Create conjunction of propositions
    pub fn and(props: impl IntoIterator<Item = Proposition>) -> Self {
        let props: Vec<_> = props.into_iter().collect();
        if props.is_empty() {
            Proposition::True
        } else if props.len() == 1 {
            props.into_iter().next().unwrap()
        } else {
            Proposition::And(props)
        }
    }

    /// Create disjunction of propositions
    pub fn or(props: impl IntoIterator<Item = Proposition>) -> Self {
        let props: Vec<_> = props.into_iter().collect();
        if props.is_empty() {
            Proposition::False
        } else if props.len() == 1 {
            props.into_iter().next().unwrap()
        } else {
            Proposition::Or(props)
        }
    }

    /// Negate a proposition
    pub fn not(p: Proposition) -> Self {
        match p {
            Proposition::True => Proposition::False,
            Proposition::False => Proposition::True,
            Proposition::Not(inner) => *inner,
            _ => Proposition::Not(Box::new(p)),
        }
    }

    /// Check if proposition is trivially true
    pub fn is_trivially_true(&self) -> bool {
        matches!(self, Proposition::True)
    }

    /// Check if proposition is trivially false
    pub fn is_trivially_false(&self) -> bool {
        matches!(self, Proposition::False)
    }
}

/// Comparison operators
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CompareOp {
    Lt,  // <
    Le,  // <=
    Gt,  // >
    Ge,  // >=
    Eq,  // ==
    Ne,  // !=
}

impl CompareOp {
    /// Negate the comparison operator
    pub fn negate(self) -> Self {
        match self {
            CompareOp::Lt => CompareOp::Ge,
            CompareOp::Le => CompareOp::Gt,
            CompareOp::Gt => CompareOp::Le,
            CompareOp::Ge => CompareOp::Lt,
            CompareOp::Eq => CompareOp::Ne,
            CompareOp::Ne => CompareOp::Eq,
        }
    }

    /// Flip the operands (swap lhs and rhs)
    pub fn flip(self) -> Self {
        match self {
            CompareOp::Lt => CompareOp::Gt,
            CompareOp::Le => CompareOp::Ge,
            CompareOp::Gt => CompareOp::Lt,
            CompareOp::Ge => CompareOp::Le,
            CompareOp::Eq => CompareOp::Eq,
            CompareOp::Ne => CompareOp::Ne,
        }
    }
}

impl std::fmt::Display for CompareOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CompareOp::Lt => write!(f, "<"),
            CompareOp::Le => write!(f, "<="),
            CompareOp::Gt => write!(f, ">"),
            CompareOp::Ge => write!(f, ">="),
            CompareOp::Eq => write!(f, "=="),
            CompareOp::Ne => write!(f, "!="),
        }
    }
}

// ============================================================================
// CIR Expressions
// ============================================================================

/// CIR expression - semantic representation of computations
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CirExpr {
    /// Integer literal
    IntLit(i64),

    /// Float literal (represented as bits for Eq)
    FloatLit(u64),

    /// Boolean literal
    BoolLit(bool),

    /// String literal
    StringLit(String),

    /// Variable reference
    Var(String),

    /// Binary operation
    BinOp {
        op: BinOp,
        lhs: Box<CirExpr>,
        rhs: Box<CirExpr>,
    },

    /// Unary operation
    UnaryOp {
        op: UnaryOp,
        operand: Box<CirExpr>,
    },

    /// Function call
    Call {
        func: String,
        args: Vec<CirExpr>,
    },

    /// Array/slice indexing
    Index {
        base: Box<CirExpr>,
        index: Box<CirExpr>,
    },

    /// Index assignment: arr[i] = value
    IndexAssign {
        array: Box<CirExpr>,
        index: Box<CirExpr>,
        value: Box<CirExpr>,
    },

    /// Field access
    Field {
        base: Box<CirExpr>,
        field: String,
    },

    /// Field assignment: obj.field = value
    FieldAssign {
        object: Box<CirExpr>,
        field: String,
        value: Box<CirExpr>,
    },

    /// Dereference store: *ptr = value (v0.60.21)
    DerefStore {
        ptr: Box<CirExpr>,
        value: Box<CirExpr>,
    },

    /// If-then-else expression
    If {
        cond: Box<CirExpr>,
        then_branch: Box<CirExpr>,
        else_branch: Box<CirExpr>,
    },

    /// Let binding
    Let {
        name: String,
        ty: CirType,
        value: Box<CirExpr>,
        body: Box<CirExpr>,
    },

    /// Mutable let binding
    LetMut {
        name: String,
        ty: CirType,
        value: Box<CirExpr>,
        body: Box<CirExpr>,
    },

    /// Assignment to mutable variable
    Assign {
        target: String,
        value: Box<CirExpr>,
    },

    /// While loop
    While {
        cond: Box<CirExpr>,
        body: Box<CirExpr>,
        /// Loop invariant (if specified)
        invariant: Option<Proposition>,
    },

    /// Infinite loop
    Loop {
        body: Box<CirExpr>,
    },

    /// For loop
    For {
        var: String,
        iter: Box<CirExpr>,
        body: Box<CirExpr>,
    },

    /// Break from loop
    Break(Box<CirExpr>),

    /// Continue to next iteration
    Continue,

    /// Block expression (sequence of expressions)
    Block(Vec<CirExpr>),

    /// Struct construction
    Struct {
        name: String,
        fields: Vec<(String, CirExpr)>,
    },

    /// Array literal
    Array(Vec<CirExpr>),

    /// Tuple literal
    Tuple(Vec<CirExpr>),

    /// Reference creation &expr
    Ref(Box<CirExpr>),

    /// Mutable reference &mut expr
    RefMut(Box<CirExpr>),

    /// Dereference *expr
    Deref(Box<CirExpr>),

    /// Range expression start..end or start..=end
    Range {
        start: Box<CirExpr>,
        end: Box<CirExpr>,
        inclusive: bool,
    },

    /// Enum variant construction
    EnumVariant {
        enum_name: String,
        variant: String,
        args: Vec<CirExpr>,
    },

    /// State reference (pre/post)
    StateRef {
        expr: Box<CirExpr>,
        is_pre: bool,
    },

    /// Closure expression
    Closure {
        params: Vec<String>,
        body: Box<CirExpr>,
    },

    /// Type cast
    Cast {
        expr: Box<CirExpr>,
        ty: CirType,
    },

    /// Sizeof type
    Sizeof(CirType),

    /// Forall expression (as boolean)
    Forall {
        var: String,
        ty: CirType,
        body: Box<CirExpr>,
    },

    /// Exists expression (as boolean)
    Exists {
        var: String,
        ty: CirType,
        body: Box<CirExpr>,
    },

    /// Todo placeholder
    Todo(Option<String>),

    /// Array length
    Len(Box<CirExpr>),

    /// Old value (for postconditions)
    Old(Box<CirExpr>),

    /// Unit value
    Unit,
}

impl CirExpr {
    /// Create a variable reference
    pub fn var(name: impl Into<String>) -> Self {
        CirExpr::Var(name.into())
    }

    /// Create an integer literal
    pub fn int(n: i64) -> Self {
        CirExpr::IntLit(n)
    }

    /// Create a binary operation
    pub fn binop(op: BinOp, lhs: CirExpr, rhs: CirExpr) -> Self {
        CirExpr::BinOp {
            op,
            lhs: Box::new(lhs),
            rhs: Box::new(rhs),
        }
    }
}

/// Binary operators
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BinOp {
    // Arithmetic
    Add, Sub, Mul, Div, Mod,
    // Wrapping arithmetic
    AddWrap, SubWrap, MulWrap,
    // Checked arithmetic
    AddChecked, SubChecked, MulChecked,
    // Saturating arithmetic
    AddSat, SubSat, MulSat,
    // Comparison
    Lt, Le, Gt, Ge, Eq, Ne,
    // Logical
    And, Or, Implies,
    // Bitwise
    BitAnd, BitOr, BitXor, Shl, Shr,
}

/// Unary operators
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UnaryOp {
    Neg,    // -x
    Not,    // !x (logical)
    BitNot, // ~x (bitwise)
}

// ============================================================================
// CIR Types
// ============================================================================

/// CIR type representation
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CirType {
    /// Unit type ()
    Unit,
    /// Boolean
    Bool,
    /// 8-bit signed integer
    I8,
    /// 16-bit signed integer
    I16,
    /// 32-bit signed integer
    I32,
    /// 64-bit signed integer
    I64,
    /// 128-bit signed integer
    I128,
    /// 8-bit unsigned integer
    U8,
    /// 16-bit unsigned integer
    U16,
    /// 32-bit unsigned integer
    U32,
    /// 64-bit unsigned integer
    U64,
    /// 128-bit unsigned integer
    U128,
    /// 32-bit floating point
    F32,
    /// 64-bit floating point
    F64,
    /// Character
    Char,
    /// String
    String,
    /// Array type [T; n]
    Array(Box<CirType>, usize),
    /// Slice type [T]
    Slice(Box<CirType>),
    /// Reference type &T
    Ref(Box<CirType>),
    /// Mutable reference &mut T
    RefMut(Box<CirType>),
    /// Optional type T?
    Option(Box<CirType>),
    /// Range type Range<T>
    Range(Box<CirType>),
    /// Struct type
    Struct(String),
    /// Enum type
    Enum(String),
    /// Type parameter
    TypeParam(String),
    /// Generic type with type arguments
    Generic(String, Vec<CirType>),
    /// Tuple type
    Tuple(Vec<CirType>),
    /// Function type (params) -> ret
    Fn {
        params: Vec<CirType>,
        ret: Box<CirType>,
    },
    /// Raw pointer (for FFI)
    Ptr(Box<CirType>),
    /// Type inference placeholder
    Infer,
    /// Never type (unreachable)
    Never,
}

impl std::fmt::Display for CirType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CirType::Unit => write!(f, "()"),
            CirType::Bool => write!(f, "bool"),
            CirType::I8 => write!(f, "i8"),
            CirType::I16 => write!(f, "i16"),
            CirType::I32 => write!(f, "i32"),
            CirType::I64 => write!(f, "i64"),
            CirType::I128 => write!(f, "i128"),
            CirType::U8 => write!(f, "u8"),
            CirType::U16 => write!(f, "u16"),
            CirType::U32 => write!(f, "u32"),
            CirType::U64 => write!(f, "u64"),
            CirType::U128 => write!(f, "u128"),
            CirType::F32 => write!(f, "f32"),
            CirType::F64 => write!(f, "f64"),
            CirType::Char => write!(f, "char"),
            CirType::String => write!(f, "String"),
            CirType::Array(ty, n) => write!(f, "[{}; {}]", ty, n),
            CirType::Slice(ty) => write!(f, "[{}]", ty),
            CirType::Ref(ty) => write!(f, "&{}", ty),
            CirType::RefMut(ty) => write!(f, "&mut {}", ty),
            CirType::Option(ty) => write!(f, "{}?", ty),
            CirType::Range(ty) => write!(f, "Range<{}>", ty),
            CirType::Struct(name) => write!(f, "{}", name),
            CirType::Enum(name) => write!(f, "{}", name),
            CirType::TypeParam(name) => write!(f, "{}", name),
            CirType::Generic(name, args) => {
                write!(f, "{}<", name)?;
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{}", arg)?;
                }
                write!(f, ">")
            }
            CirType::Tuple(elems) => {
                write!(f, "(")?;
                for (i, elem) in elems.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{}", elem)?;
                }
                write!(f, ")")
            }
            CirType::Fn { params, ret } => {
                write!(f, "(")?;
                for (i, p) in params.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{}", p)?;
                }
                write!(f, ") -> {}", ret)
            }
            CirType::Ptr(ty) => write!(f, "*{}", ty),
            CirType::Infer => write!(f, "_"),
            CirType::Never => write!(f, "!"),
        }
    }
}

// ============================================================================
// Effect System
// ============================================================================

/// Effect classification for functions
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct EffectSet {
    /// Function has no side effects (deterministic, no memory writes)
    pub is_pure: bool,
    /// Function can be evaluated at compile time
    pub is_const: bool,
    /// Function reads from memory/globals
    pub reads: bool,
    /// Function writes to memory/globals
    pub writes: bool,
    /// Function performs I/O
    pub io: bool,
    /// Function may allocate memory
    pub allocates: bool,
    /// Function may not terminate
    pub diverges: bool,
}

impl EffectSet {
    /// Create a pure effect set (no side effects)
    pub fn pure() -> Self {
        Self {
            is_pure: true,
            is_const: false,
            reads: false,
            writes: false,
            io: false,
            allocates: false,
            diverges: false,
        }
    }

    /// Create a const effect set (compile-time evaluatable)
    pub fn const_() -> Self {
        Self {
            is_pure: true,
            is_const: true,
            reads: false,
            writes: false,
            io: false,
            allocates: false,
            diverges: false,
        }
    }

    /// Create an impure effect set (may have side effects)
    pub fn impure() -> Self {
        Self::default()
    }

    /// Combine two effect sets (union of effects)
    pub fn union(&self, other: &Self) -> Self {
        Self {
            is_pure: self.is_pure && other.is_pure,
            is_const: self.is_const && other.is_const,
            reads: self.reads || other.reads,
            writes: self.writes || other.writes,
            io: self.io || other.io,
            allocates: self.allocates || other.allocates,
            diverges: self.diverges || other.diverges,
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_proposition_and_empty() {
        let p = Proposition::and(vec![]);
        assert!(matches!(p, Proposition::True));
    }

    #[test]
    fn test_proposition_and_single() {
        let p = Proposition::and(vec![Proposition::True]);
        assert!(matches!(p, Proposition::True));
    }

    #[test]
    fn test_proposition_not_double_negation() {
        let p = Proposition::not(Proposition::not(Proposition::True));
        assert!(matches!(p, Proposition::True));
    }

    #[test]
    fn test_compare_op_negate() {
        assert_eq!(CompareOp::Lt.negate(), CompareOp::Ge);
        assert_eq!(CompareOp::Eq.negate(), CompareOp::Ne);
    }

    #[test]
    fn test_compare_op_flip() {
        assert_eq!(CompareOp::Lt.flip(), CompareOp::Gt);
        assert_eq!(CompareOp::Eq.flip(), CompareOp::Eq);
    }

    #[test]
    fn test_cir_type_display() {
        assert_eq!(CirType::I64.to_string(), "i64");
        assert_eq!(CirType::Ref(Box::new(CirType::I64)).to_string(), "&i64");
        assert_eq!(CirType::Option(Box::new(CirType::String)).to_string(), "String?");
    }

    #[test]
    fn test_effect_set_pure() {
        let effects = EffectSet::pure();
        assert!(effects.is_pure);
        assert!(!effects.writes);
    }

    #[test]
    fn test_effect_set_union() {
        let pure = EffectSet::pure();
        let writes = EffectSet {
            writes: true,
            ..Default::default()
        };
        let combined = pure.union(&writes);
        assert!(!combined.is_pure);
        assert!(combined.writes);
    }

    // --- Cycle 63: Additional CIR tests ---

    #[test]
    fn test_proposition_or_empty() {
        let p = Proposition::or(vec![]);
        assert!(matches!(p, Proposition::False));
    }

    #[test]
    fn test_proposition_or_single() {
        let p = Proposition::or(vec![Proposition::True]);
        assert!(matches!(p, Proposition::True));
    }

    #[test]
    fn test_proposition_or_multiple() {
        let p = Proposition::or(vec![Proposition::True, Proposition::False]);
        assert!(matches!(p, Proposition::Or(_)));
    }

    #[test]
    fn test_proposition_and_multiple() {
        let p = Proposition::and(vec![Proposition::True, Proposition::True]);
        assert!(matches!(p, Proposition::And(_)));
    }

    #[test]
    fn test_proposition_not_true_becomes_false() {
        let p = Proposition::not(Proposition::True);
        assert!(matches!(p, Proposition::False));
    }

    #[test]
    fn test_proposition_not_false_becomes_true() {
        let p = Proposition::not(Proposition::False);
        assert!(matches!(p, Proposition::True));
    }

    #[test]
    fn test_proposition_is_trivially_true() {
        assert!(Proposition::True.is_trivially_true());
        assert!(!Proposition::False.is_trivially_true());
        assert!(!Proposition::Not(Box::new(Proposition::True)).is_trivially_true());
    }

    #[test]
    fn test_proposition_is_trivially_false() {
        assert!(Proposition::False.is_trivially_false());
        assert!(!Proposition::True.is_trivially_false());
        assert!(!Proposition::Not(Box::new(Proposition::False)).is_trivially_false());
    }

    #[test]
    fn test_proposition_compare_constructor() {
        let p = Proposition::compare(
            CirExpr::var("x"),
            CompareOp::Lt,
            CirExpr::int(10),
        );
        match p {
            Proposition::Compare { lhs, op, rhs } => {
                assert_eq!(*lhs, CirExpr::Var("x".to_string()));
                assert_eq!(op, CompareOp::Lt);
                assert_eq!(*rhs, CirExpr::IntLit(10));
            }
            _ => panic!("Expected Compare"),
        }
    }

    #[test]
    fn test_compare_op_negate_all() {
        assert_eq!(CompareOp::Lt.negate(), CompareOp::Ge);
        assert_eq!(CompareOp::Le.negate(), CompareOp::Gt);
        assert_eq!(CompareOp::Gt.negate(), CompareOp::Le);
        assert_eq!(CompareOp::Ge.negate(), CompareOp::Lt);
        assert_eq!(CompareOp::Eq.negate(), CompareOp::Ne);
        assert_eq!(CompareOp::Ne.negate(), CompareOp::Eq);
    }

    #[test]
    fn test_compare_op_flip_all() {
        assert_eq!(CompareOp::Lt.flip(), CompareOp::Gt);
        assert_eq!(CompareOp::Le.flip(), CompareOp::Ge);
        assert_eq!(CompareOp::Gt.flip(), CompareOp::Lt);
        assert_eq!(CompareOp::Ge.flip(), CompareOp::Le);
        assert_eq!(CompareOp::Eq.flip(), CompareOp::Eq);
        assert_eq!(CompareOp::Ne.flip(), CompareOp::Ne);
    }

    #[test]
    fn test_compare_op_double_negate() {
        for op in [CompareOp::Lt, CompareOp::Le, CompareOp::Gt, CompareOp::Ge, CompareOp::Eq, CompareOp::Ne] {
            assert_eq!(op.negate().negate(), op);
        }
    }

    #[test]
    fn test_compare_op_display() {
        assert_eq!(CompareOp::Lt.to_string(), "<");
        assert_eq!(CompareOp::Le.to_string(), "<=");
        assert_eq!(CompareOp::Gt.to_string(), ">");
        assert_eq!(CompareOp::Ge.to_string(), ">=");
        assert_eq!(CompareOp::Eq.to_string(), "==");
        assert_eq!(CompareOp::Ne.to_string(), "!=");
    }

    #[test]
    fn test_cir_type_display_numeric() {
        assert_eq!(CirType::I8.to_string(), "i8");
        assert_eq!(CirType::I16.to_string(), "i16");
        assert_eq!(CirType::I32.to_string(), "i32");
        assert_eq!(CirType::I128.to_string(), "i128");
        assert_eq!(CirType::U8.to_string(), "u8");
        assert_eq!(CirType::U16.to_string(), "u16");
        assert_eq!(CirType::U32.to_string(), "u32");
        assert_eq!(CirType::U64.to_string(), "u64");
        assert_eq!(CirType::U128.to_string(), "u128");
        assert_eq!(CirType::F32.to_string(), "f32");
        assert_eq!(CirType::F64.to_string(), "f64");
    }

    #[test]
    fn test_cir_type_display_complex() {
        assert_eq!(CirType::Bool.to_string(), "bool");
        assert_eq!(CirType::Unit.to_string(), "()");
        assert_eq!(CirType::Char.to_string(), "char");
        assert_eq!(CirType::String.to_string(), "String");
        assert_eq!(CirType::Never.to_string(), "!");
        assert_eq!(CirType::Infer.to_string(), "_");
        assert_eq!(CirType::Ptr(Box::new(CirType::I64)).to_string(), "*i64");
        assert_eq!(CirType::RefMut(Box::new(CirType::I64)).to_string(), "&mut i64");
        assert_eq!(CirType::Slice(Box::new(CirType::U8)).to_string(), "[u8]");
        assert_eq!(CirType::Array(Box::new(CirType::I32), 10).to_string(), "[i32; 10]");
        assert_eq!(CirType::Range(Box::new(CirType::I64)).to_string(), "Range<i64>");
    }

    #[test]
    fn test_cir_type_display_composite() {
        assert_eq!(CirType::Struct("Point".to_string()).to_string(), "Point");
        assert_eq!(CirType::Enum("Color".to_string()).to_string(), "Color");
        assert_eq!(CirType::TypeParam("T".to_string()).to_string(), "T");

        let tuple = CirType::Tuple(vec![CirType::I64, CirType::Bool]);
        assert_eq!(tuple.to_string(), "(i64, bool)");

        let generic = CirType::Generic("Vec".to_string(), vec![CirType::I64]);
        assert_eq!(generic.to_string(), "Vec<i64>");

        let fn_type = CirType::Fn {
            params: vec![CirType::I64, CirType::Bool],
            ret: Box::new(CirType::String),
        };
        assert_eq!(fn_type.to_string(), "(i64, bool) -> String");
    }

    #[test]
    fn test_cir_expr_var_constructor() {
        assert_eq!(CirExpr::var("x"), CirExpr::Var("x".to_string()));
    }

    #[test]
    fn test_cir_expr_int_constructor() {
        assert_eq!(CirExpr::int(42), CirExpr::IntLit(42));
        assert_eq!(CirExpr::int(-1), CirExpr::IntLit(-1));
    }

    #[test]
    fn test_cir_expr_binop_constructor() {
        let expr = CirExpr::binop(BinOp::Add, CirExpr::int(1), CirExpr::int(2));
        match expr {
            CirExpr::BinOp { op, lhs, rhs } => {
                assert_eq!(op, BinOp::Add);
                assert_eq!(*lhs, CirExpr::IntLit(1));
                assert_eq!(*rhs, CirExpr::IntLit(2));
            }
            _ => panic!("Expected BinOp"),
        }
    }

    #[test]
    fn test_effect_set_const() {
        let effects = EffectSet::const_();
        assert!(effects.is_pure);
        assert!(effects.is_const);
        assert!(!effects.writes);
        assert!(!effects.io);
    }

    #[test]
    fn test_effect_set_impure() {
        let effects = EffectSet::impure();
        assert!(!effects.is_pure);
        assert!(!effects.is_const);
    }

    #[test]
    fn test_effect_set_union_preserves_pure() {
        let a = EffectSet::pure();
        let b = EffectSet::pure();
        let c = a.union(&b);
        assert!(c.is_pure);
        assert!(!c.writes);
    }

    #[test]
    fn test_effect_set_union_const_with_impure() {
        let a = EffectSet::const_();
        let b = EffectSet { io: true, ..Default::default() };
        let c = a.union(&b);
        assert!(!c.is_pure);
        assert!(!c.is_const);
        assert!(c.io);
    }

    #[test]
    fn test_effect_set_union_accumulates() {
        let a = EffectSet { reads: true, ..Default::default() };
        let b = EffectSet { writes: true, ..Default::default() };
        let c = EffectSet { allocates: true, ..Default::default() };
        let combined = a.union(&b).union(&c);
        assert!(combined.reads);
        assert!(combined.writes);
        assert!(combined.allocates);
    }
}
