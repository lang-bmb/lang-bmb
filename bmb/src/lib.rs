//! BMB Compiler Library
//!
//! AI-Native programming language with contract-based verification.
//!
//! # Deprecation Notice (v0.31.21)
//!
//! This Rust implementation is deprecated in favor of the self-hosted BMB bootstrap compiler.
//! The BMB bootstrap (`bootstrap/*.bmb`) provides feature parity for core compiler components.
//!
//! **Archive**: `archive/rust-v0.31` branch preserves this implementation.
//! **Migration**: See `docs/TRANSITION.md` for the transition roadmap.
//!
//! ## Component Status
//!
//! | Module | BMB Equivalent | Status |
//! |--------|----------------|--------|
//! | `lexer` | `bootstrap/lexer.bmb` | Deprecated |
//! | `parser` | `bootstrap/parser.bmb` | Deprecated |
//! | `types` | `bootstrap/types.bmb` | Deprecated |
//! | `mir` | `bootstrap/mir.bmb` | Deprecated |
//! | `codegen` | `bootstrap/llvm_ir.bmb` | Deprecated |
//! | `interp` | N/A | Still Required |
//! | `repl` | N/A | Still Required |
//! | `lsp` | N/A | Still Required |

// v0.30.299: Allow clippy false positives
// - only_used_in_recursion: Tree traversal functions use &self for consistency, not just recursion
// - large_enum_variant: AST node size differences are by design
// - should_implement_trait: from_str methods are intentional, not FromStr trait
// - type_complexity: Complex function types are necessary for type system
#![allow(clippy::only_used_in_recursion)]
#![allow(clippy::large_enum_variant)]
#![allow(clippy::should_implement_trait)]
#![allow(clippy::type_complexity)]

pub mod ast;
pub mod build;
pub mod cfg;
pub mod cir;
pub mod codegen;
pub mod derive;
pub mod error;
pub mod index;
pub mod interp;
pub mod lexer;
pub mod lsp;
pub mod mir;
pub mod parser;
pub mod pir;
pub mod preprocessor;
pub mod query;
pub mod repl;
pub mod resolver;
pub mod smt;
pub mod types;
pub mod util;
pub mod verify;

pub use ast::Span;
pub use error::{CompileError, Result};
