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

// --- Cycle 64: Tests ---

#[cfg(test)]
mod tests {
    use super::*;

    fn dummy_span() -> Span {
        Span { start: 0, end: 0 }
    }

    fn spanned<T>(node: T) -> Spanned<T> {
        Spanned { node, span: dummy_span() }
    }

    #[test]
    fn test_attribute_simple_name() {
        let attr = Attribute::Simple {
            name: spanned("inline".to_string()),
            span: dummy_span(),
        };
        assert_eq!(attr.name(), "inline");
        assert!(!attr.is_trust());
        assert!(attr.reason().is_none());
    }

    #[test]
    fn test_attribute_with_args_name() {
        let attr = Attribute::WithArgs {
            name: spanned("decreases".to_string()),
            args: vec![spanned(Expr::Var("n".to_string()))],
            span: dummy_span(),
        };
        assert_eq!(attr.name(), "decreases");
        assert!(!attr.is_trust());
        assert!(attr.reason().is_none());
    }

    #[test]
    fn test_attribute_trust_with_reason() {
        let attr = Attribute::WithReason {
            name: spanned("trust".to_string()),
            reason: spanned("verified externally".to_string()),
            span: dummy_span(),
        };
        assert_eq!(attr.name(), "trust");
        assert!(attr.is_trust());
        assert_eq!(attr.reason(), Some("verified externally"));
    }

    #[test]
    fn test_attribute_with_reason_non_trust() {
        let attr = Attribute::WithReason {
            name: spanned("other".to_string()),
            reason: spanned("some reason".to_string()),
            span: dummy_span(),
        };
        assert_eq!(attr.name(), "other");
        assert!(!attr.is_trust());
        assert_eq!(attr.reason(), Some("some reason"));
    }

    #[test]
    fn test_abi_display() {
        assert_eq!(Abi::Bmb.to_string(), "bmb");
        assert_eq!(Abi::C.to_string(), "C");
        assert_eq!(Abi::System.to_string(), "system");
    }

    #[test]
    fn test_abi_default() {
        let abi = Abi::default();
        assert_eq!(abi, Abi::Bmb);
    }

    #[test]
    fn test_visibility_variants() {
        let pub_vis = Visibility::Public;
        let priv_vis = Visibility::Private;
        assert_ne!(format!("{:?}", pub_vis), format!("{:?}", priv_vis));
    }

    // --- Cycle 1225: AST Struct Construction Tests ---

    #[test]
    fn test_visibility_default_is_private() {
        assert_eq!(Visibility::default(), Visibility::Private);
    }

    #[test]
    fn test_attribute_simple_span() {
        let attr = Attribute::Simple {
            name: spanned("inline".to_string()),
            span: Span::new(10, 20),
        };
        assert_eq!(attr.span(), Span::new(10, 20));
    }

    #[test]
    fn test_attribute_with_args_span() {
        let attr = Attribute::WithArgs {
            name: spanned("decreases".to_string()),
            args: vec![],
            span: Span::new(5, 30),
        };
        assert_eq!(attr.span(), Span::new(5, 30));
    }

    #[test]
    fn test_attribute_with_reason_span() {
        let attr = Attribute::WithReason {
            name: spanned("trust".to_string()),
            reason: spanned("safe".to_string()),
            span: Span::new(0, 15),
        };
        assert_eq!(attr.span(), Span::new(0, 15));
    }

    #[test]
    fn test_program_empty() {
        let prog = Program {
            header: None,
            items: vec![],
        };
        assert!(prog.header.is_none());
        assert!(prog.items.is_empty());
    }

    #[test]
    fn test_module_header_construction() {
        let header = ModuleHeader {
            name: spanned("math.arithmetic".to_string()),
            version: Some(spanned("1.0.0".to_string())),
            summary: Some(spanned("integer arithmetic".to_string())),
            exports: vec![spanned("add".to_string()), spanned("sub".to_string())],
            depends: vec![],
            span: dummy_span(),
        };
        assert_eq!(header.name.node, "math.arithmetic");
        assert_eq!(header.version.as_ref().unwrap().node, "1.0.0");
        assert_eq!(header.summary.as_ref().unwrap().node, "integer arithmetic");
        assert_eq!(header.exports.len(), 2);
        assert!(header.depends.is_empty());
    }

