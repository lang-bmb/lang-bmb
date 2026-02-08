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

    // ================================================================
    // Cycles 119-120: Additional Lexer Tokenization Tests
    // ================================================================

    #[test]
    fn test_tokenize_whitespace_only() {
        let tokens = tokenize("   \t\t\n\n\r\n   ").unwrap();
        assert!(tokens.is_empty(), "whitespace-only source should produce no tokens");
    }

    #[test]
    fn test_tokenize_unexpected_character_error() {
        // The backtick is not a valid BMB token
        let result = tokenize("`");
        assert!(result.is_err(), "unexpected character should produce an error");
        let err = result.unwrap_err();
        assert!(err.message().contains("unexpected character"));
    }

    #[test]
    fn test_tokenize_very_long_identifier() {
        let long_name = "a".repeat(500);
        let tokens = tokenize(&long_name).unwrap();
        assert_eq!(tokens.len(), 1);
        assert!(matches!(&tokens[0].0, Token::Ident(s) if s.len() == 500));
    }

    #[test]
    fn test_tokenize_type_keywords() {
        let tokens = tokenize("i32 i64 u32 u64 f64 bool String char").unwrap();
        let kinds: Vec<_> = tokens.iter().map(|(t, _)| t.clone()).collect();
        assert_eq!(kinds, vec![
            Token::TyI32, Token::TyI64, Token::TyU32, Token::TyU64,
            Token::TyF64, Token::TyBool, Token::TyString, Token::TyChar,
        ]);
    }

    #[test]
    fn test_tokenize_bitwise_operators() {
        let tokens = tokenize("band bor bxor bnot").unwrap();
        let kinds: Vec<_> = tokens.iter().map(|(t, _)| t.clone()).collect();
        assert_eq!(kinds, vec![Token::Band, Token::Bor, Token::Bxor, Token::Bnot]);
    }

    #[test]
    fn test_tokenize_shift_operators() {
        let tokens = tokenize("<< >>").unwrap();
        let kinds: Vec<_> = tokens.iter().map(|(t, _)| t.clone()).collect();
        assert_eq!(kinds, vec![Token::LtLt, Token::GtGt]);
    }

    #[test]
    fn test_tokenize_wrapping_checked_saturating_operators() {
        let tokens = tokenize("+% -% *% +? -? *? +| -| *|").unwrap();
        let kinds: Vec<_> = tokens.iter().map(|(t, _)| t.clone()).collect();
        assert_eq!(kinds, vec![
            Token::PlusPercent, Token::MinusPercent, Token::StarPercent,
            Token::PlusQuestion, Token::MinusQuestion, Token::StarQuestion,
            Token::PlusPipe, Token::MinusPipe, Token::StarPipe,
        ]);
    }

    #[test]
    fn test_tokenize_symbolic_logical_operators() {
        let tokens = tokenize("&& || !").unwrap();
        let kinds: Vec<_> = tokens.iter().map(|(t, _)| t.clone()).collect();
        assert_eq!(kinds, vec![Token::AmpAmp, Token::PipePipe, Token::Bang]);
    }

    #[test]
    fn test_tokenize_reference_and_special_symbols() {
        let tokens = tokenize("& @ ? |").unwrap();
        let kinds: Vec<_> = tokens.iter().map(|(t, _)| t.clone()).collect();
        assert_eq!(kinds, vec![Token::Ampersand, Token::At, Token::Question, Token::Pipe]);
    }

    #[test]
    fn test_tokenize_range_operators() {
        let tokens = tokenize(".. ..< ..=").unwrap();
        let kinds: Vec<_> = tokens.iter().map(|(t, _)| t.clone()).collect();
        assert_eq!(kinds, vec![Token::DotDot, Token::DotDotLt, Token::DotDotEq]);
    }

    #[test]
    fn test_tokenize_string_with_escapes() {
        let tokens = tokenize(r#""\n\t\r\\\"\0""#).unwrap();
        assert_eq!(tokens.len(), 1);
        match &tokens[0].0 {
            Token::StringLit(s) => {
                assert_eq!(s, "\n\t\r\\\"\0");
            }
            other => panic!("expected StringLit, got {:?}", other),
        }
    }

    #[test]
    fn test_tokenize_char_literal() {
        let tokens = tokenize("'x'").unwrap();
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].0, Token::CharLit('x'));
    }

    #[test]
    fn test_tokenize_scientific_notation_float() {
        let tokens = tokenize("3.14e10 1e5 6.022E23").unwrap();
        assert_eq!(tokens.len(), 3);
        assert!(matches!(&tokens[0].0, Token::FloatLit(_)));
        assert!(matches!(&tokens[1].0, Token::FloatLit(_)));
        assert!(matches!(&tokens[2].0, Token::FloatLit(_)));
    }

    #[test]
    fn test_tokenize_concurrency_keywords() {
        let tokens = tokenize("spawn async await Future select").unwrap();
        let kinds: Vec<_> = tokens.iter().map(|(t, _)| t.clone()).collect();
        assert_eq!(kinds, vec![
            Token::Spawn, Token::Async, Token::Await, Token::FutureType, Token::Select,
        ]);
    }

    #[test]
    fn test_tokenize_contract_keywords() {
        let tokens = tokenize("pre post invariant implies forall exists where it").unwrap();
        let kinds: Vec<_> = tokens.iter().map(|(t, _)| t.clone()).collect();
        assert_eq!(kinds, vec![
            Token::Pre, Token::Post, Token::Invariant, Token::Implies,
            Token::Forall, Token::Exists, Token::Where, Token::It,
        ]);
    }

    #[test]
    fn test_tokenize_module_and_visibility_keywords() {
        let tokens = tokenize("pub mod use extern module").unwrap();
        let kinds: Vec<_> = tokens.iter().map(|(t, _)| t.clone()).collect();
        assert_eq!(kinds, vec![
            Token::Pub, Token::Mod, Token::Use, Token::Extern, Token::Module,
        ]);
    }

    #[test]
    fn test_tokenize_control_flow_keywords() {
        let tokens = tokenize("while for in loop break continue return").unwrap();
        let kinds: Vec<_> = tokens.iter().map(|(t, _)| t.clone()).collect();
        assert_eq!(kinds, vec![
            Token::While, Token::For, Token::In, Token::Loop,
            Token::Break, Token::Continue, Token::Return,
        ]);
    }

    #[test]
    fn test_tokenize_header_separator() {
        let tokens = tokenize("===").unwrap();
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].0, Token::HeaderSep);
    }

    #[test]
    fn test_tokenize_dash_comment_preserves_tokens() {
        let tokens = tokenize("42 -- this is ignored\n99").unwrap();
        assert_eq!(tokens.len(), 2);
        assert!(matches!(&tokens[0].0, Token::IntLit(42)));
        assert!(matches!(&tokens[1].0, Token::IntLit(99)));
    }

    #[test]
    fn test_tokenize_adjacent_symbols_disambiguation() {
        // Ensure multi-char operators are correctly distinguished from their prefixes
        let tokens = tokenize(":: : := . .. ..< ..=").unwrap();
        let kinds: Vec<_> = tokens.iter().map(|(t, _)| t.clone()).collect();
        assert_eq!(kinds, vec![
            Token::ColonColon, Token::Colon, Token::ColonEq,
            Token::Dot, Token::DotDot, Token::DotDotLt, Token::DotDotEq,
        ]);
    }

    #[test]
    fn test_tokenize_negative_integer_as_minus_then_int() {
        // Lexer should produce Minus + IntLit, not a negative IntLit
        let tokens = tokenize("-42").unwrap();
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].0, Token::Minus);
        assert!(matches!(&tokens[1].0, Token::IntLit(42)));
    }
}
