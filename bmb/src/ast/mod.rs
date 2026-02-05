//! Abstract Syntax Tree definitions

mod expr;
pub mod output;
mod span;
mod types;

pub use expr::*;
pub use span::*;
pub use types::*;

use serde::{Deserialize, Serialize};

/// A program is a sequence of top-level items
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Program {
    /// Optional module header (v0.31: RFC-0002)
    pub header: Option<ModuleHeader>,
    pub items: Vec<Item>,
}

/// Module header (v0.31: RFC-0002)
/// Provides metadata for AI-friendly navigation and dependency tracking
///
/// Syntax:
/// ```bmb
/// module math.arithmetic
///   version 1.0.0
///   summary "integer arithmetic"
///   exports add, subtract
///   depends
///     core.types (i64)
/// ===
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleHeader {
    /// Fully qualified module name (e.g., "math.arithmetic")
    pub name: Spanned<String>,
    /// Optional SemVer version (e.g., "1.0.0")
    pub version: Option<Spanned<String>>,
    /// Optional one-line description
    pub summary: Option<Spanned<String>>,
    /// List of exported symbols
    pub exports: Vec<Spanned<String>>,
    /// Module dependencies
    pub depends: Vec<ModuleDependency>,
    /// Span of the entire header
    pub span: Span,
}

/// Module dependency (v0.31: RFC-0002)
/// Represents an explicit dependency on another module
///
/// Syntax: `core.types (i64, i128)`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleDependency {
    /// Module path (e.g., "core.types")
    pub module_path: Spanned<String>,
    /// Specific imports from the module (e.g., ["i64", "i128"])
    pub imports: Vec<Spanned<String>>,
    /// Span
    pub span: Span,
}

/// Visibility modifier (v0.5 Phase 4)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum Visibility {
    /// Private (default)
    #[default]
    Private,
    /// Public (pub keyword)
    Public,
}

/// Top-level item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Item {
    FnDef(FnDef),
    StructDef(StructDef),
    EnumDef(EnumDef),
    /// Type alias (v0.50.6): type Name = Type; or type Name = Type where cond;
    TypeAlias(TypeAliasDef),
    /// Use statement: use path::to::item (v0.5 Phase 4)
    Use(UseStmt),
    /// External function declaration (v0.13.0): extern fn name(...) -> Type;
    ExternFn(ExternFn),
    /// Trait definition (v0.20.1): trait Name { fn method(...) -> Type; }
    TraitDef(TraitDef),
    /// Impl block (v0.20.1): impl Trait for Type { ... }
    ImplBlock(ImplBlock),
}

/// Use statement (v0.5 Phase 4)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UseStmt {
    /// Path segments (e.g., ["lexer", "Token"] for use lexer::Token)
    pub path: Vec<Spanned<String>>,
    /// Span of the entire use statement
    pub span: Span,
}

/// ABI (Application Binary Interface) specification (v0.20.2)
/// Used to specify calling conventions for FFI
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum Abi {
    /// Default BMB calling convention
    #[default]
    Bmb,
    /// C calling convention (cdecl)
    C,
    /// System calling convention (varies by platform)
    System,
}

impl std::fmt::Display for Abi {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Abi::Bmb => write!(f, "bmb"),
            Abi::C => write!(f, "C"),
            Abi::System => write!(f, "system"),
        }
    }
}

/// External function declaration (v0.13.0, updated v0.20.2)
/// Syntax: extern fn name(params) -> Type;           // Default ABI
/// Syntax: extern "C" fn name(params) -> Type;       // C ABI (v0.20.2)
/// Used for FFI with WASI, libc, or other external libraries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternFn {
    /// Attributes (e.g., @wasi for WASI imports)
    pub attributes: Vec<Attribute>,
    /// Visibility
    pub visibility: Visibility,
    /// ABI specification (v0.20.2): "C", "system", or default
    pub abi: Abi,
    /// External module name (e.g., "wasi_snapshot_preview1")
    /// Specified via @link("module_name") attribute
    pub link_name: Option<String>,
    /// Function name
    pub name: Spanned<String>,
    /// Parameters
    pub params: Vec<Param>,
    /// Return type
    pub ret_ty: Spanned<Type>,
    /// Span
    pub span: Span,
}

/// Trait definition (v0.20.1)
/// Syntax: trait Name { fn method(self) -> Type; }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraitDef {
    /// Attributes
    pub attributes: Vec<Attribute>,
    /// Visibility
    pub visibility: Visibility,
    /// Trait name
    pub name: Spanned<String>,
    /// Type parameters (if any): `trait Container<T> { ... }`
    pub type_params: Vec<TypeParam>,
    /// Trait method signatures (without bodies)
    pub methods: Vec<TraitMethod>,
    /// Span
    pub span: Span,
}

/// Trait method signature (v0.20.1)
/// Method declaration in a trait (without body)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraitMethod {
    /// Method name
    pub name: Spanned<String>,
    /// Parameters (first is typically `self`)
    pub params: Vec<Param>,
    /// Return type
    pub ret_ty: Spanned<Type>,
    /// Span
    pub span: Span,
}

/// Impl block (v0.20.1)
/// Syntax: impl Trait for Type { fn method(self) -> Type = body; }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImplBlock {
    /// Attributes
    pub attributes: Vec<Attribute>,
    /// Type parameters (if any): `impl<T> Trait for Container<T>`
    pub type_params: Vec<TypeParam>,
    /// Trait being implemented
    pub trait_name: Spanned<String>,
    /// Target type (the type implementing the trait)
    pub target_type: Spanned<Type>,
    /// Method implementations
    pub methods: Vec<FnDef>,
    /// Span
    pub span: Span,
}

