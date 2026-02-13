//! Error types and reporting

use crate::ast::Span;
use thiserror::Error;

/// Result type alias
pub type Result<T> = std::result::Result<T, CompileError>;

// ============================================================================
// v0.47: Warning Infrastructure
// ============================================================================

/// Compile warning - non-fatal diagnostic messages
/// P0 Correctness: Helps catch potential issues without blocking compilation
#[derive(Debug, Clone)]
pub enum CompileWarning {
    /// Unreachable pattern arm in match expression (v0.47)
    UnreachablePattern {
        message: String,
        span: Span,
        arm_index: usize,
    },

    /// Unused variable binding (v0.47)
    UnusedBinding {
        name: String,
        span: Span,
    },

    /// Redundant pattern (subset of another pattern) (v0.47)
    RedundantPattern {
        message: String,
        span: Span,
    },

    /// Integer range may overflow (v0.47)
    IntegerRangeOverflow {
        message: String,
        span: Span,
    },

    /// v0.51: Match with guards but no unconditional fallback
    /// May fail at runtime if all guards evaluate to false
    GuardedNonExhaustive {
        span: Span,
    },

    /// v0.52: Mutable variable that is never mutated
    /// Should be `let` instead of `var`
    UnusedMut {
        name: String,
        span: Span,
    },

    /// v0.53: Unreachable code after divergent expression
    /// Statement after return, break, or continue will never execute
    UnreachableCode {
        span: Span,
    },

    /// v0.74: Unused import
    /// Import that is never used in the code
    UnusedImport {
        name: String,
        span: Span,
    },

    /// v0.76: Unused function
    /// Private function that is never called
    UnusedFunction {
        name: String,
        span: Span,
    },

    /// v0.77: Unused type/struct
    /// Private type definition that is never used
    UnusedType {
        name: String,
        span: Span,
    },

    /// v0.78: Unused enum
    /// Private enum definition that is never used
    UnusedEnum {
        name: String,
        span: Span,
    },

    /// v0.79: Shadow binding
    /// Variable shadows another binding in an outer scope
    ShadowBinding {
        name: String,
        span: Span,
        original_span: Span,
    },

    /// v0.80: Unused trait
    /// Private trait definition that is never implemented
    UnusedTrait {
        name: String,
        span: Span,
    },

    /// v0.50.11: Duplicate function definition
    /// Function with the same name already defined (later definition wins)
    DuplicateFunction {
        name: String,
        span: Span,
        original_span: Span,
    },

    /// v0.81: Missing postcondition
    /// Function lacks explicit postcondition contract
    MissingPostcondition {
        name: String,
        span: Span,
    },

    /// v0.84: Semantic duplication
    /// Two functions have equivalent contracts (same signature + same postcondition)
    SemanticDuplication {
        name: String,
        duplicate_of: String,
        span: Span,
    },

    /// v0.82: Trivial contract (tautology)
    /// Contract that is always true, providing no meaningful specification
    TrivialContract {
        name: String,
        contract_kind: String, // "precondition", "postcondition", or contract name
        span: Span,
    },

    /// v0.90.122: Single-arm match — suggest if-let
    /// Match with one specific arm and one wildcard could be simplified
    SingleArmMatch {
        span: Span,
    },

    /// v0.90.123: Redundant type cast — casting a value to its own type
    RedundantCast {
        ty: String,
        span: Span,
    },

    /// v0.90.127: Constant condition — if/while with literal true/false
    ConstantCondition {
        value: bool,
        context: String, // "if" or "while"
        span: Span,
    },

    /// v0.90.128: Self-comparison — comparing a variable to itself
    SelfComparison {
        name: String,
        op: String,
        span: Span,
    },

    /// v0.90.129: Redundant boolean comparison — `x == true` or `x == false`
    RedundantBoolComparison {
        value: bool,
        span: Span,
    },

    /// v0.90.130: Duplicate match arm — same pattern appears more than once
    DuplicateMatchArm {
        pattern: String,
        span: Span,
    },

    /// v0.90.131: Integer division truncation — literal division doesn't divide evenly
    IntDivisionTruncation {
        left: i64,
        right: i64,
        span: Span,
    },

    /// v0.90.132: Unused function return value — non-unit return value discarded
    UnusedReturnValue {
        func: String,
        span: Span,
    },

    /// v0.90.121: Non-snake_case function name
    /// Function names should use snake_case
    NonSnakeCaseFunction {
        name: String,
        suggestion: String,
        span: Span,
    },

    /// v0.90.121: Non-PascalCase type name
    /// Struct, enum, and trait names should use PascalCase
    NonPascalCaseType {
        name: String,
        suggestion: String,
        kind: String, // "struct", "enum", or "trait"
        span: Span,
    },

