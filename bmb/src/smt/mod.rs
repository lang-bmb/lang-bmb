//! SMT solver integration module
//!
//! This module generates SMT-LIB2 format and invokes Z3 as external process
//! for contract verification (pre/post conditions).

mod translator;
mod solver;

pub use translator::{SmtTranslator, SmtLibGenerator, SmtSort, TranslateError};
pub use solver::{SmtSolver, SolverResult, VerifyResult, Counterexample};
