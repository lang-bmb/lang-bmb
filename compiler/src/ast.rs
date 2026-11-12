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
    /// `key = value` (`#[rationale(kind = nonlinear, hash = "..")]`,
    /// ADR-0029 §1): the value is an identifier or a string.
    Pair(Ident, Box<AttrArg>),
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
    /// `const NAME: T = e;` items (ADR-0052).
    pub consts: Vec<ConstDef>,
    /// The repeat counts written as a `const`'s name (`[0; N]`, ADR-0052
    /// §4), by the span of the count: the checker reads the value, the
    /// formatter the name. The count holds 0 until then.
    pub lens: Vec<(Span, Vec<Ident>)>,
    pub span: Span,
}

/// A closure expression (ADR-0053 §1, §2): its parameters and result are
/// typed, its contract is optional.
#[derive(Debug, Clone)]
pub struct Closure {
    pub is_move: bool,
    pub params: Vec<Param>,
    pub ret: Ret,
    pub requires: Vec<Expr>,
    pub ensures: Vec<Expr>,
    pub body: Block,
    pub span: Span,
}

/// `const NAME: T = e;` (ADR-0052): a value the checker evaluates.
#[derive(Debug, Clone)]
pub struct ConstDef {
    /// `#[summary("..")]` (ADR-0117).
    pub attrs: Vec<Attr>,
    pub name: Ident,
    pub ty: Type,
    pub value: Expr,
    pub span: Span,
}

/// A single-name `use` item.
#[derive(Debug, Clone)]
pub struct UseItem {
    pub path: Vec<Ident>,
    /// From `use` to `;`: what E0214 points at and its fix deletes (the
    /// fix layout takes the line the deletion leaves blank).
    pub span: Span,
}

/// A type parameter `T`, `T: Copy` (ADR-0046 §1).
#[derive(Debug, Clone)]
pub struct TypeParam {
    pub name: Ident,
    /// Trait bounds, each a path (`Copy`, `geo::Shape`).
    pub bounds: Vec<Vec<Ident>>,
    /// `F: Fn(A, B) -> R` (ADR-0053 §1): a function value's bound.
    pub fn_bound: Option<FnBound>,
    pub span: Span,
}

/// Which of the built-in function traits a bound names (ADR-0053 §2).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FnTrait {
    Fn,
    FnMut,
    FnOnce,
}

impl FnTrait {
    pub fn name(self) -> &'static str {
        match self {
            FnTrait::Fn => "Fn",
            FnTrait::FnMut => "FnMut",
            FnTrait::FnOnce => "FnOnce",
        }
    }
}

/// `Fn(A, B) -> R`: the parameter types and the result (`()` when no
/// `->` is written).
#[derive(Debug, Clone)]
pub struct FnBound {
    pub kind: FnTrait,
    pub params: Vec<Type>,
    pub ret: Type,
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
    /// `proof fn` (ADR-0062 §1): a ghost function whose body is a proof of
    /// its `ensures`; called only from ghost code, erased by code
    /// generation.
    Proof,
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
    /// IEEE 754 binary64 (ADR-0061 §1).
    F64,
    Bool,
    Unit,
    /// A Unicode scalar value (ADR-0050 §1).
    Char,
    /// The owned UTF-8 string `String` (ADR-0050 §1): a move type.
    String,
    /// `str`, only behind a reference: `&str` (ADR-0050 §1).
    Str,
    Array(Box<Type>, u64),
    /// `[T; N]` with a `const`'s name as its length (ADR-0052 §4), as
    /// written: the checker reads it as the `Array` of the evaluated length.
    ArrayN(Box<Type>, Vec<String>),
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
    /// `assert(cond);`, or `assert(cond) by (bit_vector);` (ADR-0048 §4):
    /// `by` proves the assertion in a query of its own, in an exact theory,
    /// before the enclosing proof assumes it.
    Assert {
        cond: Expr,
        by: Option<Prover>,
        span: Span,
    },
    /// `assume(p(..));` (ADR-0029 §1): an escape. The proof after it may
    /// assume the named predicate, as a trusted fact (§2: the definition is
    /// then `assumed` if it needs it). `rationale` is the
    /// `#[rationale(..)]` written before it (E1001 without).
    Assume {
        cond: Expr,
        rationale: Option<Attr>,
        span: Span,
    },
    /// `proof { .. }` (ADR-0062 §1): ghost statements in executable code.
    /// It changes no executable state and produces no value; code
    /// generation erases it.
    Proof {
        body: Block,
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

/// The prover of `assert(e) by (..)` (ADR-0048 §4). `bit_vector` proves
/// `e` with integers as bit-vectors of their widths and bit operations
/// interpreted exactly; ADR-0061 §5 reuses it for floating point.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Prover {
    BitVector,
}

impl Prover {
    pub fn name(self) -> &'static str {
        match self {
            Prover::BitVector => "bit_vector",
        }
    }

