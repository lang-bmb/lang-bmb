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
    /// Character literal (v0.64)
    CharLit(char),
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
    /// v0.37: Optional invariant for verification
    /// Syntax: while cond invariant inv { body }
    While {
        cond: Box<Spanned<Expr>>,
        /// v0.37: Optional loop invariant for SMT verification
        /// The invariant must hold before the loop and be preserved by each iteration
        invariant: Option<Box<Spanned<Expr>>>,
        body: Box<Spanned<Expr>>,
    },

    /// For loop: for var in iter { body } (v0.5 Phase 3)
    For {
        var: String,
        iter: Box<Spanned<Expr>>,
        body: Box<Spanned<Expr>>,
    },

    // v0.36: Additional control flow

    /// Infinite loop: loop { body }
    /// Exit with break, can return a value with `break value`
    Loop {
        body: Box<Spanned<Expr>>,
    },

    /// Break from loop: break or break value
    /// Returns unit or the specified value from the enclosing loop
    Break {
        value: Option<Box<Spanned<Expr>>>,
    },

    /// Continue to next iteration: continue
    Continue,

    /// Early return: return or return value
    Return {
        value: Option<Box<Spanned<Expr>>>,
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

    /// v0.43: Tuple field access: expr.0, expr.1, etc.
    /// Accesses tuple element by index (compile-time checked)
    TupleField {
        expr: Box<Spanned<Expr>>,
        index: usize,
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

    /// v0.42: Tuple expression: (expr1, expr2, ...)
    Tuple(Vec<Spanned<Expr>>),

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

    // v0.37: Quantifiers for verification

    /// Universal quantifier: forall x: T, condition
    /// Returns bool. True if condition holds for all x of type T.
    /// Used primarily in contract verification (SMT-based).
    Forall {
        /// Bound variable name
        var: Spanned<String>,
        /// Type of the bound variable
        ty: Spanned<Type>,
        /// Condition that must hold for all values
        body: Box<Spanned<Expr>>,
    },

    /// Existential quantifier: exists x: T, condition
    /// Returns bool. True if condition holds for some x of type T.
    /// Used primarily in contract verification (SMT-based).
    Exists {
        /// Bound variable name
        var: Spanned<String>,
        /// Type of the bound variable
        ty: Spanned<Type>,
        /// Condition that must hold for some value
        body: Box<Spanned<Expr>>,
    },

    // v0.39: Type casting

    /// Type cast expression: expr as Type
    /// Explicit conversion between numeric types.
    /// Examples: x as i64, y as u32, z as f64
    Cast {
        /// Expression to cast
        expr: Box<Spanned<Expr>>,
        /// Target type
        ty: Spanned<Type>,
    },
}

/// A single arm in a match expression
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchArm {
    pub pattern: Spanned<Pattern>,
    /// v0.40: Optional pattern guard (if condition)
    pub guard: Option<Spanned<Expr>>,
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
        /// v0.41: Changed from EnumBinding to Pattern to support nested patterns
        bindings: Vec<Spanned<Pattern>>,
    },
    /// Struct pattern: StructName { field1: pat1, field2: pat2 }
    Struct {
        name: String,
        fields: Vec<(Spanned<String>, Spanned<Pattern>)>,
    },
    /// v0.39: Range pattern: 1..10 or 1..=10
    Range {
        start: LiteralPattern,
        end: LiteralPattern,
        inclusive: bool,
    },
    /// v0.40: Or-pattern: A | B
    Or(Vec<Spanned<Pattern>>),
    /// v0.41: Binding pattern: name @ pattern
    /// Binds the matched value to `name` while also matching `pattern`
    Binding {
        name: String,
        pattern: Box<Spanned<Pattern>>,
    },
    /// v0.42: Tuple pattern: (pat1, pat2, ...)
    /// Matches tuple values and destructures into component patterns
    Tuple(Vec<Spanned<Pattern>>),
    /// v0.44: Array pattern: [pat1, pat2, ...]
    /// Matches fixed-size arrays and destructures into component patterns
    /// Array size is checked at compile-time for P0 correctness
    Array(Vec<Spanned<Pattern>>),
    /// v0.45: Array pattern with rest: [first, ..], [.., last], [first, .., last]
    /// Matches fixed-size arrays with variable middle elements (non-capturing)
    /// The ".." skips zero or more elements without binding them
    /// P0 Performance: Zero overhead - all indices computed at compile-time
    ArrayRest {
        /// Patterns to match at the beginning of the array
        prefix: Vec<Spanned<Pattern>>,
        /// Patterns to match at the end of the array
        suffix: Vec<Spanned<Pattern>>,
    },
}

// v0.41: EnumBinding removed - use Pattern directly for nested pattern support

