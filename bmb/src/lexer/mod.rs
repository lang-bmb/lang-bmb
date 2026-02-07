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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize_empty() {
        let tokens = tokenize("").unwrap();
        assert!(tokens.is_empty());
    }

    #[test]
    fn test_tokenize_keywords() {
        let tokens = tokenize("fn let if else true false").unwrap();
        let kinds: Vec<_> = tokens.iter().map(|(t, _)| t.clone()).collect();
        assert_eq!(kinds, vec![Token::Fn, Token::Let, Token::If, Token::Else, Token::True, Token::False]);
    }

    #[test]
    fn test_tokenize_integer_literal() {
        let tokens = tokenize("42").unwrap();
        assert_eq!(tokens.len(), 1);
        assert!(matches!(&tokens[0].0, Token::IntLit(n) if *n == 42));
    }

    #[test]
    fn test_tokenize_float_literal() {
        let tokens = tokenize("1.5").unwrap();
        assert_eq!(tokens.len(), 1);
        assert!(matches!(&tokens[0].0, Token::FloatLit(n) if (*n - 1.5).abs() < f64::EPSILON));
    }

    #[test]
    fn test_tokenize_string_literal() {
        let tokens = tokenize(r#""hello world""#).unwrap();
        assert_eq!(tokens.len(), 1);
        assert!(matches!(&tokens[0].0, Token::StringLit(_)));
    }

    #[test]
    fn test_tokenize_operators() {
        let tokens = tokenize("+ - * / %").unwrap();
        let kinds: Vec<_> = tokens.iter().map(|(t, _)| t.clone()).collect();
        assert_eq!(kinds, vec![Token::Plus, Token::Minus, Token::Star, Token::Slash, Token::Percent]);
    }

    #[test]
    fn test_tokenize_comparison_operators() {
        let tokens = tokenize("== != < > <= >=").unwrap();
        let kinds: Vec<_> = tokens.iter().map(|(t, _)| t.clone()).collect();
        assert_eq!(kinds, vec![Token::EqEq, Token::NotEq, Token::Lt, Token::Gt, Token::LtEq, Token::GtEq]);
    }

    #[test]
    fn test_tokenize_delimiters() {
        let tokens = tokenize("( ) { } [ ]").unwrap();
        let kinds: Vec<_> = tokens.iter().map(|(t, _)| t.clone()).collect();
        assert_eq!(kinds, vec![Token::LParen, Token::RParen, Token::LBrace, Token::RBrace, Token::LBracket, Token::RBracket]);
    }

    #[test]
    fn test_tokenize_punctuation() {
        let tokens = tokenize(", ; : . -> =>").unwrap();
        let kinds: Vec<_> = tokens.iter().map(|(t, _)| t.clone()).collect();
        assert_eq!(kinds, vec![Token::Comma, Token::Semi, Token::Colon, Token::Dot, Token::Arrow, Token::FatArrow]);
    }

    #[test]
    fn test_tokenize_identifier() {
        let tokens = tokenize("foo bar_baz x123").unwrap();
        assert_eq!(tokens.len(), 3);
        assert!(matches!(&tokens[0].0, Token::Ident(s) if s == "foo"));
        assert!(matches!(&tokens[1].0, Token::Ident(s) if s == "bar_baz"));
        assert!(matches!(&tokens[2].0, Token::Ident(s) if s == "x123"));
    }

    #[test]
    fn test_tokenize_spans() {
        let tokens = tokenize("fn main").unwrap();
        assert_eq!(tokens[0].1, Span::new(0, 2)); // "fn" at 0..2
        assert_eq!(tokens[1].1, Span::new(3, 7)); // "main" at 3..7
    }

    #[test]
    fn test_tokenize_skips_comments() {
        let tokens = tokenize("fn // this is a comment\nmain").unwrap();
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].0, Token::Fn);
        assert!(matches!(&tokens[1].0, Token::Ident(s) if s == "main"));
    }

    #[test]
    fn test_tokenize_skips_dash_comments() {
        let tokens = tokenize("fn -- this is a comment\nmain").unwrap();
        assert_eq!(tokens.len(), 2);
    }

    #[test]
    fn test_tokenize_skips_whitespace() {
        let tokens = tokenize("  fn  \t\n  main  ").unwrap();
        assert_eq!(tokens.len(), 2);
    }

    #[test]
    fn test_tokenize_simple_function() {
        let tokens = tokenize("fn add(a: i64, b: i64) -> i64 = a + b;").unwrap();
        assert!(tokens.len() > 10);
        assert_eq!(tokens[0].0, Token::Fn);
    }

    #[test]
    fn test_tokenize_logical_operators() {
        let tokens = tokenize("and or not").unwrap();
        let kinds: Vec<_> = tokens.iter().map(|(t, _)| t.clone()).collect();
        assert_eq!(kinds, vec![Token::And, Token::Or, Token::Not]);
    }

    #[test]
    fn test_tokenize_struct_enum_match() {
        let tokens = tokenize("struct enum match").unwrap();
        let kinds: Vec<_> = tokens.iter().map(|(t, _)| t.clone()).collect();
        assert_eq!(kinds, vec![Token::Struct, Token::Enum, Token::Match]);
    }

    #[test]
    fn test_tokenize_assignment() {
        let tokens = tokenize("= :=").unwrap();
        assert_eq!(tokens[0].0, Token::Eq);
        assert_eq!(tokens[1].0, Token::ColonEq);
    }
}
