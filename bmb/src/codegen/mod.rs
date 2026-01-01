//! LLVM Code Generation
//!
//! This module generates LLVM IR from MIR using the inkwell crate.
//! It requires the `llvm` feature to be enabled.

#[cfg(feature = "llvm")]
mod llvm;

#[cfg(feature = "llvm")]
pub use llvm::*;

#[cfg(not(feature = "llvm"))]
mod stub;

#[cfg(not(feature = "llvm"))]
pub use stub::*;