    #[test]
    fn test_module_dependency_construction() {
        let dep = ModuleDependency {
            module_path: spanned("core.types".to_string()),
            imports: vec![spanned("i64".to_string()), spanned("i128".to_string())],
            span: dummy_span(),
        };
        assert_eq!(dep.module_path.node, "core.types");
        assert_eq!(dep.imports.len(), 2);
        assert_eq!(dep.imports[0].node, "i64");
    }

    #[test]
    fn test_struct_def_construction() {
        let sd = StructDef {
            attributes: vec![],
            visibility: Visibility::Public,
            name: spanned("Point".to_string()),
            type_params: vec![],
            fields: vec![
                StructField {
                    name: spanned("x".to_string()),
                    ty: spanned(Type::I64),
                },
                StructField {
                    name: spanned("y".to_string()),
                    ty: spanned(Type::I64),
                },
            ],
            span: dummy_span(),
        };
        assert_eq!(sd.name.node, "Point");
        assert_eq!(sd.visibility, Visibility::Public);
        assert_eq!(sd.fields.len(), 2);
        assert_eq!(sd.fields[0].name.node, "x");
    }

    #[test]
    fn test_enum_def_construction() {
        let ed = EnumDef {
            attributes: vec![],
            visibility: Visibility::Private,
            name: spanned("Color".to_string()),
            type_params: vec![],
            variants: vec![
                EnumVariant {
                    name: spanned("Red".to_string()),
                    fields: vec![],
                },
                EnumVariant {
                    name: spanned("Green".to_string()),
                    fields: vec![],
                },
                EnumVariant {
                    name: spanned("Rgb".to_string()),
                    fields: vec![spanned(Type::I64), spanned(Type::I64), spanned(Type::I64)],
                },
            ],
            span: dummy_span(),
        };
        assert_eq!(ed.name.node, "Color");
        assert_eq!(ed.variants.len(), 3);
        assert!(ed.variants[0].fields.is_empty());
        assert_eq!(ed.variants[2].fields.len(), 3);
    }

    #[test]
    fn test_fn_def_basic() {
        let fd = FnDef {
            attributes: vec![],
            visibility: Visibility::Private,
            is_async: false,
            name: spanned("add".to_string()),
            type_params: vec![],
            params: vec![
                Param {
                    name: spanned("a".to_string()),
                    ty: spanned(Type::I64),
                },
                Param {
                    name: spanned("b".to_string()),
                    ty: spanned(Type::I64),
                },
            ],
            ret_name: None,
            ret_ty: spanned(Type::I64),
            pre: None,
            post: None,
            contracts: vec![],
            body: spanned(Expr::IntLit(0)),
            span: dummy_span(),
        };
        assert_eq!(fd.name.node, "add");
        assert!(!fd.is_async);
        assert_eq!(fd.params.len(), 2);
        assert!(fd.contracts.is_empty());
    }

    #[test]
    fn test_fn_def_with_contracts() {
        let fd = FnDef {
            attributes: vec![],
            visibility: Visibility::Public,
            is_async: false,
            name: spanned("safe_div".to_string()),
            type_params: vec![],
            params: vec![
                Param { name: spanned("a".to_string()), ty: spanned(Type::I64) },
                Param { name: spanned("b".to_string()), ty: spanned(Type::I64) },
            ],
            ret_name: Some(spanned("r".to_string())),
            ret_ty: spanned(Type::I64),
            pre: None,
            post: None,
            contracts: vec![
                NamedContract {
                    name: Some(spanned("nonzero_divisor".to_string())),
                    condition: spanned(Expr::BoolLit(true)),
                    span: dummy_span(),
                },
            ],
            body: spanned(Expr::IntLit(0)),
            span: dummy_span(),
        };
        assert_eq!(fd.contracts.len(), 1);
        assert_eq!(fd.contracts[0].name.as_ref().unwrap().node, "nonzero_divisor");
        assert_eq!(fd.ret_name.as_ref().unwrap().node, "r");
    }

    #[test]
    fn test_extern_fn_construction() {
        let ef = ExternFn {
            attributes: vec![],
            visibility: Visibility::Public,
            abi: Abi::C,
            link_name: Some("libc".to_string()),
            name: spanned("printf".to_string()),
            params: vec![],
            ret_ty: spanned(Type::I64),
            span: dummy_span(),
        };
        assert_eq!(ef.name.node, "printf");
        assert_eq!(ef.abi, Abi::C);
        assert_eq!(ef.link_name.as_deref(), Some("libc"));
    }

