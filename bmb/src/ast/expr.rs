//! Expression AST nodes

use super::{Spanned, Type};
use serde::{Deserialize, Serialize};

/// Expression
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Expr {
    /// Integer literal
    IntLit(i64),
    /// Float literal
    FloatLit(f64),
    /// Boolean literal
    BoolLit(bool),
    /// String literal (v0.5 Phase 2)
    StringLit(String),
    /// Unit value
    Unit,

    /// Variable reference
    Var(String),

    /// Binary operation
    Binary {
        left: Box<Spanned<Expr>>,
        op: BinOp,
        right: Box<Spanned<Expr>>,
    },

    /// Unary operation
    Unary {
        op: UnOp,
        expr: Box<Spanned<Expr>>,
    },

    /// Conditional: if cond then then_branch else else_branch
    If {
        cond: Box<Spanned<Expr>>,
        then_branch: Box<Spanned<Expr>>,
        else_branch: Box<Spanned<Expr>>,
    },

    /// Let binding: `let [mut] name = value; body`
    Let {
        name: String,
        mutable: bool,
        ty: Option<Spanned<Type>>,
        value: Box<Spanned<Expr>>,
        body: Box<Spanned<Expr>>,
    },

    /// Assignment: name = value (v0.5 Phase 2)
    Assign {
        name: String,
        value: Box<Spanned<Expr>>,
    },

    /// While loop: while cond { body } (v0.5 Phase 2)
    While {
        cond: Box<Spanned<Expr>>,
        body: Box<Spanned<Expr>>,
    },

    /// For loop: for var in iter { body } (v0.5 Phase 3)
    For {
        var: String,
        iter: Box<Spanned<Expr>>,
        body: Box<Spanned<Expr>>,
    },

    /// Range expression: start..end, start..<end, start..=end (v0.2)
    Range {
        start: Box<Spanned<Expr>>,
        end: Box<Spanned<Expr>>,
        kind: RangeKind,
    },

    /// Function call
    Call {
        func: String,
        args: Vec<Spanned<Expr>>,
    },

    /// Block: { expr1; expr2; ...; result }
    Block(Vec<Spanned<Expr>>),

    /// Return value reference (for post conditions)
    Ret,

    /// Refinement self-reference (v0.2): for T{constraints}
    /// Refers to the value being refined
    It,

    // v0.5: Struct and Enum expressions

    /// Struct initialization: new StructName { field1: value1, field2: value2 }
    StructInit {
        name: String,
        fields: Vec<(Spanned<String>, Spanned<Expr>)>,
    },

    /// Field access: expr.field
    FieldAccess {
        expr: Box<Spanned<Expr>>,
        field: Spanned<String>,
    },

    /// Enum variant: EnumName::Variant or EnumName::Variant(args)
    EnumVariant {
        enum_name: String,
        variant: String,
        args: Vec<Spanned<Expr>>,
    },

    /// Match expression
    Match {
        expr: Box<Spanned<Expr>>,
        arms: Vec<MatchArm>,
    },

    // v0.5 Phase 5: References

    /// Create reference: &expr
    Ref(Box<Spanned<Expr>>),

    /// Create mutable reference: &mut expr
    RefMut(Box<Spanned<Expr>>),

    /// Dereference: *expr
    Deref(Box<Spanned<Expr>>),

    // v0.5 Phase 6: Arrays

    /// Array literal: [elem1, elem2, ...]
    ArrayLit(Vec<Spanned<Expr>>),

    /// Index access: `expr[index]`
    Index {
        expr: Box<Spanned<Expr>>,
        index: Box<Spanned<Expr>>,
    },

    // v0.5 Phase 8: Method calls

    /// Method call: expr.method(args) (v0.5 Phase 8)
    MethodCall {
        receiver: Box<Spanned<Expr>>,
        method: String,
        args: Vec<Spanned<Expr>>,
    },

    // v0.2: State references for contracts

    /// State reference: expr.pre or expr.post (v0.2)
    /// Used in contracts to reference pre/post-state values
    StateRef {
        expr: Box<Spanned<Expr>>,
        state: StateKind,
    },

    // v0.13.2: Error propagation

    /// Try block: try { expr }
    /// Captures errors from the block and converts them to a Result
    Try {
        body: Box<Spanned<Expr>>,
    },

    /// Question mark operator: expr?
    /// Propagates error if expr is Err, unwraps Ok value otherwise
    Question {
        expr: Box<Spanned<Expr>>,
    },

    // v0.20.0: Closures

    /// Closure expression: |params| body
    /// Captures variables from the enclosing scope by value (move semantics)
    Closure {
        /// Closure parameters: name and optional type annotation
        params: Vec<ClosureParam>,
        /// Optional explicit return type
        ret_ty: Option<Box<Spanned<Type>>>,
        /// Closure body expression
        body: Box<Spanned<Expr>>,
    },

    // v0.31: Incremental development

    /// Todo expression: todo "message"
    /// Placeholder for unimplemented code. Type-checks as any type.
    /// At runtime, panics with the given message.
    Todo {
        /// Optional message describing what needs to be implemented
        message: Option<String>,
    },
}