/// Struct definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructDef {
    /// Attributes (v0.12.3): @cfg, @derive, etc.
    pub attributes: Vec<Attribute>,
    pub visibility: Visibility,
    pub name: Spanned<String>,
    /// Type parameters (v0.13.1): e.g., `<T>`, `<T, U>`, `<T: Ord>`
    pub type_params: Vec<TypeParam>,
    pub fields: Vec<StructField>,
    pub span: Span,
}

/// Struct field
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructField {
    pub name: Spanned<String>,
    pub ty: Spanned<Type>,
}

/// Enum definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnumDef {
    /// Attributes (v0.12.3): @cfg, @derive, etc.
    pub attributes: Vec<Attribute>,
    pub visibility: Visibility,
    pub name: Spanned<String>,
    /// Type parameters (v0.13.1): e.g., `<T>`, `<T, E>`
    pub type_params: Vec<TypeParam>,
    pub variants: Vec<EnumVariant>,
    pub span: Span,
}

/// Enum variant
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnumVariant {
    pub name: Spanned<String>,
    /// Fields for tuple-like or struct-like variants (empty for unit variants)
    pub fields: Vec<Spanned<Type>>,
}

/// Type alias definition (v0.50.6)
/// Syntax: `type Name = Type;`
/// Syntax: `type Name<T> = Type<T>;`
/// Syntax: `type Name = Type where { condition };` (refinement type)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeAliasDef {
    /// Attributes (e.g., `@deprecated`)
    pub attributes: Vec<Attribute>,
    /// Visibility (pub or private)
    pub visibility: Visibility,
    /// Name of the type alias
    pub name: Spanned<String>,
    /// Type parameters for generic type aliases (e.g., `<T>`, `<T, U>`)
    pub type_params: Vec<TypeParam>,
    /// The target type being aliased
    pub target: Spanned<Type>,
    /// Optional refinement condition (for refined types)
    /// e.g., where { self != 0 } or where { self > 0 && self < 100 }
    pub refinement: Option<Spanned<Expr>>,
    /// Span of the entire definition
    pub span: Span,
}

/// Named contract (v0.2)
/// A contract with an optional name for better error messages
/// e.g., `sorted_input: forall(i in 0..<len(arr)-1): arr[i] <= arr[i+1]`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NamedContract {
    /// Optional name for the contract (for error messages)
    /// e.g., "sorted_input", "found_correct"
    pub name: Option<Spanned<String>>,
    /// The contract condition expression
    pub condition: Spanned<Expr>,
    /// Span of the entire contract
    pub span: Span,
}

/// Function definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FnDef {
    /// Attributes (v0.2): @inline, @pure, @decreases, etc.
    pub attributes: Vec<Attribute>,
    pub visibility: Visibility,
    /// v0.75: Whether this is an async function
    pub is_async: bool,
    pub name: Spanned<String>,
    /// Type parameters (v0.13.1): e.g., `<T>`, `<T: Ord, U>`
    pub type_params: Vec<TypeParam>,
    pub params: Vec<Param>,
    /// Optional explicit return value binding name (v0.2)
    /// e.g., `-> r: i64` binds return value to `r`
    /// If None, implicit `ret` is used
    pub ret_name: Option<Spanned<String>>,
    pub ret_ty: Spanned<Type>,
    /// Legacy pre-condition (deprecated in v0.2, use contracts)
    pub pre: Option<Spanned<Expr>>,
    /// Legacy post-condition (deprecated in v0.2, use contracts)
    pub post: Option<Spanned<Expr>>,
    /// Named contracts in where {} block (v0.2)
    /// Replaces pre/post with named, structured contracts
    pub contracts: Vec<NamedContract>,
    pub body: Spanned<Expr>,
    pub span: Span,
}

/// Function parameter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Param {
    pub name: Spanned<String>,
    pub ty: Spanned<Type>,
}

/// Attribute (v0.2, v0.31: @trust "reason")
/// e.g., `@inline`, `@inline(always)`, `@decreases(n)`, `@trust "reason"`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Attribute {
    /// Simple attribute: @name
    Simple {
        name: Spanned<String>,
        span: Span,
    },
    /// Attribute with arguments: @name(arg1, arg2, ...)
    WithArgs {
        name: Spanned<String>,
        args: Vec<Spanned<Expr>>,
        span: Span,
    },
    /// v0.31: Attribute with mandatory reason string: @trust "reason"
    WithReason {
        name: Spanned<String>,
        reason: Spanned<String>,
        span: Span,
    },
}

impl Attribute {
    /// Get the attribute name
    pub fn name(&self) -> &str {
        match self {
            Attribute::Simple { name, .. } => &name.node,
            Attribute::WithArgs { name, .. } => &name.node,
            Attribute::WithReason { name, .. } => &name.node,
        }
    }

    /// Get the span of the attribute
    pub fn span(&self) -> Span {
        match self {
            Attribute::Simple { span, .. } => *span,
            Attribute::WithArgs { span, .. } => *span,
            Attribute::WithReason { span, .. } => *span,
        }
    }

    /// v0.31: Get the reason string for @trust attribute
    pub fn reason(&self) -> Option<&str> {
        match self {
            Attribute::WithReason { reason, .. } => Some(&reason.node),
            _ => None,
        }
    }

    /// v0.31: Check if this is a @trust attribute
    pub fn is_trust(&self) -> bool {
        self.name() == "trust"
    }
}