    pub fn from_name(s: &str) -> Option<Prover> {
        match s {
            "bit_vector" => Some(Prover::BitVector),
            _ => None,
        }
    }
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
    /// `&`, `|`, `^`: bitwise on integers, logical (both sides evaluated)
    /// on `bool`, as in Rust (ADR-0048 §2).
    BitAnd,
    BitOr,
    BitXor,
    /// `<<`, `>>`: the amount is any integer type and must be below the
    /// width (E0803); `>>` is arithmetic on signed types (ADR-0048 §2).
    Shl,
    Shr,
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
            BinOp::BitAnd => "&",
            BinOp::BitOr => "|",
            BinOp::BitXor => "^",
            BinOp::Shl => "<<",
            BinOp::Shr => ">>",
        }
    }
    pub fn is_arith(self) -> bool {
        matches!(
            self,
            BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div | BinOp::Rem
        )
    }
    /// `&`, `|`, `^`: both operands and the result have one type.
    pub fn is_bit(self) -> bool {
        matches!(self, BinOp::BitAnd | BinOp::BitOr | BinOp::BitXor)
    }
    pub fn is_shift(self) -> bool {
        matches!(self, BinOp::Shl | BinOp::Shr)
    }
    pub fn is_cmp(self) -> bool {
        matches!(
            self,
            BinOp::Eq | BinOp::Ne | BinOp::Lt | BinOp::Le | BinOp::Gt | BinOp::Ge
        )
    }
}

/// How a method call passes its receiver (ADR-0051): as written, or
/// borrowed for a `self: &T` / `self: &mut T` receiver, or a `String`'s
/// text for a standard library text function taking `&str` (`s.as_str()`,
/// ADR-0092 §2).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Adjust {
    Value,
    Ref,
    RefMut,
    AsStr,
}

#[derive(Debug, Clone)]
pub struct Expr {
    pub kind: ExprKind,
    pub span: Span,
}

/// The base an integer literal is written in (ADR-0048 §3). The normal form
/// keeps it: `0xff` stays `0xff`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Radix {
    Dec,
    Hex,
    Oct,
    Bin,
}

impl Radix {
    /// The normal form of `v` written in this base: the prefix, lowercase
    /// digits, no `_` separators and no leading zeros (ADR-0019 §1).
    pub fn text(self, v: u128) -> String {
        match self {
            Radix::Dec => v.to_string(),
            Radix::Hex => format!("0x{v:x}"),
            Radix::Oct => format!("0o{v:o}"),
            Radix::Bin => format!("0b{v:b}"),
        }
    }
}

#[derive(Debug, Clone)]
pub enum ExprKind {
    /// An integer literal and the base it is written in.
    Int(u128, Radix),
    /// A floating-point literal (ADR-0061 §2): the bits of its `f64`
    /// value. It is an `f64` wherever it appears.
    Float(u64),
    /// `b'a'`: a `u8` (ADR-0044 §2).
    ByteLit(u8),
    /// `b"..."`: a `&[u8]`, only as a call argument (ADR-0044 §2).
    ByteStr(Vec<u8>),
    /// `"..."`: a `&str`, only as a call argument (ADR-0050 §1).
    StrLit(String),
    /// `'a'`: a `char`, the scalar value (ADR-0050 §1).
    CharLit(u32),
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
        /// Written as the method call `x.f(..)` (ADR-0051): how the
        /// receiver, the first argument, was passed. Only
        /// `check::check_resolved` makes such calls; printing writes them
        /// as they were written.
        method: Option<Adjust>,
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
    /// `forall|x: T| e` or `exists|x: T| e` (specifications only, ADR-0079).
    Quant {
        q: Quantifier,
        binders: Vec<Param>,
        body: Box<Expr>,
    },
    Trigger(Box<Expr>),
    /// `move |x: T| -> (r: R) requires P, ensures Q, { body }` (ADR-0053
    /// §1): lowered before checking (`closures::lower`), so only the parser,
    /// the formatter and the JSON AST meet it.
    Closure(Box<Closure>),
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
    /// `lo..hi`: only as the first argument of `copy_within` (ADR-0083);
    /// anywhere else the checker refuses it (E0435).
    Range {
        lo: Box<Expr>,
        hi: Box<Expr>,
        /// Written `lo..=hi`: the parser reported it (E0438), and the
        /// checker reads the range as it reads `lo..hi`.
        inclusive: bool,
    },
    Error,
}