/// A single arm in a match expression
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchArm {
    pub pattern: Spanned<Pattern>,
    pub body: Spanned<Expr>,
}

/// Closure parameter (v0.20.0)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClosureParam {
    /// Parameter name
    pub name: Spanned<String>,
    /// Optional type annotation
    pub ty: Option<Spanned<Type>>,
}

/// Pattern for match expressions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Pattern {
    /// Wildcard pattern: _
    Wildcard,
    /// Variable binding: name
    Var(String),
    /// Literal pattern: 42, true, etc.
    Literal(LiteralPattern),
    /// Enum variant pattern: EnumName::Variant or EnumName::Variant(bindings)
    EnumVariant {
        enum_name: String,
        variant: String,
        bindings: Vec<Spanned<String>>,
    },
    /// Struct pattern: StructName { field1: pat1, field2: pat2 }
    Struct {
        name: String,
        fields: Vec<(Spanned<String>, Spanned<Pattern>)>,
    },
}

/// Literal patterns for match
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LiteralPattern {
    Int(i64),
    Float(f64),
    Bool(bool),
    String(String),
}

/// Binary operator
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BinOp {
    // Arithmetic
    Add,
    Sub,
    Mul,
    Div,
    Mod,

    // Comparison
    Eq,
    Ne,
    Lt,
    Gt,
    Le,
    Ge,

    // Logical
    And,
    Or,
}

impl std::fmt::Display for BinOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BinOp::Add => write!(f, "+"),
            BinOp::Sub => write!(f, "-"),
            BinOp::Mul => write!(f, "*"),
            BinOp::Div => write!(f, "/"),
            BinOp::Mod => write!(f, "%"),
            BinOp::Eq => write!(f, "=="),
            BinOp::Ne => write!(f, "!="),
            BinOp::Lt => write!(f, "<"),
            BinOp::Gt => write!(f, ">"),
            BinOp::Le => write!(f, "<="),
            BinOp::Ge => write!(f, ">="),
            BinOp::And => write!(f, "and"),
            BinOp::Or => write!(f, "or"),
        }
    }
}

/// Unary operator
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UnOp {
    /// Negation (-)
    Neg,
    /// Logical not
    Not,
}

impl std::fmt::Display for UnOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UnOp::Neg => write!(f, "-"),
            UnOp::Not => write!(f, "not"),
        }
    }
}

/// Range kind for different range operators (v0.2)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RangeKind {
    /// Exclusive/half-open range: start..<end or start..end (legacy)
    /// Represents [start, end)
    Exclusive,
    /// Inclusive/closed range: start..=end
    /// Represents [start, end]
    Inclusive,
}

/// State kind for contract state references (v0.2)
/// Used to reference values before or after function execution
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StateKind {
    /// Pre-state: value before function body executes
    Pre,
    /// Post-state: value after function body executes
    Post,
}

impl std::fmt::Display for StateKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StateKind::Pre => write!(f, ".pre"),
            StateKind::Post => write!(f, ".post"),
        }
    }
}

impl std::fmt::Display for RangeKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RangeKind::Exclusive => write!(f, "..<"),
            RangeKind::Inclusive => write!(f, "..="),
        }
    }
}
