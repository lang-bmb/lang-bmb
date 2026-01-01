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

    /// Let binding: let name = value; body
    Let {
        name: String,
        ty: Option<Spanned<Type>>,
        value: Box<Spanned<Expr>>,
        body: Box<Spanned<Expr>>,
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

    // v0.5: Struct and Enum expressions

    /// Struct initialization: StructName { field1: value1, field2: value2 }
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
}

/// A single arm in a match expression
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchArm {
    pub pattern: Spanned<Pattern>,
    pub body: Spanned<Expr>,
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
