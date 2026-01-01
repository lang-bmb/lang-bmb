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

    pub fn span(&self) -> Span {
        match self {
            Self::Lexer { span, .. } => *span,
            Self::Parser { span, .. } => *span,
            Self::Type { span, .. } => *span,
        }
    }

    pub fn message(&self) -> &str {
        match self {
            Self::Lexer { message, .. } => message,
            Self::Parser { message, .. } => message,
            Self::Type { message, .. } => message,
        }
    }
}

/// Report error with ariadne
pub fn report_error(filename: &str, source: &str, error: &CompileError) {
    use ariadne::{Color, Label, Report, ReportKind, Source};

    let span = error.span();
    let kind = match error {
        CompileError::Lexer { .. } => "Lexer",
        CompileError::Parser { .. } => "Parser",
        CompileError::Type { .. } => "Type",
    };

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
}
