//! Expression AST nodes

use super::{Span, Spanned, Type};
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
    /// Null pointer literal (v0.51.40)
    Null,
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

    /// Let binding without initializer: `let [mut] name: Type; body` (v0.60.21)
    /// Used for stack-allocated arrays: `let mut arr: [i64; 100];`
    /// Only allowed for array types - uninitialized primitives are dangerous
    LetUninit {
        name: String,
        mutable: bool,
        ty: Box<Spanned<Type>>,
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

    /// v0.60.22: Array repeat: [val; N] - creates array of N elements with value val
    ArrayRepeat {
        value: Box<Spanned<Expr>>,
        count: usize,
    },

    /// v0.42: Tuple expression: (expr1, expr2, ...)
    Tuple(Vec<Spanned<Expr>>),

    /// Index access: `expr[index]`
    Index {
        expr: Box<Spanned<Expr>>,
        index: Box<Spanned<Expr>>,
    },

    /// Index assignment: `expr[index] = value` (v0.51)
    IndexAssign {
        array: Box<Spanned<Expr>>,
        index: Box<Spanned<Expr>>,
        value: Box<Spanned<Expr>>,
    },

    /// Field assignment: `obj.field = value` (v0.51.23)
    /// Enables mutable struct patterns for C-like performance
    FieldAssign {
        object: Box<Spanned<Expr>>,
        field: Spanned<String>,
        value: Box<Spanned<Expr>>,
    },

    /// Dereference assignment: `set *ptr = value` (v0.60.21)
    /// Stores value through a native pointer, generating PtrStore MIR instruction
    DerefAssign {
        ptr: Box<Spanned<Expr>>,
        value: Box<Spanned<Expr>>,
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

    /// Size of type in bytes (v0.51.41)
    /// Example: sizeof<Node>() returns the size of Node struct in bytes
    Sizeof {
        /// The type to get the size of
        ty: Spanned<Type>,
    },

    // v0.70: Concurrency primitives

    /// Spawn expression: spawn { expr } -> Thread<T>
    /// Creates a new thread that executes the body expression.
    /// Variables are captured by value (move semantics).
    Spawn {
        /// The expression to execute in the spawned thread
        body: Box<Spanned<Expr>>,
    },

    // v0.72: Atomic creation expression

    /// Atomic creation: Atomic::new(value) -> Atomic<T>
    /// Creates a new atomic variable with the given initial value.
    /// The type T is inferred from the value.
    AtomicNew {
        /// The initial value for the atomic
        value: Box<Spanned<Expr>>,
    },

    // v0.71: Mutex creation expression

    /// Mutex creation: Mutex::new(value) -> Mutex<T>
    /// Creates a new mutex wrapping the given value.
    /// The type T is inferred from the value.
    MutexNew {
        /// The initial value to wrap in the mutex
        value: Box<Spanned<Expr>>,
    },

    // v0.73: Channel creation expression

    /// Channel creation: channel<T>(capacity) -> (Sender<T>, Receiver<T>)
    /// Creates a bounded MPSC channel with the specified capacity.
    ChannelNew {
        /// Element type of the channel
        elem_ty: Box<Spanned<Type>>,
        /// Channel buffer capacity
        capacity: Box<Spanned<Expr>>,
    },

    // v0.74: Advanced synchronization primitives

    /// RwLock creation: RwLock::new(value) -> RwLock<T>
    /// Creates a reader-writer lock with the given initial value.
    RwLockNew {
        /// The initial value to wrap in the RwLock
        value: Box<Spanned<Expr>>,
    },

    /// Barrier creation: Barrier::new(count) -> Barrier
    /// Creates a barrier for the specified number of threads.
    BarrierNew {
        /// Number of threads that must call wait() before all are released
        count: Box<Spanned<Expr>>,
    },

    /// Condvar creation: Condvar::new() -> Condvar
    /// Creates a new condition variable.
    CondvarNew,

    // v0.75: Async/await expressions

    /// Await expression: expr.await
    /// Suspends execution until the future completes and returns the result.
    Await {
        /// The future expression to await
        future: Box<Spanned<Expr>>,
    },

    // v0.82: Select macro for multi-channel operations

    /// Select expression: select { arm1, arm2, ... }
    /// Waits on multiple channels/futures simultaneously.
    /// Returns the result of the first arm that becomes ready.
    Select {
        /// The select arms to wait on
        arms: Vec<SelectArm>,
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

/// A single arm in a select expression (v0.82)
/// Example: `value = rx.recv() => { process(value) }`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectArm {
    /// Variable binding for the result (None for `_`)
    pub binding: Option<String>,
    /// The channel/future operation (e.g., rx.recv(), timeout(100))
    pub operation: Spanned<Expr>,
    /// Optional guard condition
    pub guard: Option<Spanned<Expr>>,
    /// Arm body expression
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

/// v0.89.4: Desugar statement-style let bindings in blocks into nested Let expressions.
///
/// Transforms: `{ let x = 1; let y = 2; x + y }` → `Let(x, 1, Let(y, 2, Block([x + y])))`
///
/// Let bindings with `Expr::Unit` body (from BlockStmt grammar) are "statement-style" lets.
/// This function nests them so the type checker and MIR lowering handle scoping correctly.
pub fn desugar_block_lets(stmts: Vec<Spanned<Expr>>) -> Expr {
    if stmts.is_empty() {
        return Expr::Block(stmts);
    }

    // Check if any statement is a statement-style Let (body is Unit)
    let has_stmt_let = stmts.iter().any(|s| {
        matches!(&s.node,
            Expr::Let { body, .. } | Expr::LetUninit { body, .. }
            if matches!(body.node, Expr::Unit))
    });

    if !has_stmt_let {
        return Expr::Block(stmts);
    }

    // Build the expression from left to right, nesting Lets around remaining statements
    desugar_stmts(stmts)
}

fn desugar_stmts(stmts: Vec<Spanned<Expr>>) -> Expr {
    if stmts.is_empty() {
        return Expr::Unit;
    }
    if stmts.len() == 1 {
        return stmts.into_iter().next().unwrap().node;
    }

    let mut stmts = stmts;
    let first = stmts.remove(0);
    let first_span = first.span;

    match first.node {
        Expr::Let { name, mutable, ty, value, body } if matches!(body.node, Expr::Unit) => {
            // Statement-style let: make remaining statements the body
            let rest = desugar_stmts(stmts);
            let rest_span = Span::new(first_span.end, first_span.end + 1);
            Expr::Let {
                name,
                mutable,
                ty,
                value,
                body: Box::new(Spanned::new(rest, rest_span)),
            }
        }
        Expr::LetUninit { name, mutable, ty, body } if matches!(body.node, Expr::Unit) => {
            let rest = desugar_stmts(stmts);
            let rest_span = Span::new(first_span.end, first_span.end + 1);
            Expr::LetUninit {
                name,
                mutable,
                ty,
                body: Box::new(Spanned::new(rest, rest_span)),
            }
        }
        other => {
            // Non-let statement: wrap in Block with remaining statements
            let mut all = vec![Spanned::new(other, first_span)];
            // Check if remaining has any statement-style lets
            let rest_has_let = stmts.iter().any(|s| {
                matches!(&s.node,
                    Expr::Let { body, .. } | Expr::LetUninit { body, .. }
                    if matches!(body.node, Expr::Unit))
            });
            if rest_has_let {
                let rest = desugar_stmts(stmts);
                let rest_span = Span::new(first_span.end, first_span.end + 1);
                all.push(Spanned::new(rest, rest_span));
                Expr::Block(all)
            } else {
                all.extend(stmts);
                Expr::Block(all)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---- Cycle 72: BinOp Display tests ----

    #[test]
    fn test_binop_display_arithmetic() {
        assert_eq!(format!("{}", BinOp::Add), "+");
        assert_eq!(format!("{}", BinOp::Sub), "-");
        assert_eq!(format!("{}", BinOp::Mul), "*");
        assert_eq!(format!("{}", BinOp::Div), "/");
        assert_eq!(format!("{}", BinOp::Mod), "%");
    }

    #[test]
    fn test_binop_display_wrapping() {
        assert_eq!(format!("{}", BinOp::AddWrap), "+%");
        assert_eq!(format!("{}", BinOp::SubWrap), "-%");
        assert_eq!(format!("{}", BinOp::MulWrap), "*%");
    }

    #[test]
    fn test_binop_display_checked() {
        assert_eq!(format!("{}", BinOp::AddChecked), "+?");
        assert_eq!(format!("{}", BinOp::SubChecked), "-?");
        assert_eq!(format!("{}", BinOp::MulChecked), "*?");
    }

    #[test]
    fn test_binop_display_saturating() {
        assert_eq!(format!("{}", BinOp::AddSat), "+|");
        assert_eq!(format!("{}", BinOp::SubSat), "-|");
        assert_eq!(format!("{}", BinOp::MulSat), "*|");
    }

    #[test]
    fn test_binop_display_comparison() {
        assert_eq!(format!("{}", BinOp::Eq), "==");
        assert_eq!(format!("{}", BinOp::Ne), "!=");
        assert_eq!(format!("{}", BinOp::Lt), "<");
        assert_eq!(format!("{}", BinOp::Gt), ">");
        assert_eq!(format!("{}", BinOp::Le), "<=");
        assert_eq!(format!("{}", BinOp::Ge), ">=");
    }

    #[test]
    fn test_binop_display_logical() {
        assert_eq!(format!("{}", BinOp::And), "and");
        assert_eq!(format!("{}", BinOp::Or), "or");
        assert_eq!(format!("{}", BinOp::Implies), "implies");
    }

    #[test]
    fn test_binop_display_bitwise() {
        assert_eq!(format!("{}", BinOp::Band), "band");
        assert_eq!(format!("{}", BinOp::Bor), "bor");
        assert_eq!(format!("{}", BinOp::Bxor), "bxor");
        assert_eq!(format!("{}", BinOp::Shl), "<<");
        assert_eq!(format!("{}", BinOp::Shr), ">>");
    }

    // ---- UnOp Display tests ----

    #[test]
    fn test_unop_display() {
        assert_eq!(format!("{}", UnOp::Neg), "-");
        assert_eq!(format!("{}", UnOp::Not), "not");
        assert_eq!(format!("{}", UnOp::Bnot), "bnot");
    }

    // ---- RangeKind Display tests ----

    #[test]
    fn test_range_kind_display() {
        assert_eq!(format!("{}", RangeKind::Exclusive), "..<");
        assert_eq!(format!("{}", RangeKind::Inclusive), "..=");
    }

    #[test]
    fn test_range_kind_eq() {
        assert_eq!(RangeKind::Exclusive, RangeKind::Exclusive);
        assert_eq!(RangeKind::Inclusive, RangeKind::Inclusive);
        assert_ne!(RangeKind::Exclusive, RangeKind::Inclusive);
    }

    // ---- StateKind Display tests ----

    #[test]
    fn test_state_kind_display() {
        assert_eq!(format!("{}", StateKind::Pre), ".pre");
        assert_eq!(format!("{}", StateKind::Post), ".post");
    }

    #[test]
    fn test_state_kind_eq() {
        assert_eq!(StateKind::Pre, StateKind::Pre);
        assert_ne!(StateKind::Pre, StateKind::Post);
    }

    // ---- BinOp Copy/Clone tests ----

    #[test]
    fn test_binop_copy() {
        let op = BinOp::Add;
        let op2 = op; // Copy
        assert_eq!(format!("{}", op), format!("{}", op2));
    }

    // --- Cycle 1225: Expr Variant & Pattern Tests ---

    fn dummy_span() -> Span {
        Span { start: 0, end: 0 }
    }

    fn spanned<T>(node: T) -> Spanned<T> {
        Spanned { node, span: dummy_span() }
    }

    #[test]
    fn test_expr_intlit() {
        let e = Expr::IntLit(42);
        assert!(matches!(e, Expr::IntLit(42)));
    }

    #[test]
    fn test_expr_floatlit() {
        let e = Expr::FloatLit(3.14);
        assert!(matches!(e, Expr::FloatLit(v) if (v - 3.14).abs() < f64::EPSILON));
    }

    #[test]
    fn test_expr_boollit() {
        assert!(matches!(Expr::BoolLit(true), Expr::BoolLit(true)));
        assert!(matches!(Expr::BoolLit(false), Expr::BoolLit(false)));
    }

    #[test]
    fn test_expr_stringlit() {
        let e = Expr::StringLit("hello".to_string());
        assert!(matches!(e, Expr::StringLit(ref s) if s == "hello"));
    }

    #[test]
    fn test_expr_charlit() {
        let e = Expr::CharLit('x');
        assert!(matches!(e, Expr::CharLit('x')));
    }

    #[test]
    fn test_expr_null_unit() {
        assert!(matches!(Expr::Null, Expr::Null));
        assert!(matches!(Expr::Unit, Expr::Unit));
    }

    #[test]
    fn test_expr_var() {
        let e = Expr::Var("x".to_string());
        assert!(matches!(e, Expr::Var(ref n) if n == "x"));
    }

    #[test]
    fn test_expr_binary() {
        let e = Expr::Binary {
            left: Box::new(spanned(Expr::IntLit(1))),
            op: BinOp::Add,
            right: Box::new(spanned(Expr::IntLit(2))),
        };
        assert!(matches!(e, Expr::Binary { op: BinOp::Add, .. }));
    }

    #[test]
    fn test_expr_unary() {
        let e = Expr::Unary {
            op: UnOp::Neg,
            expr: Box::new(spanned(Expr::IntLit(5))),
        };
        assert!(matches!(e, Expr::Unary { op: UnOp::Neg, .. }));
    }

    #[test]
    fn test_expr_if() {
        let e = Expr::If {
            cond: Box::new(spanned(Expr::BoolLit(true))),
            then_branch: Box::new(spanned(Expr::IntLit(1))),
            else_branch: Box::new(spanned(Expr::IntLit(0))),
        };
        assert!(matches!(e, Expr::If { .. }));
    }

    #[test]
    fn test_expr_let() {
        let e = Expr::Let {
            name: "x".to_string(),
            mutable: false,
            ty: None,
            value: Box::new(spanned(Expr::IntLit(42))),
            body: Box::new(spanned(Expr::Var("x".to_string()))),
        };
        assert!(matches!(e, Expr::Let { mutable: false, .. }));
    }

    #[test]
    fn test_expr_let_mut() {
        let e = Expr::Let {
            name: "x".to_string(),
            mutable: true,
            ty: Some(spanned(Type::I64)),
            value: Box::new(spanned(Expr::IntLit(0))),
            body: Box::new(spanned(Expr::Unit)),
        };
        assert!(matches!(e, Expr::Let { mutable: true, .. }));
    }

    #[test]
    fn test_expr_while() {
        let e = Expr::While {
            cond: Box::new(spanned(Expr::BoolLit(true))),
            invariant: None,
            body: Box::new(spanned(Expr::Unit)),
        };
        assert!(matches!(e, Expr::While { invariant: None, .. }));
    }

    #[test]
    fn test_expr_for() {
        let e = Expr::For {
            var: "i".to_string(),
            iter: Box::new(spanned(Expr::Range {
                start: Box::new(spanned(Expr::IntLit(0))),
                end: Box::new(spanned(Expr::IntLit(10))),
                kind: RangeKind::Exclusive,
            })),
            body: Box::new(spanned(Expr::Unit)),
        };
        assert!(matches!(e, Expr::For { ref var, .. } if var == "i"));
    }

    #[test]
    fn test_expr_loop_break_continue() {
        assert!(matches!(Expr::Loop { body: Box::new(spanned(Expr::Break { value: None })) }, Expr::Loop { .. }));
        assert!(matches!(Expr::Break { value: None }, Expr::Break { value: None }));
        assert!(matches!(Expr::Continue, Expr::Continue));
    }

    #[test]
    fn test_expr_return() {
        let e = Expr::Return { value: Some(Box::new(spanned(Expr::IntLit(42)))) };
        assert!(matches!(e, Expr::Return { value: Some(_) }));
        let e2 = Expr::Return { value: None };
        assert!(matches!(e2, Expr::Return { value: None }));
    }

    #[test]
    fn test_expr_call() {
        let e = Expr::Call {
            func: "add".to_string(),
            args: vec![spanned(Expr::IntLit(1)), spanned(Expr::IntLit(2))],
        };
        assert!(matches!(e, Expr::Call { ref func, ref args } if func == "add" && args.len() == 2));
    }

    #[test]
    fn test_expr_block() {
        let e = Expr::Block(vec![spanned(Expr::IntLit(1)), spanned(Expr::IntLit(2))]);
        assert!(matches!(e, Expr::Block(ref v) if v.len() == 2));
    }

    #[test]
    fn test_expr_struct_init() {
        let e = Expr::StructInit {
            name: "Point".to_string(),
            fields: vec![
                (spanned("x".to_string()), spanned(Expr::IntLit(1))),
                (spanned("y".to_string()), spanned(Expr::IntLit(2))),
            ],
        };
        assert!(matches!(e, Expr::StructInit { ref name, ref fields } if name == "Point" && fields.len() == 2));
    }

    #[test]
    fn test_expr_field_access() {
        let e = Expr::FieldAccess {
            expr: Box::new(spanned(Expr::Var("p".to_string()))),
            field: spanned("x".to_string()),
        };
        assert!(matches!(e, Expr::FieldAccess { .. }));
    }

    #[test]
    fn test_expr_tuple_field() {
        let e = Expr::TupleField {
            expr: Box::new(spanned(Expr::Var("t".to_string()))),
            index: 0,
        };
        assert!(matches!(e, Expr::TupleField { index: 0, .. }));
    }

    #[test]
    fn test_expr_enum_variant() {
        let e = Expr::EnumVariant {
            enum_name: "Option".to_string(),
            variant: "Some".to_string(),
            args: vec![spanned(Expr::IntLit(42))],
        };
        assert!(matches!(e, Expr::EnumVariant { ref variant, .. } if variant == "Some"));
    }

    #[test]
    fn test_expr_match() {
        let e = Expr::Match {
            expr: Box::new(spanned(Expr::Var("x".to_string()))),
            arms: vec![
                MatchArm {
                    pattern: spanned(Pattern::Literal(LiteralPattern::Int(1))),
                    guard: None,
                    body: spanned(Expr::StringLit("one".to_string())),
                },
                MatchArm {
                    pattern: spanned(Pattern::Wildcard),
                    guard: None,
                    body: spanned(Expr::StringLit("other".to_string())),
                },
            ],
        };
        assert!(matches!(e, Expr::Match { ref arms, .. } if arms.len() == 2));
    }

    #[test]
    fn test_expr_ref_deref_refmut() {
        let inner = spanned(Expr::Var("x".to_string()));
        assert!(matches!(Expr::Ref(Box::new(inner.clone())), Expr::Ref(_)));
        assert!(matches!(Expr::Deref(Box::new(inner.clone())), Expr::Deref(_)));
        assert!(matches!(Expr::RefMut(Box::new(inner)), Expr::RefMut(_)));
    }

    #[test]
    fn test_expr_array_lit() {
        let e = Expr::ArrayLit(vec![spanned(Expr::IntLit(1)), spanned(Expr::IntLit(2))]);
        assert!(matches!(e, Expr::ArrayLit(ref v) if v.len() == 2));
    }

    #[test]
    fn test_expr_array_repeat() {
        let e = Expr::ArrayRepeat {
            value: Box::new(spanned(Expr::IntLit(0))),
            count: 100,
        };
        assert!(matches!(e, Expr::ArrayRepeat { count: 100, .. }));
    }

    #[test]
    fn test_expr_tuple() {
        let e = Expr::Tuple(vec![spanned(Expr::IntLit(1)), spanned(Expr::BoolLit(true))]);
        assert!(matches!(e, Expr::Tuple(ref v) if v.len() == 2));
    }

    #[test]
    fn test_expr_index() {
        let e = Expr::Index {
            expr: Box::new(spanned(Expr::Var("arr".to_string()))),
            index: Box::new(spanned(Expr::IntLit(0))),
        };
        assert!(matches!(e, Expr::Index { .. }));
    }

    #[test]
    fn test_expr_index_assign() {
        let e = Expr::IndexAssign {
            array: Box::new(spanned(Expr::Var("arr".to_string()))),
            index: Box::new(spanned(Expr::IntLit(0))),
            value: Box::new(spanned(Expr::IntLit(42))),
        };
        assert!(matches!(e, Expr::IndexAssign { .. }));
    }

    #[test]
    fn test_expr_method_call() {
        let e = Expr::MethodCall {
            receiver: Box::new(spanned(Expr::Var("v".to_string()))),
            method: "len".to_string(),
            args: vec![],
        };
        assert!(matches!(e, Expr::MethodCall { ref method, .. } if method == "len"));
    }

    #[test]
    fn test_expr_closure() {
        let e = Expr::Closure {
            params: vec![
                ClosureParam {
                    name: spanned("x".to_string()),
                    ty: Some(spanned(Type::I64)),
                },
            ],
            ret_ty: None,
            body: Box::new(spanned(Expr::Var("x".to_string()))),
        };
        assert!(matches!(e, Expr::Closure { ref params, .. } if params.len() == 1));
    }

    #[test]
    fn test_expr_cast() {
        let e = Expr::Cast {
            expr: Box::new(spanned(Expr::IntLit(42))),
            ty: spanned(Type::F64),
        };
        assert!(matches!(e, Expr::Cast { .. }));
    }

    #[test]
    fn test_expr_todo() {
        let e = Expr::Todo { message: Some("not yet".to_string()) };
        assert!(matches!(e, Expr::Todo { message: Some(ref m) } if m == "not yet"));
        let e2 = Expr::Todo { message: None };
        assert!(matches!(e2, Expr::Todo { message: None }));
    }

    #[test]
    fn test_expr_ret_and_it() {
        assert!(matches!(Expr::Ret, Expr::Ret));
        assert!(matches!(Expr::It, Expr::It));
    }

    #[test]
    fn test_expr_state_ref() {
        let e = Expr::StateRef {
            expr: Box::new(spanned(Expr::Var("x".to_string()))),
            state: StateKind::Pre,
        };
        assert!(matches!(e, Expr::StateRef { state: StateKind::Pre, .. }));
    }

    #[test]
    fn test_expr_spawn() {
        let e = Expr::Spawn {
            body: Box::new(spanned(Expr::IntLit(42))),
        };
        assert!(matches!(e, Expr::Spawn { .. }));
    }

    #[test]
    fn test_expr_concurrency_constructors() {
        assert!(matches!(Expr::AtomicNew { value: Box::new(spanned(Expr::IntLit(0))) }, Expr::AtomicNew { .. }));
        assert!(matches!(Expr::MutexNew { value: Box::new(spanned(Expr::IntLit(0))) }, Expr::MutexNew { .. }));
        assert!(matches!(Expr::RwLockNew { value: Box::new(spanned(Expr::IntLit(0))) }, Expr::RwLockNew { .. }));
        assert!(matches!(Expr::BarrierNew { count: Box::new(spanned(Expr::IntLit(4))) }, Expr::BarrierNew { .. }));
        assert!(matches!(Expr::CondvarNew, Expr::CondvarNew));
    }

    // --- Pattern Tests ---

    #[test]
    fn test_pattern_wildcard() {
        assert!(matches!(Pattern::Wildcard, Pattern::Wildcard));
    }

    #[test]
    fn test_pattern_var() {
        let p = Pattern::Var("x".to_string());
        assert!(matches!(p, Pattern::Var(ref n) if n == "x"));
    }

    #[test]
    fn test_pattern_literal_all_variants() {
        assert!(matches!(Pattern::Literal(LiteralPattern::Int(42)), Pattern::Literal(LiteralPattern::Int(42))));
        assert!(matches!(Pattern::Literal(LiteralPattern::Bool(true)), Pattern::Literal(LiteralPattern::Bool(true))));
        let p = Pattern::Literal(LiteralPattern::String("hi".to_string()));
        assert!(matches!(p, Pattern::Literal(LiteralPattern::String(ref s)) if s == "hi"));
    }

    #[test]
    fn test_pattern_enum_variant() {
        let p = Pattern::EnumVariant {
            enum_name: "Option".to_string(),
            variant: "Some".to_string(),
            bindings: vec![spanned(Pattern::Var("x".to_string()))],
        };
        assert!(matches!(p, Pattern::EnumVariant { ref variant, .. } if variant == "Some"));
    }

    #[test]
    fn test_pattern_struct() {
        let p = Pattern::Struct {
            name: "Point".to_string(),
            fields: vec![
                (spanned("x".to_string()), spanned(Pattern::Var("px".to_string()))),
            ],
        };
        assert!(matches!(p, Pattern::Struct { ref name, .. } if name == "Point"));
    }

    #[test]
    fn test_pattern_range() {
        let p = Pattern::Range {
            start: LiteralPattern::Int(1),
            end: LiteralPattern::Int(10),
            inclusive: true,
        };
        assert!(matches!(p, Pattern::Range { inclusive: true, .. }));
    }

    #[test]
    fn test_pattern_or() {
        let p = Pattern::Or(vec![
            spanned(Pattern::Literal(LiteralPattern::Int(1))),
            spanned(Pattern::Literal(LiteralPattern::Int(2))),
        ]);
        assert!(matches!(p, Pattern::Or(ref v) if v.len() == 2));
    }

    #[test]
    fn test_pattern_binding() {
        let p = Pattern::Binding {
            name: "val".to_string(),
            pattern: Box::new(spanned(Pattern::Literal(LiteralPattern::Int(42)))),
        };
        assert!(matches!(p, Pattern::Binding { ref name, .. } if name == "val"));
    }

    #[test]
    fn test_pattern_tuple() {
        let p = Pattern::Tuple(vec![
            spanned(Pattern::Var("a".to_string())),
            spanned(Pattern::Var("b".to_string())),
        ]);
        assert!(matches!(p, Pattern::Tuple(ref v) if v.len() == 2));
    }

    #[test]
    fn test_pattern_array() {
        let p = Pattern::Array(vec![
            spanned(Pattern::Literal(LiteralPattern::Int(1))),
            spanned(Pattern::Literal(LiteralPattern::Int(2))),
        ]);
        assert!(matches!(p, Pattern::Array(ref v) if v.len() == 2));
    }

    #[test]
    fn test_pattern_array_rest() {
        let p = Pattern::ArrayRest {
            prefix: vec![spanned(Pattern::Var("first".to_string()))],
            suffix: vec![spanned(Pattern::Var("last".to_string()))],
        };
        assert!(matches!(p, Pattern::ArrayRest { ref prefix, ref suffix } if prefix.len() == 1 && suffix.len() == 1));
    }

    // --- ArrayPatternPart Tests ---

    #[test]
    fn test_array_pattern_part_no_rest() {
        let parts = vec![
            ArrayPatternPart::Pattern(spanned(Pattern::Var("a".to_string()))),
            ArrayPatternPart::Pattern(spanned(Pattern::Var("b".to_string()))),
        ];
        let result = ArrayPatternPart::into_pattern(parts);
        assert!(matches!(result, Pattern::Array(ref v) if v.len() == 2));
    }

    #[test]
    fn test_array_pattern_part_with_rest() {
        let parts = vec![
            ArrayPatternPart::Pattern(spanned(Pattern::Var("first".to_string()))),
            ArrayPatternPart::Rest,
            ArrayPatternPart::Pattern(spanned(Pattern::Var("last".to_string()))),
        ];
        let result = ArrayPatternPart::into_pattern(parts);
        assert!(matches!(result, Pattern::ArrayRest { ref prefix, ref suffix } if prefix.len() == 1 && suffix.len() == 1));
    }

    #[test]
    fn test_array_pattern_part_rest_at_end() {
        let parts = vec![
            ArrayPatternPart::Pattern(spanned(Pattern::Var("a".to_string()))),
            ArrayPatternPart::Rest,
        ];
        let result = ArrayPatternPart::into_pattern(parts);
        assert!(matches!(result, Pattern::ArrayRest { ref prefix, ref suffix } if prefix.len() == 1 && suffix.is_empty()));
    }

    #[test]
    fn test_array_pattern_part_rest_at_start() {
        let parts = vec![
            ArrayPatternPart::Rest,
            ArrayPatternPart::Pattern(spanned(Pattern::Var("last".to_string()))),
        ];
        let result = ArrayPatternPart::into_pattern(parts);
        assert!(matches!(result, Pattern::ArrayRest { ref prefix, ref suffix } if prefix.is_empty() && suffix.len() == 1));
    }

    // --- desugar_block_lets Tests ---

    #[test]
    fn test_desugar_block_lets_empty() {
        let result = desugar_block_lets(vec![]);
        assert!(matches!(result, Expr::Block(ref v) if v.is_empty()));
    }

    #[test]
    fn test_desugar_block_lets_no_lets() {
        let stmts = vec![
            spanned(Expr::IntLit(1)),
            spanned(Expr::IntLit(2)),
        ];
        let result = desugar_block_lets(stmts);
        assert!(matches!(result, Expr::Block(ref v) if v.len() == 2));
    }

    #[test]
    fn test_desugar_block_lets_with_stmt_let() {
        // let x = 1; x  → Let(x, 1, x)
        let stmts = vec![
            spanned(Expr::Let {
                name: "x".to_string(),
                mutable: false,
                ty: None,
                value: Box::new(spanned(Expr::IntLit(1))),
                body: Box::new(spanned(Expr::Unit)),  // stmt-style: body is Unit
            }),
            spanned(Expr::Var("x".to_string())),
        ];
        let result = desugar_block_lets(stmts);
        assert!(matches!(result, Expr::Let { ref name, .. } if name == "x"));
    }

    #[test]
    fn test_desugar_block_lets_non_stmt_let_passthrough() {
        // Let with non-Unit body is already nested, not a stmt-style let
        let stmts = vec![
            spanned(Expr::Let {
                name: "x".to_string(),
                mutable: false,
                ty: None,
                value: Box::new(spanned(Expr::IntLit(1))),
                body: Box::new(spanned(Expr::Var("x".to_string()))),  // not Unit → normal let
            }),
        ];
        let result = desugar_block_lets(stmts);
        // No stmt-style lets, so returns Block
        assert!(matches!(result, Expr::Block(_)));
    }

    // --- MatchArm and SelectArm ---

    #[test]
    fn test_match_arm_with_guard() {
        let arm = MatchArm {
            pattern: spanned(Pattern::Var("x".to_string())),
            guard: Some(spanned(Expr::Binary {
                left: Box::new(spanned(Expr::Var("x".to_string()))),
                op: BinOp::Gt,
                right: Box::new(spanned(Expr::IntLit(0))),
            })),
            body: spanned(Expr::Var("x".to_string())),
        };
        assert!(arm.guard.is_some());
    }

    #[test]
    fn test_select_arm_construction() {
        let arm = SelectArm {
            binding: Some("msg".to_string()),
            operation: spanned(Expr::Var("rx".to_string())),
            guard: None,
            body: spanned(Expr::Var("msg".to_string())),
        };
        assert_eq!(arm.binding.as_deref(), Some("msg"));
        assert!(arm.guard.is_none());
    }

    #[test]
    fn test_closure_param_with_type() {
        let cp = ClosureParam {
            name: spanned("x".to_string()),
            ty: Some(spanned(Type::I64)),
        };
        assert_eq!(cp.name.node, "x");
        assert!(cp.ty.is_some());
    }

    #[test]
    fn test_closure_param_without_type() {
        let cp = ClosureParam {
            name: spanned("y".to_string()),
            ty: None,
        };
        assert_eq!(cp.name.node, "y");
        assert!(cp.ty.is_none());
    }

    #[test]
    fn test_literal_pattern_float() {
        let lp = LiteralPattern::Float(3.14);
        assert!(matches!(lp, LiteralPattern::Float(v) if (v - 3.14).abs() < f64::EPSILON));
    }

    #[test]
    fn test_expr_sizeof() {
        let e = Expr::Sizeof { ty: spanned(Type::I64) };
        assert!(matches!(e, Expr::Sizeof { .. }));
    }

    #[test]
    fn test_expr_field_assign() {
        let e = Expr::FieldAssign {
            object: Box::new(spanned(Expr::Var("p".to_string()))),
            field: spanned("x".to_string()),
            value: Box::new(spanned(Expr::IntLit(10))),
        };
        assert!(matches!(e, Expr::FieldAssign { .. }));
    }

    #[test]
    fn test_expr_deref_assign() {
        let e = Expr::DerefAssign {
            ptr: Box::new(spanned(Expr::Var("ptr".to_string()))),
            value: Box::new(spanned(Expr::IntLit(42))),
        };
        assert!(matches!(e, Expr::DerefAssign { .. }));
    }

    #[test]
    fn test_expr_let_uninit() {
        let e = Expr::LetUninit {
            name: "arr".to_string(),
            mutable: true,
            ty: Box::new(spanned(Type::Array(Box::new(Type::I64), 100))),
            body: Box::new(spanned(Expr::Unit)),
        };
        assert!(matches!(e, Expr::LetUninit { ref name, mutable: true, .. } if name == "arr"));
    }

    #[test]
    fn test_expr_assign() {
        let e = Expr::Assign {
            name: "x".to_string(),
            value: Box::new(spanned(Expr::IntLit(10))),
        };
        assert!(matches!(e, Expr::Assign { ref name, .. } if name == "x"));
    }

    #[test]
    fn test_expr_range() {
        let e = Expr::Range {
            start: Box::new(spanned(Expr::IntLit(0))),
            end: Box::new(spanned(Expr::IntLit(10))),
            kind: RangeKind::Inclusive,
        };
        assert!(matches!(e, Expr::Range { kind: RangeKind::Inclusive, .. }));
    }

    #[test]
    fn test_expr_await() {
        let e = Expr::Await {
            future: Box::new(spanned(Expr::Var("f".to_string()))),
        };
        assert!(matches!(e, Expr::Await { .. }));
    }

    #[test]
    fn test_expr_channel_new() {
        let e = Expr::ChannelNew {
            elem_ty: Box::new(spanned(Type::I64)),
            capacity: Box::new(spanned(Expr::IntLit(16))),
        };
        assert!(matches!(e, Expr::ChannelNew { .. }));
    }

    #[test]
    fn test_expr_forall_exists() {
        let forall = Expr::Forall {
            var: spanned("i".to_string()),
            ty: spanned(Type::I64),
            body: Box::new(spanned(Expr::BoolLit(true))),
        };
        assert!(matches!(forall, Expr::Forall { .. }));

        let exists = Expr::Exists {
            var: spanned("j".to_string()),
            ty: spanned(Type::I64),
            body: Box::new(spanned(Expr::BoolLit(true))),
        };
        assert!(matches!(exists, Expr::Exists { .. }));
    }

    #[test]
    fn test_expr_select() {
        let e = Expr::Select {
            arms: vec![
                SelectArm {
                    binding: Some("v".to_string()),
                    operation: spanned(Expr::Var("rx1".to_string())),
                    guard: None,
                    body: spanned(Expr::Var("v".to_string())),
                },
            ],
        };
        assert!(matches!(e, Expr::Select { ref arms } if arms.len() == 1));
    }

    #[test]
    fn test_expr_while_with_invariant() {
        let e = Expr::While {
            cond: Box::new(spanned(Expr::BoolLit(true))),
            invariant: Some(Box::new(spanned(Expr::BoolLit(true)))),
            body: Box::new(spanned(Expr::Unit)),
        };
        assert!(matches!(e, Expr::While { invariant: Some(_), .. }));
    }
}
