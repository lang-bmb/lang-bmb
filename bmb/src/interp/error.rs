//! Runtime errors for the interpreter

use std::fmt;

/// Runtime error during interpretation
#[derive(Debug, Clone)]
pub struct RuntimeError {
    pub kind: ErrorKind,
    pub message: String,
}

/// Kinds of runtime errors
#[derive(Debug, Clone)]
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
    /// Control flow: break from loop (with optional value)
    Break(Option<Box<crate::interp::Value>>),
    /// Control flow: continue to next loop iteration
    Continue,
    /// Control flow: early return from function (with value)
    Return(Box<crate::interp::Value>),
}

impl PartialEq for ErrorKind {
    fn eq(&self, other: &Self) -> bool {
        // Compare discriminants only — Break/Continue/Return are control flow,
        // not real errors, so exact value comparison isn't needed
        std::mem::discriminant(self) == std::mem::discriminant(other)
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    // ---- Cycle 75: Interpreter error tests ----

    #[test]
    fn test_undefined_variable() {
        let err = RuntimeError::undefined_variable("foo");
        assert_eq!(err.kind, ErrorKind::UndefinedVariable);
        assert!(err.message.contains("foo"));
    }

    #[test]
    fn test_undefined_function() {
        let err = RuntimeError::undefined_function("bar");
        assert_eq!(err.kind, ErrorKind::UndefinedFunction);
        assert!(err.message.contains("bar"));
    }

    #[test]
    fn test_type_error() {
        let err = RuntimeError::type_error("i64", "bool");
        assert_eq!(err.kind, ErrorKind::TypeError);
        assert!(err.message.contains("i64"));
        assert!(err.message.contains("bool"));
    }

    #[test]
    fn test_division_by_zero() {
        let err = RuntimeError::division_by_zero();
        assert_eq!(err.kind, ErrorKind::DivisionByZero);
        assert!(err.message.contains("division by zero"));
    }

    #[test]
    fn test_assertion_failed_with_message() {
        let err = RuntimeError::assertion_failed(Some("x > 0"));
        assert_eq!(err.kind, ErrorKind::AssertionFailed);
        assert!(err.message.contains("x > 0"));
    }

    #[test]
    fn test_assertion_failed_without_message() {
        let err = RuntimeError::assertion_failed(None);
        assert_eq!(err.kind, ErrorKind::AssertionFailed);
        assert!(err.message.contains("assertion failed"));
    }

    #[test]
    fn test_arity_mismatch() {
        let err = RuntimeError::arity_mismatch("foo", 3, 2);
        assert_eq!(err.kind, ErrorKind::ArityMismatch);
        assert!(err.message.contains("foo"));
        assert!(err.message.contains("3"));
        assert!(err.message.contains("2"));
    }

    #[test]
    fn test_pre_condition_failed() {
        let err = RuntimeError::pre_condition_failed("my_func");
        assert_eq!(err.kind, ErrorKind::PreConditionFailed);
        assert!(err.message.contains("my_func"));
    }

    #[test]
    fn test_stack_overflow() {
        let err = RuntimeError::stack_overflow();
        assert_eq!(err.kind, ErrorKind::StackOverflow);
        assert!(err.message.contains("stack overflow"));
    }

    #[test]
    fn test_io_error() {
        let err = RuntimeError::io_error("file not found");
        assert_eq!(err.kind, ErrorKind::IoError);
        assert!(err.message.contains("file not found"));
    }

    #[test]
    fn test_index_out_of_bounds() {
        let err = RuntimeError::index_out_of_bounds(5, 3);
        assert_eq!(err.kind, ErrorKind::IndexOutOfBounds);
        assert!(err.message.contains("5"));
        assert!(err.message.contains("3"));
    }

    #[test]
    fn test_todo() {
        let err = RuntimeError::todo("not yet implemented");
        assert_eq!(err.kind, ErrorKind::TodoNotImplemented);
        assert!(err.message.contains("not yet implemented"));
    }

    #[test]
    fn test_display() {
        let err = RuntimeError::division_by_zero();
        let display = format!("{}", err);
        assert!(display.starts_with("Runtime error:"));
        assert!(display.contains("division by zero"));
    }

    #[test]
    fn test_error_kind_eq() {
        assert_eq!(ErrorKind::UndefinedVariable, ErrorKind::UndefinedVariable);
        assert_eq!(ErrorKind::TypeError, ErrorKind::TypeError);
        assert_ne!(ErrorKind::UndefinedVariable, ErrorKind::TypeError);
    }

    #[test]
    fn test_error_clone() {
        let err = RuntimeError::type_error("i64", "bool");
        let cloned = err.clone();
        assert_eq!(err.kind, cloned.kind);
        assert_eq!(err.message, cloned.message);
    }

    #[test]
    fn test_interp_result_ok() {
        let result: InterpResult<i64> = Ok(42);
        assert!(result.is_ok());
    }

    #[test]
    fn test_interp_result_err() {
        let result: InterpResult<i64> = Err(RuntimeError::division_by_zero());
        assert!(result.is_err());
    }

    // --- Cycle 1230: Additional Interpreter Error Tests ---

    #[test]
    fn test_error_kind_break_eq_discriminant() {
        // Break variants compare by discriminant only
        let b1 = ErrorKind::Break(None);
        let b2 = ErrorKind::Break(Some(Box::new(crate::interp::Value::Int(42))));
        assert_eq!(b1, b2); // Same discriminant
    }

    #[test]
    fn test_error_kind_continue_eq() {
        assert_eq!(ErrorKind::Continue, ErrorKind::Continue);
    }

    #[test]
    fn test_error_kind_return_eq_discriminant() {
        let r1 = ErrorKind::Return(Box::new(crate::interp::Value::Int(1)));
        let r2 = ErrorKind::Return(Box::new(crate::interp::Value::Int(2)));
        assert_eq!(r1, r2); // Same discriminant
    }

    #[test]
    fn test_error_kind_break_ne_continue() {
        assert_ne!(ErrorKind::Break(None), ErrorKind::Continue);
    }

    #[test]
    fn test_error_kind_all_variants_ne() {
        let kinds = vec![
            ErrorKind::UndefinedVariable,
            ErrorKind::UndefinedFunction,
            ErrorKind::TypeError,
            ErrorKind::DivisionByZero,
            ErrorKind::AssertionFailed,
            ErrorKind::ArityMismatch,
            ErrorKind::PreConditionFailed,
            ErrorKind::StackOverflow,
            ErrorKind::IoError,
            ErrorKind::IndexOutOfBounds,
            ErrorKind::TodoNotImplemented,
            ErrorKind::Break(None),
            ErrorKind::Continue,
            ErrorKind::Return(Box::new(crate::interp::Value::Unit)),
        ];
        for i in 0..kinds.len() {
            for j in (i + 1)..kinds.len() {
                assert_ne!(kinds[i], kinds[j], "kinds[{}] should != kinds[{}]", i, j);
            }
        }
    }

    #[test]
    fn test_display_all_constructors() {
        let errors = vec![
            RuntimeError::undefined_variable("x"),
            RuntimeError::undefined_function("f"),
            RuntimeError::type_error("i64", "bool"),
            RuntimeError::division_by_zero(),
            RuntimeError::assertion_failed(Some("msg")),
            RuntimeError::assertion_failed(None),
            RuntimeError::arity_mismatch("fn", 2, 3),
            RuntimeError::pre_condition_failed("pre"),
            RuntimeError::stack_overflow(),
            RuntimeError::io_error("err"),
            RuntimeError::index_out_of_bounds(5, 3),
            RuntimeError::todo("wip"),
        ];
        for err in errors {
            let display = format!("{}", err);
            assert!(display.starts_with("Runtime error:"));
        }
    }

    #[test]
    fn test_error_is_std_error() {
        let err = RuntimeError::division_by_zero();
        let _: &dyn std::error::Error = &err;
    }

    #[test]
    fn test_undefined_variable_message_format() {
        let err = RuntimeError::undefined_variable("my_var");
        assert_eq!(err.message, "undefined variable: my_var");
    }

    #[test]
    fn test_undefined_function_message_format() {
        let err = RuntimeError::undefined_function("my_func");
        assert_eq!(err.message, "undefined function: my_func");
    }

    #[test]
    fn test_type_error_message_format() {
        let err = RuntimeError::type_error("string", "i64");
        assert_eq!(err.message, "type error: expected string, got i64");
    }

    #[test]
    fn test_arity_mismatch_message_format() {
        let err = RuntimeError::arity_mismatch("add", 2, 1);
        assert_eq!(err.message, "function add expects 2 argument(s), got 1");
    }

    #[test]
    fn test_index_out_of_bounds_negative() {
        let err = RuntimeError::index_out_of_bounds(-1, 5);
        assert!(err.message.contains("-1"));
        assert!(err.message.contains("5"));
    }

    #[test]
    fn test_todo_message_format() {
        let err = RuntimeError::todo("feature X");
        assert_eq!(err.message, "todo: feature X");
    }

    #[test]
    fn test_error_debug() {
        let err = RuntimeError::division_by_zero();
        let debug = format!("{:?}", err);
        assert!(debug.contains("DivisionByZero"));
    }

    // ================================================================
    // Additional interpreter error tests (Cycle 1238)
    // ================================================================

    #[test]
    fn test_error_kind_clone_break_some() {
        let kind = ErrorKind::Break(Some(Box::new(crate::interp::Value::Int(99))));
        let cloned = kind.clone();
        assert_eq!(kind, cloned);
    }

    #[test]
    fn test_error_kind_clone_return_value() {
        let kind = ErrorKind::Return(Box::new(crate::interp::Value::Bool(true)));
        let cloned = kind.clone();
        assert_eq!(kind, cloned);
    }

    #[test]
    fn test_error_kind_clone_continue() {
        let kind = ErrorKind::Continue;
        let cloned = kind.clone();
        assert_eq!(kind, cloned);
    }

    #[test]
    fn test_division_by_zero_message_exact() {
        let err = RuntimeError::division_by_zero();
        assert_eq!(err.message, "division by zero");
    }

    #[test]
    fn test_stack_overflow_message_exact() {
        let err = RuntimeError::stack_overflow();
        assert_eq!(err.message, "stack overflow: too deep recursion");
    }

    #[test]
    fn test_io_error_message_exact() {
        let err = RuntimeError::io_error("disk full");
        assert_eq!(err.message, "IO error: disk full");
    }

    #[test]
    fn test_pre_condition_failed_message_exact() {
        let err = RuntimeError::pre_condition_failed("validate");
        assert_eq!(err.message, "pre-condition failed for function: validate");
    }

    #[test]
    fn test_runtime_error_error_source_none() {
        let err = RuntimeError::division_by_zero();
        let std_err: &dyn std::error::Error = &err;
        assert!(std_err.source().is_none());
    }

    #[test]
    fn test_error_kind_debug_all_variants() {
        let variants: Vec<ErrorKind> = vec![
            ErrorKind::UndefinedVariable,
            ErrorKind::UndefinedFunction,
            ErrorKind::TypeError,
            ErrorKind::DivisionByZero,
            ErrorKind::AssertionFailed,
            ErrorKind::ArityMismatch,
            ErrorKind::PreConditionFailed,
            ErrorKind::StackOverflow,
            ErrorKind::IoError,
            ErrorKind::IndexOutOfBounds,
            ErrorKind::TodoNotImplemented,
            ErrorKind::Continue,
        ];
        for v in &variants {
            let debug = format!("{:?}", v);
            assert!(!debug.is_empty());
        }
    }

    #[test]
    fn test_interp_result_unwrap_or_default() {
        let ok_result: InterpResult<i64> = Ok(42);
        assert_eq!(ok_result.unwrap_or(0), 42);

        let err_result: InterpResult<i64> = Err(RuntimeError::division_by_zero());
        assert_eq!(err_result.unwrap_or(0), 0);
    }
}