    /// v0.90.139: Identity operation (x + 0, x * 1, x - 0, x / 1)
    /// Expression has no effect and can be simplified
    IdentityOperation {
        expr: String,
        span: Span,
    },

    /// v0.90.140: Negated if condition
    /// `if not x { a } else { b }` can be simplified to `if x { b } else { a }`
    NegatedIfCondition {
        span: Span,
    },

    /// Generic warning with span
    Generic {
        message: String,
        span: Option<Span>,
    },
}

impl CompileWarning {
    /// Create an unreachable pattern warning
    pub fn unreachable_pattern(message: impl Into<String>, span: Span, arm_index: usize) -> Self {
        Self::UnreachablePattern {
            message: message.into(),
            span,
            arm_index,
        }
    }

    /// Create an unused binding warning
    pub fn unused_binding(name: impl Into<String>, span: Span) -> Self {
        Self::UnusedBinding {
            name: name.into(),
            span,
        }
    }

    /// Create a redundant pattern warning
    pub fn redundant_pattern(message: impl Into<String>, span: Span) -> Self {
        Self::RedundantPattern {
            message: message.into(),
            span,
        }
    }

    /// Create an integer range overflow warning
    pub fn integer_range_overflow(message: impl Into<String>, span: Span) -> Self {
        Self::IntegerRangeOverflow {
            message: message.into(),
            span,
        }
    }

    /// Create a generic warning
    pub fn generic(message: impl Into<String>, span: Option<Span>) -> Self {
        Self::Generic {
            message: message.into(),
            span,
        }
    }

    /// v0.51: Create a guarded non-exhaustive warning
    pub fn guarded_non_exhaustive(span: Span) -> Self {
        Self::GuardedNonExhaustive { span }
    }

    /// v0.52: Create an unused mutable binding warning
    pub fn unused_mut(name: impl Into<String>, span: Span) -> Self {
        Self::UnusedMut {
            name: name.into(),
            span,
        }
    }

    /// v0.53: Create an unreachable code warning
    pub fn unreachable_code(span: Span) -> Self {
        Self::UnreachableCode { span }
    }

    /// v0.74: Create an unused import warning
    pub fn unused_import(name: impl Into<String>, span: Span) -> Self {
        Self::UnusedImport {
            name: name.into(),
            span,
        }
    }

    /// v0.76: Create an unused function warning
    pub fn unused_function(name: impl Into<String>, span: Span) -> Self {
        Self::UnusedFunction {
            name: name.into(),
            span,
        }
    }

    /// v0.77: Create an unused type warning
    pub fn unused_type(name: impl Into<String>, span: Span) -> Self {
        Self::UnusedType {
            name: name.into(),
            span,
        }
    }

    /// v0.78: Create an unused enum warning
    pub fn unused_enum(name: impl Into<String>, span: Span) -> Self {
        Self::UnusedEnum {
            name: name.into(),
            span,
        }
    }

    /// v0.79: Create a shadow binding warning
    pub fn shadow_binding(name: impl Into<String>, span: Span, original_span: Span) -> Self {
        Self::ShadowBinding {
            name: name.into(),
            span,
            original_span,
        }
    }

    /// v0.80: Create an unused trait warning
    pub fn unused_trait(name: impl Into<String>, span: Span) -> Self {
        Self::UnusedTrait {
            name: name.into(),
            span,
        }
    }

    /// v0.50.11: Create a duplicate function warning
    pub fn duplicate_function(name: impl Into<String>, span: Span, original_span: Span) -> Self {
        Self::DuplicateFunction {
            name: name.into(),
            span,
            original_span,
        }
    }

    /// v0.81: Create a missing postcondition warning
    pub fn missing_postcondition(name: impl Into<String>, span: Span) -> Self {
        Self::MissingPostcondition {
            name: name.into(),
            span,
        }
    }

    /// v0.84: Create a semantic duplication warning
    pub fn semantic_duplication(
        name: impl Into<String>,
        duplicate_of: impl Into<String>,
        span: Span,
    ) -> Self {
        Self::SemanticDuplication {
            name: name.into(),
            duplicate_of: duplicate_of.into(),
            span,
        }
    }

    /// v0.82: Create a trivial contract warning
    pub fn trivial_contract(
        name: impl Into<String>,
        contract_kind: impl Into<String>,
        span: Span,
    ) -> Self {
        Self::TrivialContract {
            name: name.into(),
            contract_kind: contract_kind.into(),
            span,
        }
    }

    /// v0.90.122: Create a single-arm match warning
    pub fn single_arm_match(span: Span) -> Self {
        Self::SingleArmMatch { span }
    }

    /// v0.90.123: Create a redundant cast warning
    pub fn redundant_cast(ty: impl Into<String>, span: Span) -> Self {
        Self::RedundantCast { ty: ty.into(), span }
    }

