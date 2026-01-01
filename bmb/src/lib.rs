//! BMB Compiler Library
//!
//! AI-Native programming language with contract-based verification.

pub mod ast;
pub mod error;
pub mod lexer;
pub mod parser;
pub mod types;

pub use ast::Span;
pub use error::{CompileError, Result};