    #[test]
    fn test_trait_def_construction() {
        let td = TraitDef {
            attributes: vec![],
            visibility: Visibility::Public,
            name: spanned("Display".to_string()),
            type_params: vec![],
            methods: vec![
                TraitMethod {
                    name: spanned("to_string".to_string()),
                    params: vec![],
                    ret_ty: spanned(Type::String),
                    span: dummy_span(),
                },
            ],
            span: dummy_span(),
        };
        assert_eq!(td.name.node, "Display");
        assert_eq!(td.methods.len(), 1);
        assert_eq!(td.methods[0].name.node, "to_string");
    }

    #[test]
    fn test_impl_block_construction() {
        let ib = ImplBlock {
            attributes: vec![],
            type_params: vec![],
            trait_name: spanned("Display".to_string()),
            target_type: spanned(Type::Named("Point".to_string())),
            methods: vec![],
            span: dummy_span(),
        };
        assert_eq!(ib.trait_name.node, "Display");
        assert!(ib.methods.is_empty());
    }

    #[test]
    fn test_type_alias_basic() {
        let ta = TypeAliasDef {
            attributes: vec![],
            visibility: Visibility::Private,
            name: spanned("Index".to_string()),
            type_params: vec![],
            target: spanned(Type::I64),
            refinement: None,
            span: dummy_span(),
        };
        assert_eq!(ta.name.node, "Index");
        assert!(ta.refinement.is_none());
        assert!(ta.type_params.is_empty());
    }

    #[test]
    fn test_type_alias_with_refinement() {
        let ta = TypeAliasDef {
            attributes: vec![],
            visibility: Visibility::Public,
            name: spanned("PosInt".to_string()),
            type_params: vec![],
            target: spanned(Type::I64),
            refinement: Some(spanned(Expr::BoolLit(true))),
            span: dummy_span(),
        };
        assert!(ta.refinement.is_some());
    }

    #[test]
    fn test_use_stmt_construction() {
        let us = UseStmt {
            path: vec![
                spanned("lexer".to_string()),
                spanned("Token".to_string()),
            ],
            span: dummy_span(),
        };
        assert_eq!(us.path.len(), 2);
        assert_eq!(us.path[0].node, "lexer");
        assert_eq!(us.path[1].node, "Token");
    }

    #[test]
    fn test_named_contract_without_name() {
        let nc = NamedContract {
            name: None,
            condition: spanned(Expr::BoolLit(true)),
            span: dummy_span(),
        };
        assert!(nc.name.is_none());
    }

    #[test]
    fn test_program_with_struct_item() {
        let sd = StructDef {
            attributes: vec![],
            visibility: Visibility::Public,
            name: spanned("Point".to_string()),
            type_params: vec![],
            fields: vec![],
            span: dummy_span(),
        };
        let prog = Program {
            header: None,
            items: vec![Item::StructDef(sd)],
        };
        assert_eq!(prog.items.len(), 1);
        assert!(matches!(&prog.items[0], Item::StructDef(s) if s.name.node == "Point"));
    }

    #[test]
    fn test_abi_equality() {
        assert_eq!(Abi::C, Abi::C);
        assert_eq!(Abi::Bmb, Abi::Bmb);
        assert_eq!(Abi::System, Abi::System);
        assert_ne!(Abi::C, Abi::Bmb);
        assert_ne!(Abi::C, Abi::System);
    }

    // ================================================================
    // Additional AST tests (Cycle 1234)
    // ================================================================

    #[test]
    fn test_abi_copy() {
        let abi = Abi::C;
        let abi2 = abi; // Copy
        assert_eq!(abi, abi2);
    }

    #[test]
    fn test_visibility_copy() {
        let vis = Visibility::Public;
        let vis2 = vis; // Copy
        assert_eq!(vis, vis2);
    }

    #[test]
    fn test_module_header_no_optional_fields() {
        let header = ModuleHeader {
            name: spanned("minimal".to_string()),
            version: None,
            summary: None,
            exports: vec![],
            depends: vec![],
            span: dummy_span(),
        };
        assert!(header.version.is_none());
        assert!(header.summary.is_none());
        assert!(header.exports.is_empty());
        assert!(header.depends.is_empty());
    }