/// The quantifier of an `ExprKind::Quant` (ADR-0079).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Quantifier {
    Forall,
    Exists,
}

impl Quantifier {
    /// Its keyword, as written.
    pub fn keyword(self) -> &'static str {
        match self {
            Quantifier::Forall => "forall",
            Quantifier::Exists => "exists",
        }
    }
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

/// Visits every expression of the program, executable and specification
/// alike, children before their node (`check::check_resolved` rewrites
/// method calls whose receivers it has already rewritten).
pub fn exprs_mut(p: &mut Program, f: &mut dyn FnMut(&mut Expr)) {
    fn module(m: &mut Mod, f: &mut dyn FnMut(&mut Expr)) {
        for s in &mut m.mods {
            module(s, f);
        }
        for d in fns_mut(m) {
            fn_exprs_mut(d, f);
        }
    }
    for m in &mut p.mods {
        module(m, f);
    }
}

/// Every expression of every `const`'s initializer (ADR-0052).
pub fn const_exprs_mut(p: &mut Program, f: &mut dyn FnMut(&mut Expr)) {
    fn module(m: &mut Mod, f: &mut dyn FnMut(&mut Expr)) {
        for s in &mut m.mods {
            module(s, f);
        }
        for c in &mut m.consts {
            expr_mut(&mut c.value, f);
        }
    }
    for m in &mut p.mods {
        module(m, f);
    }
}

/// The functions of a unit (not of its sub-units): free, inherent `impl`,
/// trait declaration (with or without a body) and trait `impl` ones.
pub fn fns_mut(m: &mut Mod) -> impl Iterator<Item = &mut FnDef> {
    m.fns
        .iter_mut()
        .chain(m.impls.iter_mut().flat_map(|i| &mut i.fns))
        .chain(
            m.traits
                .iter_mut()
                .flat_map(|t| t.fns.iter_mut().map(|t| &mut t.f)),
        )
        .chain(m.trait_impls.iter_mut().flat_map(|i| &mut i.fns))
}

/// Every type written in function `d`: its parameters, its result, every
/// `let` annotation at any depth, and the type arguments of its calls.
pub fn fn_types(d: &FnDef) -> Vec<Type> {
    fn stmts(b: &Block, out: &mut Vec<Type>) {
        for s in &b.stmts {
            match s {
                Stmt::Let { ty, .. } => out.push(ty.clone()),
                Stmt::While { body, .. } | Stmt::For { body, .. } | Stmt::Proof { body, .. } => {
                    stmts(body, out)
                }
                _ => {}
            }
        }
    }
    let mut out: Vec<Type> = d.params.iter().map(|p| p.ty.clone()).collect();
    out.push(d.ret.ty.clone());
    stmts(&d.body, &mut out);
    let mut inner: Vec<Type> = Vec::new();
    let mut c = d.clone();
    fn_exprs_mut(&mut c, &mut |e| match &e.kind {
        ExprKind::Block(b) | ExprKind::Loop { body: b, .. } => stmts(b, &mut inner),
        ExprKind::If { then, .. } => stmts(then, &mut inner),
        ExprKind::Call { targs, .. } => {
            for t in targs {
                inner.extend(t.args.iter().cloned());
            }
        }
        _ => {}
    });
    out.extend(inner);
    out
}

/// As `exprs_mut`, for one function: its contract, measure and body.
pub fn fn_exprs_mut(d: &mut FnDef, f: &mut dyn FnMut(&mut Expr)) {
    d.requires.iter_mut().for_each(|e| expr_mut(e, f));
    d.ensures.iter_mut().for_each(|e| expr_mut(e, f));
    if let Some(e) = &mut d.decreases {
        expr_mut(e, f);
    }
    block_mut(&mut d.body, f);
}

