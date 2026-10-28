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
    /// `struct` and `enum` declarations (ADR-0026).
    pub types: Vec<TypeDef>,
    /// `impl T { .. }` blocks (ADR-0026: one per type, same unit).
    pub impls: Vec<Impl>,
    /// `trait` declarations (ADR-0046 §1).
    pub traits: Vec<TraitDef>,
    /// `impl Trait for T { .. }` blocks (ADR-0046 §2).
    pub trait_impls: Vec<TraitImpl>,
    /// `use a::b::c;` items. Not part of the language: the checker rewrites
    /// each name they import to its full path and deletes them (E0214,
    /// backlog 2V-15).
    pub uses: Vec<UseItem>,
    pub span: Span,
}

/// A single-name `use` item.
#[derive(Debug, Clone)]
pub struct UseItem {
    pub path: Vec<Ident>,
    pub span: Span,
    /// The item with the line break and indentation before it: what the
    /// fix deletes, so no blank line is left.
    pub del: Span,
}

/// A type parameter `T`, `T: Copy` (ADR-0046 §1).
#[derive(Debug, Clone)]
pub struct TypeParam {
    pub name: Ident,
    /// Trait bounds, each a path (`Copy`, `geo::Shape`).
    pub bounds: Vec<Vec<Ident>>,
    pub span: Span,
}

/// Type arguments written after segment `seg` of a call path:
/// `f::<i64>(..)`, `Stack::<i64>::new(..)` (ADR-0046 §3).
#[derive(Debug, Clone)]
pub struct TArgs {
    pub seg: usize,
    pub args: Vec<Type>,
    /// From `::` to `>`.
    pub span: Span,
}

/// A data type declaration (ADR-0026 §1).
#[derive(Debug, Clone)]
pub struct TypeDef {
    pub name: Ident,
    pub generics: Vec<TypeParam>,
    pub attrs: Vec<Attr>,
    pub kind: TypeDefKind,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum TypeDefKind {
    /// Named fields, in declaration order.
    Struct(Vec<Field>),
    /// Unit variants `A` and tuple variants `B(T, U)`, in declaration order.
    Enum(Vec<Variant>),
}

#[derive(Debug, Clone)]
pub struct Field {
    pub name: Ident,
    pub ty: Type,
}

#[derive(Debug, Clone)]
pub struct Variant {
    pub name: Ident,
    pub payload: Vec<Type>,
    pub span: Span,
}

/// `impl T { fn .. }`: associated functions of `T` (paths `unit::T::f`).
#[derive(Debug, Clone)]
pub struct Impl {
    /// `impl<T> Stack<T> { .. }`: the impl's parameters, and the type's
    /// arguments as written.
    pub generics: Vec<TypeParam>,
    pub ty: Ident,
    pub ty_args: Vec<Type>,
    pub fns: Vec<FnDef>,
    pub span: Span,
}

/// `trait Shape { fn area(self: &Self) -> i64; }` (ADR-0046 §1). Its
/// methods name the implementing type `Self`.
#[derive(Debug, Clone)]
pub struct TraitDef {
    pub name: Ident,
    pub attrs: Vec<Attr>,
    pub fns: Vec<TraitFn>,
    pub span: Span,
}

/// A trait method: its signature, and whether a default body was written
/// (otherwise the body is empty and the signature ends with `;`).
#[derive(Debug, Clone)]
pub struct TraitFn {
    pub f: FnDef,
    pub has_body: bool,
}

/// `impl<T: Copy> geo::Shape for Wrap<T> { .. }` (ADR-0046 §2).
#[derive(Debug, Clone)]
pub struct TraitImpl {
    pub generics: Vec<TypeParam>,
    pub trait_path: Vec<Ident>,
    /// The implementing type as written.
    pub self_ty: Type,
    pub fns: Vec<FnDef>,
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
    pub generics: Vec<TypeParam>,
    pub attrs: Vec<Attr>,
    pub params: Vec<Param>,
    pub ret: Ret,
    pub requires: Vec<Expr>,
    pub ensures: Vec<Expr>,
    pub decreases: Option<Expr>,
    pub body: Block,
    pub span: Span,
}

/// The integer types (ADR-0048 §1): a width and a signedness. `isize` and
/// `usize` are 64 bits (the only target). `u8` is the byte (ADR-0044 §1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum IntTy {
    I8,
    I16,
    I32,
    I64,
    Isize,
    U8,
    U16,
    U32,
    U64,
    Usize,
}

