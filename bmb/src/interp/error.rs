//! Runtime errors for the interpreter

use std::fmt;

/// Runtime error during interpretation
#[derive(Debug, Clone)]
pub struct RuntimeError {
    pub kind: ErrorKind,
    pub message: String,
}

/// Kinds of runtime errors
#[derive(Debug, Clone, PartialEq)]
pub enum ErrorKind {
    /// Undefined variable
    UndefinedVariable,
    /// Undefined function
    UndefinedFunction,
    /// Type mismatch
    TypeError,
    /// Division by zero
    DivisionByZero,
    /// Assertion failed
    AssertionFailed,
    /// Argument count mismatch
    ArityMismatch,
    /// Pre-condition violated (runtime check)
    PreConditionFailed,
    /// Stack overflow (deep recursion)
    StackOverflow,
    /// IO error
    IoError,
    /// Index out of bounds
    IndexOutOfBounds,
    /// v0.31: Todo placeholder reached at runtime
    TodoNotImplemented,
}

impl RuntimeError {
    pub fn undefined_variable(name: &str) -> Self {
        RuntimeError {
            kind: ErrorKind::UndefinedVariable,
            message: format!("undefined variable: {name}"),
        }
    }

    pub fn undefined_function(name: &str) -> Self {
        RuntimeError {
            kind: ErrorKind::UndefinedFunction,
            message: format!("undefined function: {name}"),
        }
    }

    pub fn type_error(expected: &str, got: &str) -> Self {
        RuntimeError {
            kind: ErrorKind::TypeError,
            message: format!("type error: expected {expected}, got {got}"),
        }
    }

    pub fn division_by_zero() -> Self {
        RuntimeError {
            kind: ErrorKind::DivisionByZero,
            message: "division by zero".to_string(),
        }
    }

    pub fn assertion_failed(msg: Option<&str>) -> Self {
        RuntimeError {
            kind: ErrorKind::AssertionFailed,
            message: msg
                .map(|m| format!("assertion failed: {m}"))
                .unwrap_or_else(|| "assertion failed".to_string()),
        }
    }

    pub fn arity_mismatch(name: &str, expected: usize, got: usize) -> Self {
        RuntimeError {
            kind: ErrorKind::ArityMismatch,
            message: format!(
                "function {name} expects {expected} argument(s), got {got}"
            ),
        }
    }

    pub fn pre_condition_failed(func: &str) -> Self {
        RuntimeError {
            kind: ErrorKind::PreConditionFailed,
            message: format!("pre-condition failed for function: {func}"),
        }
    }

    pub fn stack_overflow() -> Self {
        RuntimeError {
            kind: ErrorKind::StackOverflow,
            message: "stack overflow: too deep recursion".to_string(),
        }
    }

    pub fn io_error(msg: &str) -> Self {
        RuntimeError {
            kind: ErrorKind::IoError,
            message: format!("IO error: {msg}"),
        }
    }

    pub fn index_out_of_bounds(index: i64, len: usize) -> Self {
        RuntimeError {
            kind: ErrorKind::IndexOutOfBounds,
            message: format!("index {} out of bounds for length {}", index, len),
        }
    }

    /// v0.31: Todo placeholder reached at runtime
    pub fn todo(msg: &str) -> Self {
        RuntimeError {
            kind: ErrorKind::TodoNotImplemented,
            message: format!("todo: {msg}"),
        }
    }
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Runtime error: {}", self.message)
    }
}

impl std::error::Error for RuntimeError {}

/// Result type for interpreter operations
pub type InterpResult<T> = Result<T, RuntimeError>;