    /// v0.90.127: Create a constant condition warning
    pub fn constant_condition(value: bool, context: impl Into<String>, span: Span) -> Self {
        Self::ConstantCondition { value, context: context.into(), span }
    }

    /// v0.90.128: Create a self-comparison warning
    pub fn self_comparison(name: impl Into<String>, op: impl Into<String>, span: Span) -> Self {
        Self::SelfComparison { name: name.into(), op: op.into(), span }
    }

    /// v0.90.129: Create a redundant boolean comparison warning
    pub fn redundant_bool_comparison(value: bool, span: Span) -> Self {
        Self::RedundantBoolComparison { value, span }
    }

    /// v0.90.130: Create a duplicate match arm warning
    pub fn duplicate_match_arm(pattern: impl Into<String>, span: Span) -> Self {
        Self::DuplicateMatchArm { pattern: pattern.into(), span }
    }

    /// v0.90.131: Create an integer division truncation warning
    pub fn int_division_truncation(left: i64, right: i64, span: Span) -> Self {
        Self::IntDivisionTruncation { left, right, span }
    }

    /// v0.90.132: Create an unused return value warning
    pub fn unused_return_value(func: impl Into<String>, span: Span) -> Self {
        Self::UnusedReturnValue { func: func.into(), span }
    }

    /// v0.90.121: Create a non-snake_case function warning
    pub fn non_snake_case_function(
        name: impl Into<String>,
        suggestion: impl Into<String>,
        span: Span,
    ) -> Self {
        Self::NonSnakeCaseFunction {
            name: name.into(),
            suggestion: suggestion.into(),
            span,
        }
    }

    /// v0.90.121: Create a non-PascalCase type warning
    pub fn non_pascal_case_type(
        name: impl Into<String>,
        suggestion: impl Into<String>,
        kind: impl Into<String>,
        span: Span,
    ) -> Self {
        Self::NonPascalCaseType {
            name: name.into(),
            suggestion: suggestion.into(),
            kind: kind.into(),
            span,
        }
    }

    /// v0.90.139: Create an identity operation warning
    pub fn identity_operation(expr: impl Into<String>, span: Span) -> Self {
        Self::IdentityOperation { expr: expr.into(), span }
    }

    /// v0.90.140: Create a negated if condition warning
    pub fn negated_if_condition(span: Span) -> Self {
        Self::NegatedIfCondition { span }
    }

    /// Get the span of this warning, if any
    pub fn span(&self) -> Option<Span> {
        match self {
            Self::UnreachablePattern { span, .. } => Some(*span),
            Self::UnusedBinding { span, .. } => Some(*span),
            Self::RedundantPattern { span, .. } => Some(*span),
            Self::IntegerRangeOverflow { span, .. } => Some(*span),
            Self::GuardedNonExhaustive { span } => Some(*span),
            Self::UnusedMut { span, .. } => Some(*span),
            Self::UnreachableCode { span } => Some(*span),
            Self::UnusedImport { span, .. } => Some(*span),
            Self::UnusedFunction { span, .. } => Some(*span),
            Self::UnusedType { span, .. } => Some(*span),
            Self::UnusedEnum { span, .. } => Some(*span),
            Self::ShadowBinding { span, .. } => Some(*span),
            Self::UnusedTrait { span, .. } => Some(*span),
            Self::DuplicateFunction { span, .. } => Some(*span),
            Self::MissingPostcondition { span, .. } => Some(*span),
            Self::SemanticDuplication { span, .. } => Some(*span),
            Self::TrivialContract { span, .. } => Some(*span),
            Self::SingleArmMatch { span } => Some(*span),
            Self::RedundantCast { span, .. } => Some(*span),
            Self::ConstantCondition { span, .. } => Some(*span),
            Self::SelfComparison { span, .. } => Some(*span),
            Self::RedundantBoolComparison { span, .. } => Some(*span),
            Self::DuplicateMatchArm { span, .. } => Some(*span),
            Self::IntDivisionTruncation { span, .. } => Some(*span),
            Self::UnusedReturnValue { span, .. } => Some(*span),
            Self::NonSnakeCaseFunction { span, .. } => Some(*span),
            Self::NonPascalCaseType { span, .. } => Some(*span),
            Self::IdentityOperation { span, .. } => Some(*span),
            Self::NegatedIfCondition { span } => Some(*span),
            Self::Generic { span, .. } => *span,
        }
    }