impl IntTy {
    pub const ALL: [IntTy; 10] = [
        IntTy::I8,
        IntTy::I16,
        IntTy::I32,
        IntTy::I64,
        IntTy::Isize,
        IntTy::U8,
        IntTy::U16,
        IntTy::U32,
        IntTy::U64,
        IntTy::Usize,
    ];

    pub fn bits(self) -> u32 {
        match self {
            IntTy::I8 | IntTy::U8 => 8,
            IntTy::I16 | IntTy::U16 => 16,
            IntTy::I32 | IntTy::U32 => 32,
            _ => 64,
        }
    }

    pub fn signed(self) -> bool {
        matches!(
            self,
            IntTy::I8 | IntTy::I16 | IntTy::I32 | IntTy::I64 | IntTy::Isize
        )
    }

    pub fn min_value(self) -> i128 {
        if self.signed() {
            -(1i128 << (self.bits() - 1))
        } else {
            0
        }
    }

    pub fn max_value(self) -> i128 {
        if self.signed() {
            (1i128 << (self.bits() - 1)) - 1
        } else {
            (1i128 << self.bits()) - 1
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            IntTy::I8 => "i8",
            IntTy::I16 => "i16",
            IntTy::I32 => "i32",
            IntTy::I64 => "i64",
            IntTy::Isize => "isize",
            IntTy::U8 => "u8",
            IntTy::U16 => "u16",
            IntTy::U32 => "u32",
            IntTy::U64 => "u64",
            IntTy::Usize => "usize",
        }
    }

