//! Abstract Syntax Tree definitions

mod expr;
mod span;
mod types;

pub use expr::*;
pub use span::*;
pub use types::*;

use serde::{Deserialize, Serialize};

/// A program is a sequence of top-level items
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Program {
    pub items: Vec<Item>,
}

/// Visibility modifier (v0.5 Phase 4)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Visibility {
    /// Private (default)
    Private,
    /// Public (pub keyword)
    Public,
}

impl Default for Visibility {
    fn default() -> Self {
        Visibility::Private
    }
}

/// Top-level item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Item {
    FnDef(FnDef),
    StructDef(StructDef),
    EnumDef(EnumDef),
    /// Use statement: use path::to::item (v0.5 Phase 4)
    Use(UseStmt),
}

/// Use statement (v0.5 Phase 4)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UseStmt {
    /// Path segments (e.g., ["lexer", "Token"] for use lexer::Token)
    pub path: Vec<Spanned<String>>,
    /// Span of the entire use statement
    pub span: Span,
}

/// Struct definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructDef {
    pub visibility: Visibility,
    pub name: Spanned<String>,
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
    pub visibility: Visibility,
    pub name: Spanned<String>,
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

/// Function definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FnDef {
    pub visibility: Visibility,
    pub name: Spanned<String>,
    pub params: Vec<Param>,
    pub ret_ty: Spanned<Type>,
    pub pre: Option<Spanned<Expr>>,
    pub post: Option<Spanned<Expr>>,
    pub body: Spanned<Expr>,
    pub span: Span,
}

/// Function parameter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Param {
    pub name: Spanned<String>,
    pub ty: Spanned<Type>,
}
