//! Tokenizer. Comments are `//` only and are discarded.

use crate::diag::{Diag, Diags};
use crate::source::Span;

#[derive(Debug, Clone, PartialEq)]
pub enum Tok {
    Ident(String),
    Int(u128),
    Str(String),
    P(&'static str),
    Eof,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub tok: Tok,
    pub span: Span,
}

const PUNCT3: &[&str] = &["==>", "..="];
const PUNCT2: &[&str] = &["::", "->", "=>", "==", "!=", "<=", ">=", "&&", "||", ".."];
const PUNCT1: &[&str] = &[
    "(", ")", "{", "}", "[", "]", ",", ";", ":", ".", "|", "#", "!", "&", "=", "<", ">", "+", "-",
    "*", "/", "%", "?", "@", "^", "~", "$",
];

pub fn lex(file: u32, text: &str, diags: &mut Diags) -> Vec<Token> {
    let b = text.as_bytes();
    let mut i = 0usize;
    let mut out = Vec::new();
    let sp = |lo: usize, hi: usize| Span {
        file,
        lo: lo as u32,
        hi: hi as u32,
    };
    'outer: while i < b.len() {
        let c = b[i];
        if c.is_ascii_whitespace() {
            i += 1;
            continue;
        }
        if c == b'/' && b.get(i + 1) == Some(&b'/') {
            while i < b.len() && b[i] != b'\n' {
                i += 1;
            }
            continue;
        }
        if c == b'/' && b.get(i + 1) == Some(&b'*') {
            let lo = i;
            i += 2;
            while i < b.len() && !(b[i] == b'*' && b.get(i + 1) == Some(&b'/')) {
                i += 1;
            }
            i = (i + 2).min(b.len());
            diags.push(
                Diag::new("E0106", sp(lo, i), "block comments are not allowed")
                    .fix("use `//` line comments", None),
            );
            continue;
        }
        if c.is_ascii_alphabetic() || c == b'_' {
            let lo = i;
            while i < b.len() && (b[i].is_ascii_alphanumeric() || b[i] == b'_') {
                i += 1;
            }
            out.push(Token {
                tok: Tok::Ident(text[lo..i].to_string()),
                span: sp(lo, i),
            });
            continue;
        }
        if c.is_ascii_digit() {
            let lo = i;
            let mut v: u128 = 0;
            let mut overflow = false;
            while i < b.len() && (b[i].is_ascii_digit() || b[i] == b'_') {
                if b[i] != b'_' {
                    match v
                        .checked_mul(10)
                        .and_then(|x| x.checked_add((b[i] - b'0') as u128))
                    {
                        Some(x) => v = x,
                        None => overflow = true,
                    }
                }
                i += 1;
            }
            let digits_end = i;
            if i < b.len() && (b[i].is_ascii_alphabetic() || b[i] == b'_') {
                let slo = i;
                while i < b.len() && (b[i].is_ascii_alphanumeric() || b[i] == b'_') {
                    i += 1;
                }
                diags.push(
                    Diag::new(
                        "E0104",
                        sp(lo, i),
                        format!("literal suffix `{}` is not allowed", &text[slo..i]),
                    )
                    .fix("remove the suffix", Some(text[lo..digits_end].to_string())),
                );
            }
            if overflow {
                diags.push(Diag::new(
                    "E0105",
                    sp(lo, i),
                    "integer literal does not fit in 128 bits",
                ));
            }
            out.push(Token {
                tok: Tok::Int(v),
                span: sp(lo, i),
            });
            continue;
        }
        if c == b'"' {
            let lo = i;
            i += 1;
            let mut s = String::new();
            while i < b.len() && b[i] != b'"' {
                if b[i] == b'\\' && i + 1 < b.len() {
                    match b[i + 1] {
                        b'n' => s.push('\n'),
                        b't' => s.push('\t'),
                        b'"' => s.push('"'),
                        b'\\' => s.push('\\'),
                        _ => s.push(b[i + 1] as char),
                    }
                    i += 2;
                    continue;
                }
                let ch = text[i..].chars().next().unwrap();
                s.push(ch);
                i += ch.len_utf8();
            }
            if i >= b.len() {
                diags.push(Diag::new("E0102", sp(lo, i), "unterminated string literal"));
                break;
            }
            i += 1;
            out.push(Token {
                tok: Tok::Str(s),
                span: sp(lo, i),
            });
            continue;
        }
        if c == b'\'' {
            let lo = i;
            i += 1;
            while i < b.len() && (b[i].is_ascii_alphanumeric() || b[i] == b'_') {
                i += 1;
            }
            if i < b.len() && b[i] == b'\'' {
                i += 1;
                diags.push(Diag::new(
                    "E0101",
                    sp(lo, i),
                    "character literals are not supported",
                ));
            } else {
                diags.push(
                    Diag::new("E0208", sp(lo, i), "lifetime annotations are not allowed").fix(
                        "remove the lifetime; references are second-class and need none",
                        Some(String::new()),
                    ),
                );
            }
            continue;
        }
        for set in [PUNCT3, PUNCT2, PUNCT1] {
            for p in set {
                if text[i..].starts_with(p) {
                    out.push(Token {
                        tok: Tok::P(p),
                        span: sp(i, i + p.len()),
                    });
                    i += p.len();
                    continue 'outer;
                }
            }
        }
        let ch = text[i..].chars().next().unwrap();
        diags.push(Diag::new(
            "E0101",
            sp(i, i + ch.len_utf8()),
            format!("unexpected character `{ch}`"),
        ));
        i += ch.len_utf8();
    }
    out.push(Token {
        tok: Tok::Eof,
        span: sp(b.len(), b.len()),
    });
    out
}
