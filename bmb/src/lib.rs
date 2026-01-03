//! BMB Compiler Library
//!
//! AI-Native programming language with contract-based verification.

pub mod ast;
pub mod build;
pub mod cfg;
pub mod codegen;
pub mod derive;
pub mod error;
pub mod interp;
pub mod lexer;
pub mod lsp;
pub mod mir;
pub mod parser;
pub mod repl;
pub mod resolver;
pub mod smt;
pub mod types;
pub mod verify;

pub use ast::Span;
pub use error::{CompileError, Result};
