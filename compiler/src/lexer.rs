//! Tokenizer. Comments are `//` only and are discarded.

use crate::ast::Radix;
use crate::diag::{Diag, Diags};
use crate::source::Span;

#[derive(Debug, Clone, PartialEq)]
pub enum Tok {
    Ident(String),
    Int(u128, Radix),
    Str(String),
    /// `b'a'` (ADR-0044 §2).
    Byte(u8),
    /// `b"..."` (ADR-0044 §2).
    Bytes(Vec<u8>),
    P(&'static str),
    /// A loop label `'outer` (ADR-0049 §1): before `: while`/`: loop`/
    /// `: for`, or after `break`/`continue`. Elsewhere `'a` is a lifetime
    /// (E0208).
    Label(String),
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
        if c == b'b' && matches!(b.get(i + 1), Some(b'\'' | b'"')) {
            let lo = i;
            let quote = b[i + 1];
            i += 2;
            let mut bytes = Vec::new();
            let mut non_ascii = false;
            while i < b.len() && b[i] != quote && !(quote == b'\'' && b[i] == b'\n') {
                if b[i] == b'\\' {
                    match escape(b, i) {
                        Some((v, n)) => {
                            bytes.push(v);
                            i += n;
                        }
                        None => {
                            let n = 1 + text[i + 1..].chars().next().map_or(0, char::len_utf8);
                            diags.push(Diag::new(
                                "E0101",
                                sp(i, i + n),
                                "unknown escape: byte literals take \\n \\r \\t \\\\ \\' \\\" \\0 \\xHH",
                            ));
                            i += n;
                        }
                    }
                    continue;
                }
                non_ascii |= b[i] >= 0x80;
                bytes.push(b[i]);
                i += 1;
            }
            if i >= b.len() || b[i] != quote {
                diags.push(Diag::new("E0102", sp(lo, i), "unterminated byte literal"));
                continue;
            }
            i += 1;
            let q = if quote == b'"' { '"' } else { '\'' };
            if non_ascii {
                // As in Rust: other bytes are written as escapes.
                diags.push(
                    Diag::new(
                        "E0101",
                        sp(lo, i),
                        "a byte literal holds ASCII characters and escapes",
                    )
                    .fix(
                        "write the other bytes as `\\xHH`",
                        Some(byte_text(&bytes, q)),
                    ),
                );
            }
            let tok = if quote == b'"' {
                Tok::Bytes(bytes)
            } else {
                if bytes.len() != 1 {
                    diags.push(Diag::new(
                        "E0101",
                        sp(lo, i),
                        "a byte literal holds one byte; use `b\"...\"` for several",
                    ));
                }
                Tok::Byte(bytes.first().copied().unwrap_or(0))
            };
            out.push(Token {
                tok,
                span: sp(lo, i),
            });
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
            // `0x`, `0o`, `0b` and `_` separators (ADR-0048 §3), as in Rust.
            let radix = match (c, b.get(i + 1)) {
                (b'0', Some(b'x')) => Radix::Hex,
                (b'0', Some(b'o')) => Radix::Oct,
                (b'0', Some(b'b')) => Radix::Bin,
                _ => Radix::Dec,
            };
            if radix != Radix::Dec {
                i += 2;
            }
            let base: u32 = match radix {
                Radix::Dec => 10,
                Radix::Hex => 16,
                Radix::Oct => 8,
                Radix::Bin => 2,
            };
            // Digits: hexadecimal ones in a `0x` literal, decimal ones
            // otherwise (a decimal digit outside the base is an error, as in
            // Rust); what follows is a suffix.
            let digit = |ch: u8| {
                if radix == Radix::Hex {
                    ch.is_ascii_hexdigit()
                } else {
                    ch.is_ascii_digit()
                }
            };
            let mut v: u128 = 0;
            let mut overflow = false;
            let mut any = false;
            let mut bad: Option<(usize, char)> = None;
            while i < b.len() && (digit(b[i]) || b[i] == b'_') {
                if b[i] != b'_' {
                    any = true;
                    let d = (b[i] as char).to_digit(16).expect("a digit");
                    if d >= base {
                        bad.get_or_insert((i, b[i] as char));
                    } else {
                        match v
                            .checked_mul(base as u128)
                            .and_then(|x| x.checked_add(d as u128))
                        {
                            Some(x) => v = x,
                            None => overflow = true,
                        }
                    }
                }
                i += 1;
            }
            let digits_end = i;
            let name = match radix {
                Radix::Hex => "hexadecimal",
                Radix::Oct => "octal",
                Radix::Bin => "binary",
                Radix::Dec => "decimal",
            };
            if let Some((at, ch)) = bad {
                let allowed = if radix == Radix::Bin {
                    "`0` and `1`"
                } else {
                    "`0` to `7`"
                };
                diags.push(
                    Diag::new(
                        "E0108",
                        sp(at, at + 1),
                        format!("`{ch}` is not {} {name} digit", if radix == Radix::Oct { "an" } else { "a" }),
                    )
                    .fix(
                        format!(
                            "{name} digits are {allowed}; write the value in another base (`0x` hexadecimal, or decimal)"
                        ),
                        None,
                    ),
                );
            }
            let no_digits = !any && radix != Radix::Dec;
            if no_digits {
                diags.push(
                    Diag::new(
                        "E0108",
                        sp(lo, digits_end),
                        format!("`{}` has no {name} digits", &text[lo..digits_end]),
                    )
                    .fix(
                        format!("write the digits after the prefix: `{}`", radix.text(0)),
                        None,
                    ),
                );
            }
            if i < b.len() && (b[i].is_ascii_alphanumeric() || b[i] == b'_') {
                let slo = i;
                while i < b.len() && (b[i].is_ascii_alphanumeric() || b[i] == b'_') {
                    i += 1;
                }
                let suffix = &text[slo..i];
                let written = &text[lo..digits_end];
                const TYPES: [&str; 12] = [
                    "i8", "i16", "i32", "i64", "i128", "isize", "u8", "u16", "u32", "u64", "u128",
                    "usize",
                ];
                let d = if no_digits {
                    // Reported above: the suffix is what the digits would be.
                    None
                } else if TYPES.contains(&suffix) {
                    Diag::new(
                        "E0104",
                        sp(lo, i),
                        format!("literal suffix `{suffix}` is not allowed"),
                    )
                    .fix("remove the suffix", Some(written.to_string()))
                    .into()
                } else {
                    Diag::new(
                        "E0104",
                        sp(lo, i),
                        format!("literal suffix `{suffix}` is not allowed"),
                    )
                    .fix(
                        "integer literals are digits in base 10, or after `0x`, `0o` or `0b`",
                        None,
                    )
                    .into()
                };
                if let Some(d) = d {
                    diags.push(d);
                }
            }
            if overflow {
                diags.push(Diag::new(
                    "E0105",
                    sp(lo, i),
                    "integer literal does not fit in 128 bits",
                ));
            }
            out.push(Token {
                tok: Tok::Int(v, radix),
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
        // `'a'`, `'\n'`: a character literal (Rust `char`). A byte is `b'a'`.
        if c == b'\'' {
            let mut j = i + 1;
            if b.get(j) == Some(&b'\\') {
                j += escape(b, j).map_or(2, |(_, n)| n);
            } else if let Some(ch) = text.get(j..).and_then(|t| t.chars().next()) {
                j += ch.len_utf8();
            }
            if b.get(j) == Some(&b'\'') && j > i + 1 {
                let lit = &text[i..=j];
                let mut d = Diag::new(
                    "E0101",
                    sp(i, j + 1),
                    "character literals are not supported",
                );
                d = if lit.is_ascii() {
                    d.fix("a byte is `b'..'` (`u8`)", Some(format!("b{lit}")))
                } else {
                    d.fix(
                        "bytes are `u8`; write the UTF-8 bytes with `b\"\\xHH..\"`",
                        None,
                    )
                };
                diags.push(d);
                // Recovery: the byte it stands for, so parsing goes on.
                let v = if b[i + 1] == b'\\' {
                    escape(b, i + 1).map_or(0, |(v, _)| v)
                } else {
                    b[i + 1]
                };
                out.push(Token {
                    tok: Tok::Byte(v),
                    span: sp(i, j + 1),
                });
                i = j + 1;
                continue;
            }
        }
        if c == b'\'' {
            let lo = i;
            i += 1;
            while i < b.len() && (b[i].is_ascii_alphanumeric() || b[i] == b'_') {
                i += 1;
            }
            // A label: after `break`/`continue`, or before a `:` (a loop
            // label; a lifetime takes none in BMB). Comments may come between.
            let after_jump = matches!(out.last(), Some(Token { tok: Tok::Ident(k), .. }) if k == "break" || k == "continue");
            let mut rest = &text[i..];
            loop {
                let t = rest.trim_start();
                match t.strip_prefix("//") {
                    Some(c) => rest = c.find('\n').map_or("", |n| &c[n..]),
                    None => {
                        rest = t;
                        break;
                    }
                }
            }
            let before_loop = rest.starts_with(':') && !rest.starts_with("::");
            if i > lo + 1 && (after_jump || before_loop) && !(i < b.len() && b[i] == b'\'') {
                out.push(Token {
                    tok: Tok::Label(text[lo + 1..i].to_string()),
                    span: sp(lo, i),
                });
                continue;
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

/// The escape at `b[i]` (a backslash) in a byte literal: its value and its
/// length in the source (ADR-0044 §2, as Rust).
fn escape(b: &[u8], i: usize) -> Option<(u8, usize)> {
    let v = match *b.get(i + 1)? {
        b'n' => b'\n',
        b'r' => b'\r',
        b't' => b'\t',
        b'\\' => b'\\',
        b'\'' => b'\'',
        b'"' => b'"',
        b'0' => 0,
        b'x' => {
            let h = std::str::from_utf8(b.get(i + 2..i + 4)?).ok()?;
            return Some((u8::from_str_radix(h, 16).ok()?, 4));
        }
        _ => return None,
    };
    Some((v, 2))
}

/// `bytes` written as a byte literal with quote `q` (`'` or `"`): printable
/// ASCII as itself, the rest as escapes. The normal form and fixes use it.
pub fn byte_text(bytes: &[u8], q: char) -> String {
    let mut s = format!("b{q}");
    for &c in bytes {
        match c {
            b'\n' => s.push_str("\\n"),
            b'\r' => s.push_str("\\r"),
            b'\t' => s.push_str("\\t"),
            b'\\' => s.push_str("\\\\"),
            0 => s.push_str("\\0"),
            c if c as char == q => {
                s.push('\\');
                s.push(q);
            }
            0x20..=0x7e => s.push(c as char),
            c => s.push_str(&format!("\\x{c:02x}")),
        }
    }
    s.push(q);
    s
}