    /// Get the message of this warning
    pub fn message(&self) -> String {
        match self {
            Self::UnreachablePattern { message, arm_index, .. } => {
                format!("unreachable pattern (arm {}): {}", arm_index + 1, message)
            }
            Self::UnusedBinding { name, .. } => {
                format!("unused variable: `{}`", name)
            }
            Self::RedundantPattern { message, .. } => {
                format!("redundant pattern: {}", message)
            }
            Self::IntegerRangeOverflow { message, .. } => {
                format!("integer range overflow: {}", message)
            }
            Self::GuardedNonExhaustive { .. } => {
                "match with guards may not be exhaustive; add a wildcard pattern `_ => ...` to ensure all cases are covered".to_string()
            }
            Self::UnusedMut { name, .. } => {
                format!("variable `{}` is declared mutable but never mutated; consider using `let` instead of `let mut`", name)
            }
            Self::UnreachableCode { .. } => {
                "unreachable code; this statement will never be executed".to_string()
            }
            Self::UnusedImport { name, .. } => {
                format!("unused import: `{}`", name)
            }
            Self::UnusedFunction { name, .. } => {
                format!("function `{}` is never used", name)
            }
            Self::UnusedType { name, .. } => {
                format!("type `{}` is never used", name)
            }
            Self::UnusedEnum { name, .. } => {
                format!("enum `{}` is never used", name)
            }
            Self::ShadowBinding { name, .. } => {
                format!("variable `{}` shadows a binding from an outer scope", name)
            }
            Self::UnusedTrait { name, .. } => {
                format!("trait `{}` is never implemented", name)
            }
            Self::DuplicateFunction { name, .. } => {
                format!("function `{}` is defined multiple times; later definition overrides earlier one", name)
            }
            Self::MissingPostcondition { name, .. } => {
                format!("function `{}` has no postcondition", name)
            }
            Self::SemanticDuplication { name, duplicate_of, .. } => {
                format!(
                    "function `{}` has equivalent contract to `{}`; consider consolidating",
                    name, duplicate_of
                )
            }
            Self::TrivialContract { name, contract_kind, .. } => {
                format!(
                    "function `{}`: {} is a tautology (always true); consider adding meaningful constraints",
                    name, contract_kind
                )
            }
            Self::NonSnakeCaseFunction { name, suggestion, .. } => {
                format!("function `{}` should have a snake_case name; consider renaming to `{}`", name, suggestion)
            }
            Self::NonPascalCaseType { name, suggestion, kind, .. } => {
                format!("{} `{}` should have a PascalCase name; consider renaming to `{}`", kind, name, suggestion)
            }
            Self::SingleArmMatch { .. } => {
                "match with a single arm could be simplified to `if let`".to_string()
            }
            Self::RedundantCast { ty, .. } => {
                format!("redundant cast: expression is already of type `{}`", ty)
            }
            Self::ConstantCondition { value, context, .. } => {
                format!("`{}` condition is always `{}`", context, value)
            }
            Self::SelfComparison { name, op, .. } => {
                format!("comparing `{}` {} `{}` is always the same result", name, op, name)
            }
            Self::RedundantBoolComparison { value, .. } => {
                format!("redundant boolean comparison: comparing to `{}` is unnecessary", value)
            }
            Self::DuplicateMatchArm { pattern, .. } => {
                format!("duplicate match arm: pattern `{}` appears more than once", pattern)
            }
            Self::IntDivisionTruncation { left, right, .. } => {
                format!("integer division `{} / {}` truncates to `{}`", left, right, left / right)
            }
            Self::UnusedReturnValue { func, .. } => {
                format!("return value of `{}` is unused", func)
            }
            Self::IdentityOperation { expr, .. } => {
                format!("identity operation `{}` has no effect", expr)
            }
            Self::NegatedIfCondition { .. } => {
                "negated if condition: consider swapping branches to remove negation".to_string()
            }
            Self::Generic { message, .. } => message.clone(),
        }
    }

    /// Get the warning kind as a string
    pub fn kind(&self) -> &'static str {
        match self {
            Self::UnreachablePattern { .. } => "unreachable_pattern",
            Self::UnusedBinding { .. } => "unused_binding",
            Self::RedundantPattern { .. } => "redundant_pattern",
            Self::IntegerRangeOverflow { .. } => "integer_range_overflow",
            Self::GuardedNonExhaustive { .. } => "guarded_non_exhaustive",
            Self::UnusedMut { .. } => "unused_mut",
            Self::UnreachableCode { .. } => "unreachable_code",
            Self::UnusedImport { .. } => "unused_import",
            Self::UnusedFunction { .. } => "unused_function",
            Self::UnusedType { .. } => "unused_type",
            Self::UnusedEnum { .. } => "unused_enum",
            Self::ShadowBinding { .. } => "shadow_binding",
            Self::UnusedTrait { .. } => "unused_trait",
            Self::DuplicateFunction { .. } => "duplicate_function",
            Self::MissingPostcondition { .. } => "missing_postcondition",
            Self::SemanticDuplication { .. } => "semantic_duplication",
            Self::TrivialContract { .. } => "trivial_contract",
            Self::NonSnakeCaseFunction { .. } => "non_snake_case",
            Self::NonPascalCaseType { .. } => "non_pascal_case",
            Self::SingleArmMatch { .. } => "single_arm_match",
            Self::RedundantCast { .. } => "redundant_cast",
            Self::ConstantCondition { .. } => "constant_condition",
            Self::SelfComparison { .. } => "self_comparison",
            Self::RedundantBoolComparison { .. } => "redundant_bool_comparison",
            Self::DuplicateMatchArm { .. } => "duplicate_match_arm",
            Self::IntDivisionTruncation { .. } => "int_division_truncation",
            Self::UnusedReturnValue { .. } => "unused_return_value",
            Self::IdentityOperation { .. } => "identity_operation",
            Self::NegatedIfCondition { .. } => "negated_if_condition",
            Self::Generic { .. } => "warning",
        }
    }
}

