//! Stub code generation when LLVM is not available
//!
//! This module provides error types for when the llvm feature is not enabled.

use thiserror::Error;

use crate::mir::MirProgram;

/// Code generation error
#[derive(Debug, Error)]
pub enum CodeGenError {
    #[error("LLVM code generation is not available. Build with --features llvm")]
    LlvmNotAvailable,

    #[error("LLVM error: {0}")]
    LlvmError(String),
}

/// Result type for code generation
pub type CodeGenResult<T> = Result<T, CodeGenError>;

/// Stub code generator that always returns an error
pub struct CodeGen;

impl CodeGen {
    /// Create a new code generator (stub)
    pub fn new() -> Self {
        Self
    }

    /// Compile MIR to object file (stub - always fails)
    pub fn compile(&self, _program: &MirProgram, _output: &std::path::Path) -> CodeGenResult<()> {
        Err(CodeGenError::LlvmNotAvailable)
    }

    /// Generate LLVM IR as string (stub - always fails)
    pub fn generate_ir(&self, _program: &MirProgram) -> CodeGenResult<String> {
        Err(CodeGenError::LlvmNotAvailable)
    }
}

impl Default for CodeGen {
    fn default() -> Self {
        Self::new()
    }
}