    #[test]
    fn test_module_dependency_empty_imports() {
        let dep = ModuleDependency {
            module_path: spanned("core".to_string()),
            imports: vec![],
            span: dummy_span(),
        };
        assert_eq!(dep.module_path.node, "core");
        assert!(dep.imports.is_empty());
    }

    #[test]
    fn test_enum_variant_unit_vs_tuple() {
        let unit_variant = EnumVariant {
            name: spanned("None".to_string()),
            fields: vec![],
        };
        let tuple_variant = EnumVariant {
            name: spanned("Some".to_string()),
            fields: vec![spanned(Type::I64)],
        };
        assert!(unit_variant.fields.is_empty());
        assert_eq!(tuple_variant.fields.len(), 1);
    }

    #[test]
    fn test_fn_def_async_flag() {
        let fd = FnDef {
            attributes: vec![],
            visibility: Visibility::Public,
            is_async: true,
            name: spanned("fetch".to_string()),
            type_params: vec![],
            params: vec![],
            ret_name: None,
            ret_ty: spanned(Type::I64),
            pre: None,
            post: None,
            contracts: vec![],
            body: spanned(Expr::IntLit(0)),
            span: dummy_span(),
        };
        assert!(fd.is_async);
    }

    #[test]
    fn test_use_stmt_single_segment() {
        let us = UseStmt {
            path: vec![spanned("math".to_string())],
            span: dummy_span(),
        };
        assert_eq!(us.path.len(), 1);
        assert_eq!(us.path[0].node, "math");
    }

    #[test]
    fn test_struct_field_clone() {
        let field = StructField {
            name: spanned("x".to_string()),
            ty: spanned(Type::F64),
        };
        let cloned = field.clone();
        assert_eq!(cloned.name.node, "x");
    }

    #[test]
    fn test_program_with_header() {
        let header = ModuleHeader {
            name: spanned("test_mod".to_string()),
            version: Some(spanned("0.1.0".to_string())),
            summary: None,
            exports: vec![],
            depends: vec![],
            span: dummy_span(),
        };
        let prog = Program {
            header: Some(header),
            items: vec![],
        };
        assert!(prog.header.is_some());
        assert_eq!(prog.header.as_ref().unwrap().name.node, "test_mod");
    }

    #[test]
    fn test_type_alias_with_type_params() {
        let ta = TypeAliasDef {
            attributes: vec![],
            visibility: Visibility::Public,
            name: spanned("Result".to_string()),
            type_params: vec![
                TypeParam { name: "T".to_string(), bounds: vec![] },
                TypeParam { name: "E".to_string(), bounds: vec![] },
            ],
            target: spanned(Type::I64),
            refinement: None,
            span: dummy_span(),
        };
        assert_eq!(ta.type_params.len(), 2);
        assert_eq!(ta.type_params[0].name, "T");
        assert_eq!(ta.type_params[1].name, "E");
    }

    // ================================================================
    // Additional AST tests (Cycle 1242)
    // ================================================================

    #[test]
    fn test_attribute_name_all_variants() {
        let simple = Attribute::Simple {
            name: spanned("inline".to_string()),
            span: dummy_span(),
        };
        let with_args = Attribute::WithArgs {
            name: spanned("cfg".to_string()),
            args: vec![],
            span: dummy_span(),
        };
        let with_reason = Attribute::WithReason {
            name: spanned("trust".to_string()),
            reason: spanned("safe".to_string()),
            span: dummy_span(),
        };
        assert_eq!(simple.name(), "inline");
        assert_eq!(with_args.name(), "cfg");
        assert_eq!(with_reason.name(), "trust");
    }

    #[test]
    fn test_trait_method_with_params() {
        let tm = TraitMethod {
            name: spanned("compare".to_string()),
            params: vec![
                Param { name: spanned("self".to_string()), ty: spanned(Type::I64) },
                Param { name: spanned("other".to_string()), ty: spanned(Type::I64) },
            ],
            ret_ty: spanned(Type::Bool),
            span: dummy_span(),
        };
        assert_eq!(tm.name.node, "compare");
        assert_eq!(tm.params.len(), 2);
        assert_eq!(tm.params[0].name.node, "self");
    }