impl std::fmt::Display for CompileWarning {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "warning[{}]: {}", self.kind(), self.message())
    }
}

/// Compile error
#[derive(Debug, Error)]
pub enum CompileError {
    #[error("Lexer error at {span:?}: {message}")]
    Lexer { message: String, span: Span },

    #[error("Parser error at {span:?}: {message}")]
    Parser { message: String, span: Span },

    #[error("Type error at {span:?}: {message}")]
    Type { message: String, span: Span },

    /// IO error (v0.5 Phase 7)
    #[error("IO error: {message}")]
    Io { message: String },

    /// Parse error without span (v0.5 Phase 7)
    #[error("Parse error: {message}")]
    Parse { message: String },

    /// Module resolution error (v0.5 Phase 7)
    /// v0.70: Added optional span for better error localization
    #[error("Resolution error{}: {message}", span.map(|s| format!(" at {}", s)).unwrap_or_default())]
    Resolve { message: String, span: Option<Span> },
}

impl CompileError {
    pub fn lexer(message: impl Into<String>, span: Span) -> Self {
        Self::Lexer {
            message: message.into(),
            span,
        }
    }

    pub fn parser(message: impl Into<String>, span: Span) -> Self {
        Self::Parser {
            message: message.into(),
            span,
        }
    }

    pub fn type_error(message: impl Into<String>, span: Span) -> Self {
        Self::Type {
            message: message.into(),
            span,
        }
    }

    /// Create an IO error (v0.5 Phase 7)
    pub fn io_error(message: impl Into<String>) -> Self {
        Self::Io {
            message: message.into(),
        }
    }

    /// Create a parse error without span (v0.5 Phase 7)
    pub fn parse_error(message: impl Into<String>) -> Self {
        Self::Parse {
            message: message.into(),
        }
    }

    /// Create a resolution error without span (v0.5 Phase 7)
    pub fn resolve_error(message: impl Into<String>) -> Self {
        Self::Resolve {
            message: message.into(),
            span: None,
        }
    }

    /// Create a resolution error with span (v0.70)
    pub fn resolve_error_at(message: impl Into<String>, span: Span) -> Self {
        Self::Resolve {
            message: message.into(),
            span: Some(span),
        }
    }

    pub fn span(&self) -> Option<Span> {
        match self {
            Self::Lexer { span, .. } => Some(*span),
            Self::Parser { span, .. } => Some(*span),
            Self::Type { span, .. } => Some(*span),
            Self::Resolve { span, .. } => *span,
            Self::Io { .. } | Self::Parse { .. } => None,
        }
    }

    pub fn message(&self) -> &str {
        match self {
            Self::Lexer { message, .. } => message,
            Self::Parser { message, .. } => message,
            Self::Type { message, .. } => message,
            Self::Io { message, .. } => message,
            Self::Parse { message, .. } => message,
            Self::Resolve { message, .. } => message,
        }
    }
}

/// Report error with ariadne
pub fn report_error(filename: &str, source: &str, error: &CompileError) {
    use ariadne::{Color, Label, Report, ReportKind, Source};

    let kind = match error {
        CompileError::Lexer { .. } => "Lexer",
        CompileError::Parser { .. } => "Parser",
        CompileError::Type { .. } => "Type",
        CompileError::Io { .. } => "IO",
        CompileError::Parse { .. } => "Parse",
        CompileError::Resolve { .. } => "Resolve",
    };

    if let Some(span) = error.span() {
        Report::build(ReportKind::Error, (filename, span.start..span.end))
            .with_message(format!("{kind} error"))
            .with_label(
                Label::new((filename, span.start..span.end))
                    .with_message(error.message())
                    .with_color(Color::Red),
            )
            .finish()
            .print((filename, Source::from(source)))
            .unwrap();
    } else {
        // Errors without span (IO, Parse, Resolve)
        Report::build(ReportKind::Error, (filename, 0..0))
            .with_message(format!("{kind} error: {}", error.message()))
            .finish()
            .print((filename, Source::from(source)))
            .unwrap();
    }
}

