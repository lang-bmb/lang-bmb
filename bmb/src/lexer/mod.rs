//! Lexer implementation using logos

mod token;

pub use token::Token;

use crate::ast::Span;
use crate::error::{CompileError, Result};
use logos::Logos;

/// Tokenize source code
pub fn tokenize(source: &str) -> Result<Vec<(Token, Span)>> {
    let mut tokens = Vec::new();
    let mut lexer = Token::lexer(source);

    while let Some(result) = lexer.next() {
        let span = Span::new(lexer.span().start, lexer.span().end);
        match result {
            Ok(token) => tokens.push((token, span)),
            Err(_) => {
                return Err(CompileError::lexer(
                    format!("unexpected character: {:?}", lexer.slice()),
                    span,
                ));
            }
        }
    }

    Ok(tokens)
}