/// Literal patterns for match
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LiteralPattern {
    Int(i64),
    Float(f64),
    Bool(bool),
    String(String),
}

/// v0.45: Helper for parsing array patterns with optional rest marker
/// Used internally by grammar to avoid LR conflicts
#[derive(Debug, Clone)]
pub enum ArrayPatternPart {
    /// A pattern element
    Pattern(Spanned<Pattern>),
    /// The rest marker (..)
    Rest,
}

impl ArrayPatternPart {
    /// v0.45: Convert a list of array pattern parts to a Pattern
    /// Returns Pattern::Array if no rest marker, Pattern::ArrayRest if rest marker present
    /// Panics if multiple rest markers are present (grammar should prevent this)
    pub fn into_pattern(parts: Vec<ArrayPatternPart>) -> Pattern {
        // Find the rest marker if any
        let rest_index = parts.iter().position(|p| matches!(p, ArrayPatternPart::Rest));

        if let Some(idx) = rest_index {
            // Check for multiple rest markers (should be caught at grammar level ideally)
            let second_rest = parts[idx + 1..].iter().any(|p| matches!(p, ArrayPatternPart::Rest));
            if second_rest {
                panic!("Multiple rest markers in array pattern");
            }

            // Split into prefix and suffix around the rest marker
            let prefix: Vec<_> = parts[..idx]
                .iter()
                .filter_map(|p| match p {
                    ArrayPatternPart::Pattern(sp) => Some(sp.clone()),
                    ArrayPatternPart::Rest => None,
                })
                .collect();
            let suffix: Vec<_> = parts[idx + 1..]
                .iter()
                .filter_map(|p| match p {
                    ArrayPatternPart::Pattern(sp) => Some(sp.clone()),
                    ArrayPatternPart::Rest => None,
                })
                .collect();

            Pattern::ArrayRest { prefix, suffix }
        } else {
            // No rest marker - regular array pattern
            let patterns: Vec<_> = parts
                .into_iter()
                .filter_map(|p| match p {
                    ArrayPatternPart::Pattern(sp) => Some(sp),
                    ArrayPatternPart::Rest => None,
                })
                .collect();
            Pattern::Array(patterns)
        }
    }
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

    // v0.37: Wrapping arithmetic (no overflow panic)
    AddWrap,
    SubWrap,
    MulWrap,

    // v0.38: Checked arithmetic (returns Option<T>)
    AddChecked,
    SubChecked,
    MulChecked,

    // v0.38: Saturating arithmetic (clamps to min/max)
    AddSat,
    SubSat,
    MulSat,

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

    // v0.32: Shift operators
    Shl,
    Shr,

    // v0.36: Bitwise operators
    Band,
    Bor,
    Bxor,

    // v0.36: Logical implication (for contracts)
    Implies,
}

impl std::fmt::Display for BinOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BinOp::Add => write!(f, "+"),
            BinOp::Sub => write!(f, "-"),
            BinOp::Mul => write!(f, "*"),
            BinOp::Div => write!(f, "/"),
            BinOp::Mod => write!(f, "%"),
            // v0.37: Wrapping arithmetic
            BinOp::AddWrap => write!(f, "+%"),
            BinOp::SubWrap => write!(f, "-%"),
            BinOp::MulWrap => write!(f, "*%"),
            // v0.38: Checked arithmetic
            BinOp::AddChecked => write!(f, "+?"),
            BinOp::SubChecked => write!(f, "-?"),
            BinOp::MulChecked => write!(f, "*?"),
            // v0.38: Saturating arithmetic
            BinOp::AddSat => write!(f, "+|"),
            BinOp::SubSat => write!(f, "-|"),
            BinOp::MulSat => write!(f, "*|"),
            BinOp::Eq => write!(f, "=="),
            BinOp::Ne => write!(f, "!="),
            BinOp::Lt => write!(f, "<"),
            BinOp::Gt => write!(f, ">"),
            BinOp::Le => write!(f, "<="),
            BinOp::Ge => write!(f, ">="),
            BinOp::And => write!(f, "and"),
            BinOp::Or => write!(f, "or"),
            BinOp::Shl => write!(f, "<<"),
            BinOp::Shr => write!(f, ">>"),
            // v0.36: Bitwise operators
            BinOp::Band => write!(f, "band"),
            BinOp::Bor => write!(f, "bor"),
            BinOp::Bxor => write!(f, "bxor"),
            // v0.36: Logical implication
            BinOp::Implies => write!(f, "implies"),
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
    /// v0.36: Bitwise not
    Bnot,
}

impl std::fmt::Display for UnOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UnOp::Neg => write!(f, "-"),
            UnOp::Not => write!(f, "not"),
            UnOp::Bnot => write!(f, "bnot"),
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