/// Report warning with ariadne (v0.47)
/// P0 Correctness: Visual feedback for potential issues without blocking compilation
pub fn report_warning(filename: &str, source: &str, warning: &CompileWarning) {
    use ariadne::{Color, Label, Report, ReportKind, Source};

    if let Some(span) = warning.span() {
        Report::build(ReportKind::Warning, (filename, span.start..span.end))
            .with_message(format!("warning[{}]", warning.kind()))
            .with_label(
                Label::new((filename, span.start..span.end))
                    .with_message(warning.message())
                    .with_color(Color::Yellow),
            )
            .finish()
            .print((filename, Source::from(source)))
            .unwrap();
    } else {
        // Warnings without span
        Report::build(ReportKind::Warning, (filename, 0..0))
            .with_message(warning.message())
            .finish()
            .print((filename, Source::from(source)))
            .unwrap();
    }
}

/// Report multiple warnings (v0.47)
pub fn report_warnings(filename: &str, source: &str, warnings: &[CompileWarning]) {
    for warning in warnings {
        report_warning(filename, source, warning);
    }
}

// ============================================================================
// v0.71: Machine-readable output (AI-friendly)
// ============================================================================

/// Machine-readable error output (JSON format)
pub fn report_error_machine(filename: &str, _source: &str, error: &CompileError) {
    let kind = match error {
        CompileError::Lexer { .. } => "lexer",
        CompileError::Parser { .. } => "parser",
        CompileError::Type { .. } => "type",
        CompileError::Io { .. } => "io",
        CompileError::Parse { .. } => "parse",
        CompileError::Resolve { .. } => "resolve",
    };

    let (start, end) = error.span().map(|s| (s.start, s.end)).unwrap_or((0, 0));

    println!(
        r#"{{"type":"error","kind":"{}","file":"{}","start":{},"end":{},"message":"{}"}}"#,
        kind,
        filename.replace('\\', "\\\\").replace('"', "\\\""),
        start,
        end,
        error.message().replace('\\', "\\\\").replace('"', "\\\"").replace('\n', "\\n")
    );
}

/// Machine-readable warning output (JSON format)
pub fn report_warning_machine(filename: &str, _source: &str, warning: &CompileWarning) {
    let (start, end) = warning.span().map(|s| (s.start, s.end)).unwrap_or((0, 0));

    println!(
        r#"{{"type":"warning","kind":"{}","file":"{}","start":{},"end":{},"message":"{}"}}"#,
        warning.kind(),
        filename.replace('\\', "\\\\").replace('"', "\\\""),
        start,
        end,
        warning.message().replace('\\', "\\\\").replace('"', "\\\"").replace('\n', "\\n")
    );
}

