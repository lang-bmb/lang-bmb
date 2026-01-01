//! Error types and reporting

use crate::ast::Span;
use thiserror::Error;

/// Result type alias
pub type Result<T> = std::result::Result<T, CompileError>;

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
    #[error("Resolution error: {message}")]
    Resolve { message: String },
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

    /// Create a resolution error (v0.5 Phase 7)
    pub fn resolve_error(message: impl Into<String>) -> Self {
        Self::Resolve {
            message: message.into(),
        }
    }

    pub fn span(&self) -> Option<Span> {
        match self {
            Self::Lexer { span, .. } => Some(*span),
            Self::Parser { span, .. } => Some(*span),
            Self::Type { span, .. } => Some(*span),
            Self::Io { .. } | Self::Parse { .. } | Self::Resolve { .. } => None,
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