    #[test]
    fn test_fn_def_with_type_params() {
        let fd = FnDef {
            attributes: vec![],
            visibility: Visibility::Public,
            is_async: false,
            name: spanned("identity".to_string()),
            type_params: vec![TypeParam::new("T")],
            params: vec![Param {
                name: spanned("x".to_string()),
                ty: spanned(Type::TypeVar("T".to_string())),
            }],
            ret_name: None,
            ret_ty: spanned(Type::TypeVar("T".to_string())),
            pre: None,
            post: None,
            contracts: vec![],
            body: spanned(Expr::Var("x".to_string())),
            span: dummy_span(),
        };
        assert_eq!(fd.type_params.len(), 1);
        assert_eq!(fd.type_params[0].name, "T");
    }

    #[test]
    fn test_visibility_debug_format() {
        let pub_debug = format!("{:?}", Visibility::Public);
        let priv_debug = format!("{:?}", Visibility::Private);
        assert_eq!(pub_debug, "Public");
        assert_eq!(priv_debug, "Private");
    }

    #[test]
    fn test_abi_debug_format() {
        assert_eq!(format!("{:?}", Abi::Bmb), "Bmb");
        assert_eq!(format!("{:?}", Abi::C), "C");
        assert_eq!(format!("{:?}", Abi::System), "System");
    }

    #[test]
    fn test_extern_fn_no_link_name() {
        let ef = ExternFn {
            attributes: vec![],
            visibility: Visibility::Public,
            abi: Abi::Bmb,
            link_name: None,
            name: spanned("internal_fn".to_string()),
            params: vec![],
            ret_ty: spanned(Type::Unit),
            span: dummy_span(),
        };
        assert!(ef.link_name.is_none());
        assert_eq!(ef.abi, Abi::Bmb);
    }

    #[test]
    fn test_named_contract_with_name() {
        let nc = NamedContract {
            name: Some(spanned("positive_input".to_string())),
            condition: spanned(Expr::BoolLit(true)),
            span: dummy_span(),
        };
        assert_eq!(nc.name.as_ref().unwrap().node, "positive_input");
    }

    #[test]
    fn test_impl_block_with_type_params() {
        let ib = ImplBlock {
            attributes: vec![],
            type_params: vec![TypeParam::new("T")],
            trait_name: spanned("Container".to_string()),
            target_type: spanned(Type::Generic {
                name: "Vec".to_string(),
                type_args: vec![Box::new(Type::TypeVar("T".to_string()))],
            }),
            methods: vec![],
            span: dummy_span(),
        };
        assert_eq!(ib.type_params.len(), 1);
        assert_eq!(ib.type_params[0].name, "T");
    }

    #[test]
    fn test_struct_def_with_type_params() {
        let sd = StructDef {
            attributes: vec![],
            visibility: Visibility::Public,
            name: spanned("Pair".to_string()),
            type_params: vec![TypeParam::new("T"), TypeParam::new("U")],
            fields: vec![
                StructField {
                    name: spanned("first".to_string()),
                    ty: spanned(Type::TypeVar("T".to_string())),
                },
                StructField {
                    name: spanned("second".to_string()),
                    ty: spanned(Type::TypeVar("U".to_string())),
                },
            ],
            span: dummy_span(),
        };
        assert_eq!(sd.type_params.len(), 2);
        assert_eq!(sd.fields.len(), 2);
    }

    #[test]
    fn test_enum_def_with_attributes() {
        let ed = EnumDef {
            attributes: vec![
                Attribute::Simple {
                    name: spanned("derive".to_string()),
                    span: dummy_span(),
                },
            ],
            visibility: Visibility::Public,
            name: spanned("Option".to_string()),
            type_params: vec![TypeParam::new("T")],
            variants: vec![
                EnumVariant { name: spanned("Some".to_string()), fields: vec![spanned(Type::TypeVar("T".to_string()))] },
                EnumVariant { name: spanned("None".to_string()), fields: vec![] },
            ],
            span: dummy_span(),
        };
        assert_eq!(ed.attributes.len(), 1);
        assert_eq!(ed.attributes[0].name(), "derive");
        assert_eq!(ed.type_params.len(), 1);
        assert_eq!(ed.variants.len(), 2);
    }
}