pub fn block_mut(b: &mut Block, f: &mut dyn FnMut(&mut Expr)) {
    for s in &mut b.stmts {
        match s {
            Stmt::Let { init, .. } | Stmt::LetFields { init, .. } => expr_mut(init, f),
            Stmt::Assign { target, value, .. } => {
                expr_mut(target, f);
                expr_mut(value, f);
            }
            Stmt::While {
                cond,
                invariants,
                decreases,
                body,
                ..
            } => {
                expr_mut(cond, f);
                invariants.iter_mut().for_each(|e| expr_mut(e, f));
                if let Some(e) = decreases {
                    expr_mut(e, f);
                }
                block_mut(body, f);
            }
            Stmt::For {
                lo,
                hi,
                invariants,
                body,
                ..
            } => {
                expr_mut(lo, f);
                expr_mut(hi, f);
                invariants.iter_mut().for_each(|e| expr_mut(e, f));
                block_mut(body, f);
            }
            Stmt::Assert { cond, .. } | Stmt::Assume { cond, .. } => expr_mut(cond, f),
            Stmt::Proof { body, .. } => block_mut(body, f),
            Stmt::Expr(e) => expr_mut(e, f),
        }
    }
    if let Some(t) = &mut b.tail {
        expr_mut(t, f);
    }
}

/// Visits `e` and every expression inside it.
pub fn expr_walk(e: &mut Expr, f: &mut dyn FnMut(&mut Expr)) {
    expr_mut(e, f);
}

/// Whether `e` or an expression inside it is a quantifier (`forall`,
/// `exists`).
pub fn has_quant(e: &Expr) -> bool {
    let mut e = e.clone();
    let mut found = false;
    expr_mut(&mut e, &mut |x| {
        found |= matches!(x.kind, ExprKind::Quant { .. })
    });
    found
}

pub fn expr_mut(e: &mut Expr, f: &mut dyn FnMut(&mut Expr)) {
    match &mut e.kind {
        ExprKind::Closure(c) => {
            c.requires.iter_mut().for_each(|x| expr_mut(x, f));
            c.ensures.iter_mut().for_each(|x| expr_mut(x, f));
            block_mut(&mut c.body, f);
        }
        ExprKind::Int(..)
        | ExprKind::Float(_)
        | ExprKind::ByteLit(_)
        | ExprKind::ByteStr(_)
        | ExprKind::StrLit(_)
        | ExprKind::CharLit(_)
        | ExprKind::Bool(_)
        | ExprKind::Unit
        | ExprKind::Var(_)
        | ExprKind::Path(_)
        | ExprKind::Continue { .. }
        | ExprKind::Error => {}
        ExprKind::Call { args, .. } | ExprKind::Array(args) => {
            args.iter_mut().for_each(|a| expr_mut(a, f))
        }
        ExprKind::Method { recv, args, .. } => {
            expr_mut(recv, f);
            args.iter_mut().for_each(|a| expr_mut(a, f));
        }
        ExprKind::Index { base: a, index: b }
        | ExprKind::Binary(_, a, b)
        | ExprKind::Range { lo: a, hi: b, .. } => {
            expr_mut(a, f);
            expr_mut(b, f);
        }
        ExprKind::Unary(_, x)
        | ExprKind::Ref { expr: x, .. }
        | ExprKind::Repeat { value: x, .. }
        | ExprKind::Old(x)
        | ExprKind::Trigger(x)
        | ExprKind::Deref(x)
        | ExprKind::Field { base: x, .. }
        | ExprKind::Try(x) => expr_mut(x, f),
        ExprKind::Quant { body, .. } => expr_mut(body, f),
        ExprKind::If { cond, then, els } => {
            expr_mut(cond, f);
            block_mut(then, f);
            if let Some(x) = els {
                expr_mut(x, f);
            }
        }
        ExprKind::Block(b) => block_mut(b, f),
        ExprKind::StructLit { fields, rest, .. } => {
            fields.iter_mut().for_each(|(_, x)| expr_mut(x, f));
            if let Some(x) = rest {
                expr_mut(x, f);
            }
        }
        ExprKind::Match { scrut, arms } => {
            expr_mut(scrut, f);
            arms.iter_mut().for_each(|a| expr_mut(&mut a.body, f));
        }
        ExprKind::Return(x) | ExprKind::Break { value: x, .. } => {
            if let Some(x) = x {
                expr_mut(x, f);
            }
        }
        ExprKind::Loop {
            invariants,
            decreases,
            body,
            ..
        } => {
            invariants.iter_mut().for_each(|e| expr_mut(e, f));
            if let Some(e) = decreases {
                expr_mut(e, f);
            }
            block_mut(body, f);
        }
    }
    f(e);
}

