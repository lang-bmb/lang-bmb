//! Syntax tree for the milestone-1 subset (ADR-0016).

use crate::source::Span;

#[derive(Debug, Clone)]
pub struct Ident {
    pub name: String,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum AttrArg {
    Ident(Ident),
    Str(String, Span),
}

#[derive(Debug, Clone)]
pub struct Attr {
    pub name: Ident,
    pub args: Vec<AttrArg>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct Program {
    pub mods: Vec<Mod>,
}

#[derive(Debug, Clone)]
pub struct Mod {
    pub name: Ident,
    pub attrs: Vec<Attr>,
    pub fns: Vec<FnDef>,
    pub mods: Vec<Mod>,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FnKind {
    Exec,
    Spec,
}

#[derive(Debug, Clone)]
pub struct Param {
    pub name: Ident,
    pub ty: Type,
}

#[derive(Debug, Clone)]
pub struct Ret {
    pub name: Option<Ident>,
    pub ty: Type,
}

#[derive(Debug, Clone)]
pub struct FnDef {
    pub kind: FnKind,
    pub name: Ident,
    pub attrs: Vec<Attr>,
    pub params: Vec<Param>,
    pub ret: Ret,
    pub requires: Vec<Expr>,
    pub ensures: Vec<Expr>,
    pub decreases: Option<Expr>,
    pub body: Block,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeKind {
    I64,
    U64,
    Usize,
    Bool,
    Unit,
    Array(Box<Type>, u64),
    Slice(Box<Type>),
    Ref { mutable: bool, inner: Box<Type> },
    Error,
}

#[derive(Debug, Clone)]
pub struct Type {
    pub kind: TypeKind,
    pub span: Span,
}

impl PartialEq for Type {
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind
    }
}
impl Eq for Type {}

#[derive(Debug, Clone)]
pub struct Block {
    pub stmts: Vec<Stmt>,
    pub tail: Option<Box<Expr>>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum Stmt {
    Let {
        mutable: bool,
        name: Ident,
        ty: Type,
        /// No `: T` was written (E0201); the checker infers it for the fix.
        ty_missing: bool,
        init: Expr,
        span: Span,
    },
    Assign {
        target: Expr,
        value: Expr,
        span: Span,
    },
    While {
        cond: Expr,
        invariants: Vec<Expr>,
        decreases: Option<Expr>,
        body: Block,
        span: Span,
    },
    Assert {
        cond: Expr,
        span: Span,
    },
    Expr(Expr),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnOp {
    Neg,
    Not,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    And,
    Or,
    Implies,
}

impl BinOp {
    pub fn symbol(self) -> &'static str {
        match self {
            BinOp::Add => "+",
            BinOp::Sub => "-",
            BinOp::Mul => "*",
            BinOp::Div => "/",
            BinOp::Rem => "%",
            BinOp::Eq => "==",
            BinOp::Ne => "!=",
            BinOp::Lt => "<",
            BinOp::Le => "<=",
            BinOp::Gt => ">",
            BinOp::Ge => ">=",
            BinOp::And => "&&",
            BinOp::Or => "||",
            BinOp::Implies => "==>",
        }
    }
    pub fn is_arith(self) -> bool {
        matches!(
            self,
            BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div | BinOp::Rem
        )
    }
    pub fn is_cmp(self) -> bool {
        matches!(
            self,
            BinOp::Eq | BinOp::Ne | BinOp::Lt | BinOp::Le | BinOp::Gt | BinOp::Ge
        )
    }
}

#[derive(Debug, Clone)]
pub struct Expr {
    pub kind: ExprKind,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum ExprKind {
    Int(u128),
    Bool(bool),
    Unit,
    /// A local name (single segment).
    Var(Ident),
    /// Direct call by path: `f(..)` or `a::b::f(..)`.
    Call {
        path: Vec<Ident>,
        args: Vec<Expr>,
    },
    /// Built-in method: `a.len()`, `x.wrapping_add(y)`.
    Method {
        recv: Box<Expr>,
        name: Ident,
        args: Vec<Expr>,
    },
    Index {
        base: Box<Expr>,
        index: Box<Expr>,
    },
    Unary(UnOp, Box<Expr>),
    Binary(BinOp, Box<Expr>, Box<Expr>),
    Ref {
        mutable: bool,
        expr: Box<Expr>,
    },
    If {
        cond: Box<Expr>,
        then: Block,
        els: Option<Box<Expr>>,
    },
    Block(Block),
    Array(Vec<Expr>),
    Repeat {
        value: Box<Expr>,
        count: u128,
        count_span: Span,
    },
    Old(Box<Expr>),
    Forall {
        binders: Vec<Param>,
        body: Box<Expr>,
    },
    Trigger(Box<Expr>),
    Error,
}
