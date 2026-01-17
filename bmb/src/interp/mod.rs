//! Interpreter module for BMB

mod env;
mod error;
mod eval;
mod scope;
mod value;

pub use env::{child_env, EnvRef, Environment};
pub use error::{ErrorKind, InterpResult, RuntimeError};
pub use eval::{set_program_args, BuiltinFn, Interpreter};
pub use scope::ScopeStack;
pub use value::Value;
