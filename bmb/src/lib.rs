//! BMB Compiler Library
//!
//! AI-Native programming language with contract-based verification.

pub mod ast;
pub mod codegen;
pub mod error;
pub mod interp;
pub mod lexer;
pub mod mir;
pub mod parser;
pub mod repl;
pub mod smt;
pub mod types;
pub mod verify;

pub use ast::Span;
pub use error::{CompileError, Result};