/// Visits every type written in `p` (parameters, results, `let`s, fields,
/// payloads, `impl` heads, type arguments, quantifier binders, `const`s),
/// outermost first; `f` sees a nested type only through the outer one.
pub fn types_mut(p: &mut Program, f: &mut dyn FnMut(&mut Type)) {
    fn stmts(b: &mut Block, f: &mut dyn FnMut(&mut Type)) {
        for s in &mut b.stmts {
            match s {
                Stmt::Let { ty, .. } => f(ty),
                Stmt::While { body, .. } | Stmt::For { body, .. } | Stmt::Proof { body, .. } => {
                    stmts(body, f)
                }
                _ => {}
            }
        }
    }
    fn def(d: &mut FnDef, f: &mut dyn FnMut(&mut Type)) {
        for p in &mut d.params {
            f(&mut p.ty);
        }
        f(&mut d.ret.ty);
        stmts(&mut d.body, f);
        fn_exprs_mut(d, &mut |e| match &mut e.kind {
            ExprKind::Block(b) | ExprKind::Loop { body: b, .. } => stmts(b, f),
            ExprKind::If { then, .. } => stmts(then, f),
            ExprKind::Call { targs, .. } => {
                for t in targs {
                    t.args.iter_mut().for_each(&mut *f);
                }
            }
            ExprKind::Quant { binders, .. } => {
                for b in binders {
                    f(&mut b.ty);
                }
            }
            _ => {}
        });
    }
    fn module(m: &mut Mod, f: &mut dyn FnMut(&mut Type)) {
        for c in &mut m.consts {
            f(&mut c.ty);
        }
        for t in &mut m.types {
            match &mut t.kind {
                TypeDefKind::Struct(fs) => fs.iter_mut().for_each(|x| f(&mut x.ty)),
                TypeDefKind::Enum(vs) => {
                    for v in vs {
                        v.payload.iter_mut().for_each(&mut *f);
                    }
                }
            }
        }
        for i in &mut m.impls {
            i.ty_args.iter_mut().for_each(&mut *f);
        }
        for i in &mut m.trait_impls {
            f(&mut i.self_ty);
        }
        for d in fns_mut(m) {
            def(d, f);
        }
    }
    fn all(m: &mut Mod, f: &mut dyn FnMut(&mut Type)) {
        module(m, f);
        for s in &mut m.mods {
            all(s, f);
        }
    }
    for m in &mut p.mods {
        all(m, f);
    }
}

/// As `types_mut`, for each unit with its path (`a::b`), not into its
/// sub-units' items twice.
pub fn unit_types_mut(p: &mut Program, f: &mut dyn FnMut(&str, &mut Type)) {
    fn walk(m: &mut Mod, prefix: &str, f: &mut dyn FnMut(&str, &mut Type)) {
        let unit = format!("{prefix}{}", m.name.name);
        let subs = std::mem::take(&mut m.mods);
        let mut one = Program {
            mods: vec![std::mem::replace(
                m,
                Mod {
                    name: m.name.clone(),
                    attrs: Vec::new(),
                    fns: Vec::new(),
                    mods: Vec::new(),
                    types: Vec::new(),
                    impls: Vec::new(),
                    traits: Vec::new(),
                    trait_impls: Vec::new(),
                    uses: Vec::new(),
                    consts: Vec::new(),
                    lens: Vec::new(),
                    span: m.span,
                },
            )],
        };
        types_mut(&mut one, &mut |t| f(&unit, t));
        *m = one.mods.pop().expect("one unit");
        m.mods = subs;
        for s in &mut m.mods {
            walk(s, &format!("{unit}::"), f);
        }
    }
    for m in &mut p.mods {
        walk(m, "", f);
    }
}
