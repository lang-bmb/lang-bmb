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

/// Top-level item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Item {
    FnDef(FnDef),
}

/// Function definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FnDef {
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