    pub fn from_name(s: &str) -> Option<IntTy> {
        IntTy::ALL.into_iter().find(|t| t.name() == s)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeKind {
    /// A fixed-width integer (ADR-0048 §1).
    Int(IntTy),
    Bool,
    Unit,
    Array(Box<Type>, u64),
    Slice(Box<Type>),
    Ref {
        mutable: bool,
        inner: Box<Type>,
    },
    /// A named type: a struct or enum (`Point`, `geo::Point`) or a built-in
    /// `Option<T>` / `Result<T, E>` with its arguments (ADR-0026).
    Named {
        path: Vec<String>,
        args: Vec<Type>,
    },
    /// The built-in heap vector `Vec<T>` (ADR-0031).
    Vec(Box<Type>),
    /// The built-in owned heap cell `Box<T>` (ADR-0047 §1).
    Box(Box<Type>),
    /// A type parameter in scope (ADR-0046), resolved by the checker from a
    /// one-segment `Named`.
    Param(String),
    /// The type of an expression that does not finish (`return`, ADR-0049
    /// §1): it fits any expected type.
    Never,
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
        /// `'outer: while ..` (ADR-0049 §1).
        label: Option<Ident>,
        cond: Expr,
        invariants: Vec<Expr>,
        decreases: Option<Expr>,
        body: Block,
        span: Span,
    },
    /// `'l: for i in lo..hi invariant .., { .. }` (or `lo..=hi`, ADR-0049
    /// §2): `i` is immutable, `lo <= i` and `i < hi` hold in the body, and
    /// the measure is automatic.
    For {
        label: Option<Ident>,
        var: Ident,
        lo: Expr,
        hi: Expr,
        inclusive: bool,
        invariants: Vec<Expr>,
        body: Block,
        span: Span,
    },
    Assert {
        cond: Expr,
        span: Span,
    },
    /// `let Point { x, y } = p;` (Rust destructuring, E0230): kept so the
    /// checker can write the field reads with their types.
    LetFields {
        path: Vec<Ident>,
        fields: Vec<Ident>,
        init: Expr,
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
    /// `b'a'`: a `u8` (ADR-0044 §2).
    ByteLit(u8),
    /// `b"..."`: a `&[u8]`, only as a call argument (ADR-0044 §2).
    ByteStr(Vec<u8>),
    Bool(bool),
    Unit,
    /// A local name (single segment).
    Var(Ident),
    /// Direct call by path: `f(..)` or `a::b::f(..)`.
    Call {
        path: Vec<Ident>,
        /// Type arguments written in the path (ADR-0046 §3).
        targs: Vec<TArgs>,
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
    /// `Point { x: e, y: e }` (ADR-0026 §3).
    StructLit {
        path: Vec<Ident>,
        fields: Vec<(Ident, Expr)>,
        /// `..base` (Rust update syntax, E0223): kept so the checker can
        /// write out the missing fields.
        rest: Option<Box<Expr>>,
    },
    /// A unit variant written as a path: `Shape::Dot` (`None` is a `Var`).
    Path(Vec<Ident>),
    /// `*p`: the value behind a reference (ADR-0026: `small(*p)`, `*p = ..`).
    Deref(Box<Expr>),
    /// `p.x`
    Field {
        base: Box<Expr>,
        name: Ident,
    },
    /// `match e { P => e, .. }`
    Match {
        scrut: Box<Expr>,
        arms: Vec<Arm>,
    },
    /// `return e` / `return` (ADR-0049 §1): of type `!`.
    Return(Option<Box<Expr>>),
    /// `e?` (ADR-0049 §1): the payload of `Some`/`Ok`; on `None`/`Err(x)`
    /// the function returns `None`/`Err(x)` (the same error type).
    Try(Box<Expr>),
    /// `'l: loop invariant .., decreases .., { .. }` (ADR-0049 §1): its value
    /// is that of the `break e` that leaves it.
    Loop {
        label: Option<Ident>,
        invariants: Vec<Expr>,
        decreases: Option<Box<Expr>>,
        body: Block,
    },
    /// `break`, `break 'l`, `break e` (of a `loop`): of type `!`.
    Break {
        label: Option<Ident>,
        value: Option<Box<Expr>>,
    },
    /// `continue`, `continue 'l`: of type `!`.
    Continue {
        label: Option<Ident>,
    },
    Error,
}

/// One `match` arm (ADR-0026: one per variant, in declaration order).
#[derive(Debug, Clone)]
pub struct Arm {
    pub pat: Pat,
    pub body: Expr,
    pub span: Span,
}

/// A variant pattern: `Shape::Rect(a, _)`, `Some(x)`, `None`. Each payload
/// position binds a new name or is `_` (`None`).
#[derive(Debug, Clone)]
pub struct Pat {
    pub path: Vec<Ident>,
    pub binds: Vec<Option<Ident>>,
    /// Written with parentheses (`V()` and `V` differ only in diagnostics).
    pub parens: bool,
    pub origin: PatOrigin,
    pub span: Span,
}

/// Where a pattern came from. Only `Written` patterns are what the source
/// says; the others stand in for arms the parser rebuilt.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PatOrigin {
    Written,
    /// The `else` of `if let P = e { .. } else { .. }` (E0225).
    IfLetElse,
    /// A pattern form BMB has no place for (E0232), already reported: it
    /// covers the remaining variants so no second error follows.
    Rejected,
}

impl TypeKind {
    /// The integer type, if this is one.
    pub fn int(&self) -> Option<IntTy> {
        match self {
            TypeKind::Int(t) => Some(*t),
            _ => None,
        }
    }

    /// A signed integer type.
    pub fn is_signed(&self) -> bool {
        self.int().is_some_and(IntTy::signed)
    }
}