/// Machine-readable warnings output
pub fn report_warnings_machine(filename: &str, source: &str, warnings: &[CompileWarning]) {
    for warning in warnings {
        report_warning_machine(filename, source, warning);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::Span;

    fn span() -> Span { Span::new(10, 20) }

    // ================================================================
    // CompileWarning Constructor Tests
    // ================================================================

    #[test]
    fn test_warning_unused_binding() {
        let w = CompileWarning::unused_binding("x", span());
        assert_eq!(w.kind(), "unused_binding");
        assert!(w.message().contains("`x`"));
        assert_eq!(w.span(), Some(span()));
    }

    #[test]
    fn test_warning_unreachable_pattern() {
        let w = CompileWarning::unreachable_pattern("already matched", span(), 2);
        assert_eq!(w.kind(), "unreachable_pattern");
        assert!(w.message().contains("arm 3"));
        assert!(w.message().contains("already matched"));
    }

    #[test]
    fn test_warning_redundant_pattern() {
        let w = CompileWarning::redundant_pattern("covered by wildcard", span());
        assert_eq!(w.kind(), "redundant_pattern");
        assert!(w.message().contains("covered by wildcard"));
    }

    #[test]
    fn test_warning_integer_range_overflow() {
        let w = CompileWarning::integer_range_overflow("i64 overflow", span());
        assert_eq!(w.kind(), "integer_range_overflow");
        assert!(w.message().contains("i64 overflow"));
    }

    #[test]
    fn test_warning_guarded_non_exhaustive() {
        let w = CompileWarning::guarded_non_exhaustive(span());
        assert_eq!(w.kind(), "guarded_non_exhaustive");
        assert!(w.message().contains("wildcard"));
    }

    #[test]
    fn test_warning_unused_mut() {
        let w = CompileWarning::unused_mut("counter", span());
        assert_eq!(w.kind(), "unused_mut");
        assert!(w.message().contains("`counter`"));
        assert!(w.message().contains("let"));
    }

    #[test]
    fn test_warning_unreachable_code() {
        let w = CompileWarning::unreachable_code(span());
        assert_eq!(w.kind(), "unreachable_code");
        assert!(w.message().contains("never be executed"));
    }

    #[test]
    fn test_warning_unused_import() {
        let w = CompileWarning::unused_import("std::io", span());
        assert_eq!(w.kind(), "unused_import");
        assert!(w.message().contains("`std::io`"));
    }

    #[test]
    fn test_warning_unused_function() {
        let w = CompileWarning::unused_function("helper", span());
        assert_eq!(w.kind(), "unused_function");
        assert!(w.message().contains("`helper`"));
    }

    #[test]
    fn test_warning_unused_type() {
        let w = CompileWarning::unused_type("Config", span());
        assert_eq!(w.kind(), "unused_type");
        assert!(w.message().contains("`Config`"));
    }

    #[test]
    fn test_warning_unused_enum() {
        let w = CompileWarning::unused_enum("Color", span());
        assert_eq!(w.kind(), "unused_enum");
        assert!(w.message().contains("`Color`"));
    }

    #[test]
    fn test_warning_shadow_binding() {
        let orig = Span::new(0, 5);
        let w = CompileWarning::shadow_binding("x", span(), orig);
        assert_eq!(w.kind(), "shadow_binding");
        assert!(w.message().contains("`x`"));
        assert!(w.message().contains("shadows"));
    }

    #[test]
    fn test_warning_unused_trait() {
        let w = CompileWarning::unused_trait("Drawable", span());
        assert_eq!(w.kind(), "unused_trait");
        assert!(w.message().contains("`Drawable`"));
    }

    #[test]
    fn test_warning_duplicate_function() {
        let orig = Span::new(0, 5);
        let w = CompileWarning::duplicate_function("main", span(), orig);
        assert_eq!(w.kind(), "duplicate_function");
        assert!(w.message().contains("`main`"));
    }

    #[test]
    fn test_warning_missing_postcondition() {
        let w = CompileWarning::missing_postcondition("compute", span());
        assert_eq!(w.kind(), "missing_postcondition");
        assert!(w.message().contains("`compute`"));
    }

    #[test]
    fn test_warning_semantic_duplication() {
        let w = CompileWarning::semantic_duplication("foo", "bar", span());
        assert_eq!(w.kind(), "semantic_duplication");
        assert!(w.message().contains("`foo`"));
        assert!(w.message().contains("`bar`"));
    }

    #[test]
    fn test_warning_trivial_contract() {
        let w = CompileWarning::trivial_contract("f", "precondition", span());
        assert_eq!(w.kind(), "trivial_contract");
        assert!(w.message().contains("tautology"));
    }

    #[test]
    fn test_warning_generic() {
        let w = CompileWarning::generic("custom message", Some(span()));
        assert_eq!(w.kind(), "warning");
        assert_eq!(w.message(), "custom message");
        assert_eq!(w.span(), Some(span()));
    }

    #[test]
    fn test_warning_generic_no_span() {
        let w = CompileWarning::generic("no span", None);
        assert_eq!(w.span(), None);
    }

    #[test]
    fn test_warning_display() {
        let w = CompileWarning::unused_binding("x", span());
        let display = format!("{}", w);
        assert!(display.starts_with("warning[unused_binding]:"));
        assert!(display.contains("`x`"));
    }

    // ================================================================
    // CompileError Tests
    // ================================================================

    #[test]
    fn test_error_lexer() {
        let e = CompileError::lexer("unexpected char", span());
        assert_eq!(e.message(), "unexpected char");
        assert_eq!(e.span(), Some(span()));
        let display = format!("{}", e);
        assert!(display.contains("Lexer error"));
    }

    #[test]
    fn test_error_parser() {
        let e = CompileError::parser("expected ')'", span());
        assert_eq!(e.message(), "expected ')'");
        assert_eq!(e.span(), Some(span()));
    }

    #[test]
    fn test_error_type() {
        let e = CompileError::type_error("type mismatch", span());
        assert_eq!(e.message(), "type mismatch");
        assert_eq!(e.span(), Some(span()));
    }

    #[test]
    fn test_error_io() {
        let e = CompileError::io_error("file not found");
        assert_eq!(e.message(), "file not found");
        assert_eq!(e.span(), None);
    }

    #[test]
    fn test_error_parse() {
        let e = CompileError::parse_error("invalid syntax");
        assert_eq!(e.message(), "invalid syntax");
        assert_eq!(e.span(), None);
    }

    #[test]
    fn test_error_resolve() {
        let e = CompileError::resolve_error("module not found");
        assert_eq!(e.message(), "module not found");
        assert_eq!(e.span(), None);
    }

    #[test]
    fn test_error_resolve_at() {
        let e = CompileError::resolve_error_at("ambiguous import", span());
        assert_eq!(e.message(), "ambiguous import");
        assert_eq!(e.span(), Some(span()));
    }

    // ================================================================
    // Cycles 119-120: Additional Error Module Tests
    // ================================================================

    #[test]
    fn test_error_display_format_lexer() {
        let e = CompileError::lexer("unexpected '$'", span());
        let display = format!("{}", e);
        assert!(display.contains("Lexer error"));
        assert!(display.contains("unexpected '$'"));
    }

    #[test]
    fn test_error_display_format_parser() {
        let e = CompileError::parser("expected ';' after expression", span());
        let display = format!("{}", e);
        assert!(display.contains("Parser error"));
        assert!(display.contains("expected ';'"));
    }

    #[test]
    fn test_error_display_format_type() {
        let e = CompileError::type_error("cannot unify i64 with bool", span());
        let display = format!("{}", e);
        assert!(display.contains("Type error"));
        assert!(display.contains("cannot unify"));
    }

    #[test]
    fn test_error_display_format_io() {
        let e = CompileError::io_error("permission denied");
        let display = format!("{}", e);
        assert!(display.contains("IO error"));
        assert!(display.contains("permission denied"));
    }

    #[test]
    fn test_error_display_format_resolve_with_span() {
        let e = CompileError::resolve_error_at("circular dependency", Span::new(5, 15));
        let display = format!("{}", e);
        assert!(display.contains("Resolution error"));
        assert!(display.contains("circular dependency"));
    }

    #[test]
    fn test_error_display_format_resolve_without_span() {
        let e = CompileError::resolve_error("module 'math' not found");
        let display = format!("{}", e);
        assert!(display.contains("Resolution error"));
        assert!(display.contains("module 'math' not found"));
    }

    #[test]
    fn test_warning_display_format_all_kinds() {
        // Verify that Display works correctly for each warning kind,
        // specifically the "warning[kind]: message" format.
        let cases: Vec<Box<dyn Fn() -> CompileWarning>> = vec![
            Box::new(|| CompileWarning::unused_mut("counter", span())),
            Box::new(|| CompileWarning::unreachable_code(span())),
            Box::new(|| CompileWarning::unused_import("io", span())),
            Box::new(|| CompileWarning::unused_function("helper", span())),
            Box::new(|| CompileWarning::unused_type("Config", span())),
            Box::new(|| CompileWarning::unused_enum("Color", span())),
            Box::new(|| CompileWarning::unused_trait("Show", span())),
            Box::new(|| CompileWarning::missing_postcondition("compute", span())),
        ];
        for factory in &cases {
            let w = factory();
            let display = format!("{}", w);
            // All warnings should start with "warning[<kind>]:"
            assert!(display.starts_with("warning["), "Display format wrong for kind: {}", w.kind());
            assert!(display.contains("]:"), "Missing colon separator for kind: {}", w.kind());
        }
    }

    #[test]
    fn test_warning_semantic_duplication_display() {
        let w = CompileWarning::semantic_duplication("add", "plus", span());
        let display = format!("{}", w);
        assert!(display.contains("warning[semantic_duplication]"));
        assert!(display.contains("`add`"));
        assert!(display.contains("`plus`"));
        assert!(display.contains("consolidating"));
    }

    #[test]
    fn test_warning_trivial_contract_display() {
        let w = CompileWarning::trivial_contract("check", "postcondition", span());
        let display = format!("{}", w);
        assert!(display.contains("warning[trivial_contract]"));
        assert!(display.contains("`check`"));
        assert!(display.contains("tautology"));
        assert!(display.contains("postcondition"));
    }

    #[test]
    fn test_warning_duplicate_function_message_content() {
        let w = CompileWarning::duplicate_function("init", span(), Span::new(0, 5));
        let msg = w.message();
        assert!(msg.contains("`init`"));
        assert!(msg.contains("defined multiple times"));
        assert!(msg.contains("overrides"));
    }

    #[test]
    fn test_warning_shadow_binding_message_content() {
        let w = CompileWarning::shadow_binding("result", span(), Span::new(0, 5));
        let msg = w.message();
        assert!(msg.contains("`result`"));
        assert!(msg.contains("shadows"));
        assert!(msg.contains("outer scope"));
    }

    #[test]
    fn test_error_debug_format() {
        // Verify that Debug trait is implemented and produces useful output
        let e = CompileError::type_error("mismatch", span());
        let dbg = format!("{:?}", e);
        assert!(dbg.contains("Type"));
        assert!(dbg.contains("mismatch"));
    }
}
