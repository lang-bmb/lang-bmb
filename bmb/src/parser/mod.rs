//! Parser implementation using lalrpop

use crate::ast::{Program, Span};
use crate::error::{CompileError, Result};
use crate::lexer::Token;

#[cfg(test)]
mod tests;

lalrpop_util::lalrpop_mod!(
    #[allow(clippy::all)]
    grammar
);

/// Parse tokens into AST
pub fn parse(_filename: &str, _source: &str, tokens: Vec<(Token, Span)>) -> Result<Program> {
    let token_iter = tokens
        .into_iter()
        .map(|(tok, span)| (span.start, tok, span.end));

    grammar::ProgramParser::new()
        .parse(token_iter)
        .map_err(|e| {
            let span = match &e {
                lalrpop_util::ParseError::InvalidToken { location } => Span::new(*location, *location + 1),
                lalrpop_util::ParseError::UnrecognizedEof { location, .. } => {
                    Span::new(*location, *location + 1)
                }
                lalrpop_util::ParseError::UnrecognizedToken { token, .. } => {
                    Span::new(token.0, token.2)
                }
                lalrpop_util::ParseError::ExtraToken { token } => Span::new(token.0, token.2),
                lalrpop_util::ParseError::User { .. } => Span::new(0, 1),
            };
            CompileError::parser(format!("{e}"), span)
        })
}
